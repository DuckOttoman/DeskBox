"""Package the verified learning and monitor update without private test data."""
from pathlib import Path
import hashlib,json,shutil,zipfile
root=Path(__file__).resolve().parents[1]
for filename in ['deskbox-reconstructed.exe','WebView2Loader.dll']:
 shutil.copy2(root/'target/release'/filename,root/'bin'/filename)
readme=root/'README.md'
text=readme.read_text(encoding='utf-8').replace('20 项 Rust 回归测试','26 项 Rust 回归测试')
text=text.replace('分类优先级、t:/u: 目标学习键、首次/完整整理布局和部分默认值仍未证明与原后端完全相同。','t:/u:/n: 学习键、移动写入及整理查表已依据原机器码恢复。分类优先级、首次/完整整理布局和部分默认值仍未证明与原后端完全相同。')
if '## 分类学习和断屏保存修正' not in text:
 text+='\n## 分类学习和断屏保存修正\n\n快捷方式按目标文件名（保留扩展名）、网址按完整小写 URL、其他项目按小写显示名称学习；移动到新文件或无分类自定义框保留已有规则。扫描会识别只改变快捷方式目标的更新。断屏对象在主屏上操作时可保存原 monitor 归属，避免拖动后位置回弹。证据与测试边界见 evidence/learning-drag-monitor-analysis.md。真实鼠标完整拖动、双屏不同 DPI 和热插拔仍未通过受控端到端验证。\n'
readme.write_text(text,encoding='utf-8')
dest=root/'deliverables/deskbox-reconstructed-windows-x64-learning-drag'
dest.mkdir(exist_ok=True)
for filename in ['deskbox-reconstructed.exe','WebView2Loader.dll']:
 shutil.copy2(root/'bin'/filename,dest/filename)
shutil.copy2(readme,dest/'README.md')
archive=dest.with_suffix('.zip')
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
 for p in sorted(dest.iterdir()):z.write(p,dest.name+'/'+p.name)
exe=root/'bin/deskbox-reconstructed.exe'
assert exe.read_bytes()==(root/'target/release/deskbox-reconstructed.exe').read_bytes()
with zipfile.ZipFile(archive) as z:assert z.read(dest.name+'/'+exe.name)==exe.read_bytes()
print(json.dumps({'runtime_archive':str(archive),'executable_sha256':hashlib.sha256(exe.read_bytes()).hexdigest()},indent=2))
