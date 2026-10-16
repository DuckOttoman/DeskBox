"""Read-only DeskBox catalog origin. Bind loopback behind a TLS proxy."""
import argparse
import json
import re
import socket
import ssl
import threading
from pathlib import Path
from urllib.parse import unquote, urlsplit
from http.server import BaseHTTPRequestHandler, HTTPServer
from socketserver import ThreadingMixIn

def validate_catalog(value):
 if not isinstance(value,dict) or value.get('schemaVersion')!=1 or not isinstance(value.get('entries'),list):raise ValueError('Invalid catalog schema')
 if len(value['entries'])>1000:raise ValueError('Too many entries')
 seen=set()
 for e in value['entries']:
  if not isinstance(e,dict):raise ValueError('Invalid entry')
  for k in ('id','name','description','author','type','version','url','sha256'):
   if not isinstance(e.get(k),str) or len(e[k])>2048:raise ValueError('Invalid '+k)
  if not re.fullmatch(r'[a-z0-9]+(?:[.-][a-z0-9]+)+',e['id']) or e['id'] in seen:raise ValueError('Invalid or duplicate ID')
  seen.add(e['id'])
  if e['type'] not in ('widget','theme') or not re.fullmatch(r'\d+\.\d+\.\d+',e['version']):raise ValueError('Invalid type/version')
  u=urlsplit(e['url'])
  if u.scheme!='https' or not u.hostname or u.username or u.password or u.fragment:raise ValueError('HTTPS URL required')
  if not re.fullmatch('[a-f0-9]{64}',e['sha256']):raise ValueError('Invalid hash')
 return value

def public_path(root,url):
 path=unquote(urlsplit(url).path)
 if path=='/catalog.json':parts=['catalog.json']
 elif re.fullmatch(r'/packages/[a-zA-Z0-9._-]+\.deskbox-plugin',path):parts=path.lstrip('/').split('/')
 else:return None
 p=root
 for part in parts:
  p=p/part
  if p.is_symlink():return None
 return p if p.is_file() else None

class Handler(BaseHTTPRequestHandler):
 server_version='DeskBoxMarket/1'
 def setup(self):
  if getattr(self.server,'tls',None):self.request=self.server.tls.wrap_socket(self.request,server_side=True)
  super().setup()
 def do_HEAD(self):self.serve(False)
 def do_GET(self):self.serve(True)
 def serve(self,body):
  if self.path=='/health':
   data=b'{"status":"ok","schemaVersion":1}';kind='application/json';cache='no-store'
  else:
   p=public_path(self.server.root,self.path)
   if p is None:self.send_error(404);return
   if p.stat().st_size>20*1024*1024:self.send_error(413);return
   data=p.read_bytes();kind='application/octet-stream';cache='public, max-age=3600'
   if p.name=='catalog.json':
    try:data=json.dumps(validate_catalog(json.loads(data.decode('utf-8-sig'))),ensure_ascii=False).encode('utf8')
    except (ValueError,UnicodeError):self.send_error(503);return
    kind='application/json; charset=utf-8';cache='no-cache'
  self.send_response(200)
  for k,v in [('Content-Type',kind),('Content-Length',str(len(data))),('Cache-Control',cache),('X-Content-Type-Options','nosniff'),('Access-Control-Allow-Origin','*')]:self.send_header(k,v)
  self.end_headers()
  if body:self.wfile.write(data)
class Server(ThreadingMixIn,HTTPServer):
 daemon_threads=True
 slots=threading.BoundedSemaphore(32)
 def process_request(self,request,address):
  if not self.slots.acquire(False):request.close();return
  try:super().process_request(request,address)
  except BaseException:self.slots.release();raise
 def process_request_thread(self,request,address):
  try:super().process_request_thread(request,address)
  finally:self.slots.release()
 def get_request(self):
  sock,addr=super().get_request();sock.settimeout(15);return sock,addr
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--root',required=True);p.add_argument('--port',type=int,default=8765);p.add_argument('--socket-activation',action='store_true');p.add_argument('--cert');p.add_argument('--key');a=p.parse_args()
 root=Path(a.root).resolve();validate_catalog(json.loads((root/'catalog.json').read_text(encoding='utf-8-sig')))
 s=Server(('127.0.0.1',a.port),Handler,bind_and_activate=not a.socket_activation)
 if a.socket_activation:
  s.socket.close();s.socket=socket.fromfd(3,socket.AF_INET,socket.SOCK_STREAM)
 if a.cert:
  ctx=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER);ctx.options|=ssl.OP_NO_TLSv1|ssl.OP_NO_TLSv1_1;ctx.load_cert_chain(a.cert,a.key);s.tls=ctx
 s.root=root;s.serve_forever()
