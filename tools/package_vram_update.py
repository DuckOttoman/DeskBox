from pathlib import Path
import json,hashlib,zipfile
r=Path(__file__).resolve().parents[1]
log=(r/'evidence/vram-display-native/runtime.log').read_text(encoding='utf8')
assert 'VRAM_DISPLAY_NATIVE_SUCCESS' in log and 'FRONTEND_ERROR' not in log
assert 'Finished `release` profile' in (r/'evidence/vram-display-build.log').read_text(encoding='utf8')
files=['deskbox-reconstructed.exe','WebView2Loader.dll']+['sensor-tools/'+p.name for p in sorted((r/'target/release/sensor-tools').iterdir()) if p.is_file()]
hashes={n:hashlib.sha256((r/'target/release'/n).read_bytes()).hexdigest() for n in files}
p=r/'deliverables/DeskBox-Reconstructed-2026-10-15-vram-display-update.zip'
with zipfile.ZipFile(p,'w',zipfile.ZIP_DEFLATED) as z:
 for n in files:z.write(r/'target/release'/n,n)
 z.writestr('sha256.json',json.dumps(hashes,indent=2));z.writestr('更新说明.txt','退出 DeskBox 后将本包所有文件和 sensor-tools 文件夹复制到现有安装目录并替换。保留配置、Albums、media-tools。此版显存改为与内存一致的标题、已用 / 总量 GB 和用量进度条；无需重新启用已正常工作的 CPU 温度服务。'.encode('utf8'))
print(json.dumps({'update':str(p),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()},indent=2))
