"""Package verified fence merge build; runtime fixture configuration is excluded."""
from pathlib import Path
import hashlib,json,shutil,zipfile
root=Path(__file__).resolve().parents[1]
build=(root/'evidence/fence-merge-build.log').read_text(encoding='utf-8')
assert 'Finished `release` profile' in build
core=(root/'evidence/fence-merge-core.log').read_text(encoding='utf-8')
for count in [8,14,4,5,2,7]: assert f'{count} passed; 0 failed' in core
for name in ['fence-merge-menu13','fence-merge-menu60','fence-merge-settings']:
 log=(root/'evidence'/name/'runtime.log').read_text(encoding='utf-8')
 assert 'SMOKE_RESULT success=true' in log and 'FRONTEND_ERROR' not in log,name
runtime=(root/'evidence/fence-merge-native-final/runtime.log').read_text(encoding='utf-8')
assert 'SMOKE_RESULT success=true' in runtime and 'FRONTEND_ERROR' not in runtime
for name in ['held_contents_never_commits','cancel_restores_all','contents_release_commits','restore_consumed_and_split']:
 assert name in runtime,name
for filename in ['deskbox-reconstructed.exe','WebView2Loader.dll']:
 shutil.copy2(root/'target/release'/filename,root/'bin'/filename)
readme=root/'README.md';text=readme.read_text(encoding='utf-8').replace('26 项 Rust 回归测试','40 项 Rust 回归测试')
if '## 文件夹框组合与内容合并' not in text:
 text+='\n## 文件夹框组合与内容合并\n\n左键拖动标题到另一框上，停留不足五秒松手组合外框；单击组合名称切换成员内容，拖动当前成员到空白处分离。停留达到五秒会预览内容合并，只有松开左键才写入配置，拖走、Esc 或取消会撤销预览。内容合并保留目标属性，右键框标题的“还原内容合并”子菜单可逐次还原；两个框继承还原时目标属性并恢复原名称。组合和还原点持久保存，不移动磁盘文件。\n\n主题流彩标题、合并气泡、150% 目标放大、邻框临时淡化、图标批次流入/流回和分裂还原由独立动画层处理。数据规则、模拟指针与原生 WebView 交互已验证；实际桌面父窗口上的完整真实鼠标与双屏不同 DPI 动画仍需实测。诊断参数 --fence-merge-check 使用隔离测试文件。\n'
readme.write_text(text,encoding='utf-8')
dest=root/'deliverables/deskbox-reconstructed-windows-x64-fence-merge';dest.mkdir(exist_ok=True)
for filename in ['deskbox-reconstructed.exe','WebView2Loader.dll']:shutil.copy2(root/'bin'/filename,dest/filename)
shutil.copy2(readme,dest/'README.md')
archive=dest.with_suffix('.zip')
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
 for p in sorted(dest.iterdir()):z.write(p,dest.name+'/'+p.name)
exe=root/'bin/deskbox-reconstructed.exe'
with zipfile.ZipFile(archive) as z:assert z.read(dest.name+'/'+exe.name)==exe.read_bytes()
result={'runtime_archive':str(archive),'executable_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'core_tests_passed':40,'native_merge_checks':[line for line in runtime.splitlines() if line.startswith('FRONTEND_CONTRACT_CHECKS')],'real_pointer_full_flow_verified':False}
(root/'evidence/fence-merge-verification.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(result,ensure_ascii=False,indent=2))
