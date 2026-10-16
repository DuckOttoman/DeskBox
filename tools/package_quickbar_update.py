"""Package the verified quick-access-bar replacement payload."""
from pathlib import Path
import hashlib,json,zipfile
r=Path(__file__).resolve().parents[1]
report=json.loads((r/'evidence/quickbar-verification.json').read_text(encoding='utf8'))
assert report['normalExit'] and report['mainCrashRecovery'] and report['dom'] and report['dockAnchoredToScreenBottom']
files=['deskbox-reconstructed.exe','WebView2Loader.dll']+['sensor-tools/'+p.name for p in sorted((r/'target/release/sensor-tools').iterdir()) if p.is_file()]
hashes={n:hashlib.sha256((r/'target/release'/n).read_bytes()).hexdigest() for n in files}
assert report['exeSha256']==hashes['deskbox-reconstructed.exe'],'Release changed after verification'
out=r/'deliverables/DeskBox-Reconstructed-2026-10-16-quickbar-update.zip'
with zipfile.ZipFile(out,'w',zipfile.ZIP_DEFLATED) as z:
 for n in files:z.write(r/'target/release'/n,n)
 z.writestr('sha256.json',json.dumps(hashes,indent=2))
 z.writestr('更新说明.txt','退出 DeskBox 后，将本包所有文件及 sensor-tools 文件夹复制到原安装目录并替换。保留现有配置、Albums、media-tools。本版修复拖动图标被快捷栏遮挡的问题；靠近时横向展开并显示插入槽，离开收回，松开鼠标才固定。修复 Windows 预留工作区使快捷栏上移 96px 的问题，恢复贴齐屏幕底边。设置 → 行为新增“隐藏任务栏”和快捷栏数量；默认关闭，打开后启用快捷访问栏。关闭或退出恢复系统任务栏。请勿同时强制终止主程序与恢复守护。'.encode('utf8'))
print(json.dumps({'path':str(out),'bytes':out.stat().st_size,'sha256':hashlib.sha256(out.read_bytes()).hexdigest()},indent=2))
