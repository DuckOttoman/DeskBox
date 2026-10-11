"""Recover original Brotli assets from this DeskBox build's PE slice table."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import brotli

parser = argparse.ArgumentParser()
parser.add_argument('binary', type=Path)
parser.add_argument('--output', type=Path, default=Path(__file__).resolve().parents[1] / 'dist')
args = parser.parse_args()
binary = args.binary.read_bytes()
expected = '86712e4115ffe403c153bb96803c755a5afe6efa9f8f02171f4c62326f3509a7'
if hashlib.sha256(binary).hexdigest() != expected:
    raise SystemExit('This extractor supports only the recorded DeskBox binary SHA256.')
pe = struct.unpack_from('<I', binary, 60)[0]
optional_size = struct.unpack_from('<H', binary, pe + 20)[0]
base = struct.unpack_from('<Q', binary, pe + 48)[0]
sections = [struct.unpack_from('<8sIIII', binary, pe + 24 + optional_size + i * 40)
            for i in range(struct.unpack_from('<H', binary, pe + 6)[0])]

def file_offset(address):
    for _, _, virtual_address, raw_size, raw_offset in sections:
        if virtual_address <= address - base < virtual_address + raw_size:
            return raw_offset + address - base - virtual_address
    raise ValueError(f'Unmapped pointer: {address:#x}')

records = []
for offset in [0x3e2948, 0x3e2968, 0x3e2988]:
    name_pointer, name_size, data_pointer, data_size = struct.unpack_from('<QQQQ', binary, offset)
    n, p = file_offset(name_pointer), file_offset(data_pointer)
    name = binary[n:n + name_size].decode('utf-8')
    data = brotli.decompress(binary[p:p + data_size])
    output = (args.output / name.lstrip('/')).resolve()
    if not output.is_relative_to(args.output.resolve()):
        raise ValueError('Asset escapes output directory')
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(data)
    records.append(dict(path=name, table_offset=hex(offset), data_offset=hex(p),
                        compressed_size=data_size, size=len(data), sha256=hashlib.sha256(data).hexdigest()))
    print(name, len(data))
evidence = Path(__file__).resolve().parents[1] / 'evidence/assets.json'
evidence.write_text(json.dumps(records, indent=2), encoding='utf-8')

def string(pointer, length):
    offset = file_offset(pointer)
    return binary[offset:offset + length].decode('utf-8')

rules = {'categories': [], 'extensions': {}, 'keywords': {}}
for offset in range(0x3e2b18, 0x3e2d48, 40):
    ip, il, np, nl, order = struct.unpack_from('<QQQQQ', binary, offset)
    rules['categories'].append(dict(id=string(ip, il), name=string(np, nl), order=order))
for key, start, end in [('extensions', 0x3e5c10, 0x3e5cd0), ('keywords', 0x3e9090, 0x3e9170)]:
    for offset in range(start, end, 32):
        cp, cl, rp, count = struct.unpack_from('<QQQQ', binary, offset)
        rules[key][string(cp, cl)] = [string(*struct.unpack_from('<QQ', binary, file_offset(rp) + i * 16)) for i in range(count)]
(evidence.parent / 'classification-rules.json').write_text(json.dumps(rules, ensure_ascii=False, indent=2), encoding='utf-8')

# PE resource directory: RT_ICON (3), RT_GROUP_ICON (14).
resource_rva = struct.unpack_from('<I', binary, pe + 24 + 112 + 2 * 8)[0]
resource_base = file_offset(base + resource_rva)
def resources(offset=0, path=()):
    directory = resource_base + offset
    named, numbered = struct.unpack_from('<HH', binary, directory + 12)
    for i in range(named + numbered):
        name, entry = struct.unpack_from('<II', binary, directory + 16 + i * 8)
        if entry & 0x80000000:
            yield from resources(entry & 0x7fffffff, path + (name,))
        else:
            rva, size = struct.unpack_from('<II', binary, resource_base + entry)
            pos = file_offset(base + rva)
            yield path + (name,), binary[pos:pos + size]
blobs = dict(resources())
group = next(data for path, data in blobs.items() if path[0] == 14)
count = struct.unpack_from('<H', group, 4)[0]
entries, images = [], []
cursor = 6 + count * 16
for i in range(count):
    entry = group[6 + i * 14:20 + i * 14]
    icon_id = struct.unpack_from('<H', entry, 12)[0]
    data = next(data for path, data in blobs.items() if path[0] == 3 and path[1] == icon_id)
    entries.append(entry[:8] + struct.pack('<II', len(data), cursor))
    images.append(data)
    cursor += len(data)
icons = evidence.parent.parent / 'icons'
icons.mkdir(exist_ok=True)
(icons / 'icon.ico').write_bytes(group[:6] + b''.join(entries) + b''.join(images))
(icons / 'icon.png').write_bytes(next(data for data in images if data.startswith(b'\x89PNG')))
