"""Install verified official Rust GNU components inside this project only."""
import hashlib
import pathlib
import shutil
import tarfile
import tomllib
import urllib.request
from concurrent.futures import ThreadPoolExecutor

root = pathlib.Path(__file__).resolve().parents[1]
manifest = tomllib.loads((root / 'evidence/rust-channel.toml').read_text(encoding='utf-8'))
opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
cache = root / '.tools/downloads'
prefix = root / '.tools/rust'
cache.mkdir(parents=True, exist_ok=True)
prefix.mkdir(parents=True, exist_ok=True)
for package in ['rustc', 'cargo', 'rust-std', 'rust-mingw']:
    target = manifest['pkg'][package]['target']['x86_64-pc-windows-gnu']
    url = target['xz_url']
    archive = cache / url.rsplit('/', 1)[1]
    with opener.open(urllib.request.Request(url, method='HEAD'), timeout=30) as response:
        total = int(response.headers['Content-Length'])
    existing = archive.stat().st_size if archive.exists() else 0
    if existing < total:
        print('Downloading', package, existing, '/', total, flush=True)
        ranges = [(start, min(start + 1048576, total) - 1) for start in range(existing, total, 1048576)]
        def download(pair):
            start, end = pair
            request = urllib.request.Request(url, headers={'Range': f'bytes={start}-{end}'})
            for attempt in range(4):
                try:
                    with opener.open(request, timeout=30) as response:
                        data = response.read()
                    if len(data) != end - start + 1:
                        raise RuntimeError('Partial response length mismatch')
                    return data
                except Exception:
                    if attempt == 3:
                        raise
            raise RuntimeError('Unreachable retry state')
        with ThreadPoolExecutor(max_workers=6) as pool, archive.open('ab') as output:
            for data in pool.map(download, ranges):
                output.write(data)
                output.flush()
                print('Received', package, output.tell(), '/', total, flush=True)
    if hashlib.sha256(archive.read_bytes()).hexdigest() != target['xz_hash']:
        raise RuntimeError(f'Invalid SHA256: {archive}')
    extracted = cache / archive.name.removesuffix('.tar.xz')
    if not extracted.exists():
        with tarfile.open(archive) as tar:
            tar.extractall(cache, filter='data')
    for component in (extracted / 'components').read_text().splitlines():
        component_root = extracted / component
        for line in (component_root / 'manifest.in').read_text().splitlines():
            kind, relative = line.split(':', 1)
            source = component_root / relative
            destination = prefix / relative
            if kind == 'dir':
                shutil.copytree(source, destination, dirs_exist_ok=True)
            elif kind == 'file':
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(source, destination)
    print('Installed', package, flush=True)
assembler = shutil.which('as.exe') or shutil.which('x86_64-w64-mingw32-as.exe')
if assembler:
    destination = prefix / 'lib/rustlib/x86_64-pc-windows-gnu/bin/self-contained'
    for name in ['as.exe', 'x86_64-w64-mingw32-as.exe']:
        shutil.copy2(assembler, destination / name)
else:
    print('GNU fallback requires MinGW-w64 binutils (as.exe) on PATH.', flush=True)
print('Rust prefix:', prefix, flush=True)
