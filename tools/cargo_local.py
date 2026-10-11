"""Run workspace-local Rust; relay official crate downloads through Python TLS.

The relay binds localhost only, forwards to two fixed official hosts, and ends
with Cargo. It avoids Windows Schannel's missing-credentials error in sandbox.
"""
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import subprocess
import sys
import threading
import urllib.error
import urllib.request

root = Path(__file__).resolve().parents[1]
cache = root / '.tools/relay-cache'
cache.mkdir(parents=True, exist_ok=True)
opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_GET(self):
        path = self.path.split('?', 1)[0]
        if path == '/index/config.json':
            data = json.dumps({'dl': f'http://127.0.0.1:{self.server.server_port}/crates',
                               'api': 'https://crates.io'}).encode()
        else:
            if path.startswith('/index/'):
                url = 'https://index.crates.io/' + path[len('/index/'):]
            elif path.startswith('/crates/'):
                parts = path.split('/')
                if len(parts) != 5 or parts[4] != 'download':
                    self.send_error(400)
                    return
                name, version = parts[2:4]
                url = f'https://static.crates.io/crates/{name}/{name}-{version}.crate'
            else:
                self.send_error(404)
                return
            key = cache / hashlib.sha256(url.encode()).hexdigest()
            try:
                if key.exists():
                    data = key.read_bytes()
                else:
                    with opener.open(url, timeout=90) as response:
                        data = response.read()
                    key.write_bytes(data)
            except urllib.error.HTTPError as error:
                self.send_error(error.code)
                return
            except Exception as error:
                print('Download relay:', url, str(error), flush=True)
                self.send_error(502)
                return
        self.send_response(200)
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        try:
            self.wfile.write(data)
        except (BrokenPipeError, ConnectionResetError):
            pass

server = ThreadingHTTPServer(('127.0.0.1', 18767), Handler)
thread = threading.Thread(target=server.serve_forever, daemon=True)
thread.start()
configuration = root / '.tools/relay-config.toml'
configuration.write_text('[source.crates-io]\nreplace-with = "local-relay"\n'
                         '[source.local-relay]\nregistry = "sparse+http://127.0.0.1:18767/index/"\n', encoding='utf-8')
env = os.environ.copy()
env.update(CARGO_HOME=str(root / '.tools/cargo-home'), RUSTC=str(root / '.tools/rust/bin/rustc.exe'),
           CARGO_NET_OFFLINE='false', CARGO_HTTP_PROXY='', CARGO_HTTP_MULTIPLEXING='false',
           HTTP_PROXY='', HTTPS_PROXY='', ALL_PROXY='')
env['PATH'] = (str(root / '.tools/rust/lib/rustlib/x86_64-pc-windows-gnu/bin/self-contained')
               + os.pathsep + str(root / '.tools/rust/bin') + os.pathsep + env['PATH'])
env['RUSTFLAGS'] = '-C link-self-contained=yes'
try:
    result = subprocess.run([str(root / '.tools/rust/bin/cargo.exe'), '--config', str(configuration),
                             *sys.argv[1:]], cwd=root, env=env)
finally:
    server.shutdown()
sys.exit(result.returncode)
