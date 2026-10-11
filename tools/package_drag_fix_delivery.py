"""Verify and package the drag focus handoff fix without runtime/user data."""
from pathlib import Path
import hashlib,json,shutil,zipfile
root=Path(__file__).resolve().parents[1]
assert 'Finished `release` profile' in (root/'evidence/drag-fix-build.log').read_text(encoding='utf-8')
assert 'native focus handoff without rollback' in (root/'evidence/drag-fix-pointer-green.log').read_text(encoding='utf-8')
for name in ['drag-fix-merge','drag-fix-menu13-final','drag-fix-menu60-final','drag-fix-settings']:
 log=(root/'evidence'/name/'runtime.log').read_text(encoding='utf-8')
 assert 'SMOKE_RESULT success=true' in log and 'FRONTEND_ERROR' not in log,name
trace=(root/'evidence/drag-fix-real-pointer/runtime.log').read_text(encoding='utf-8')
assert 'FRONTEND_ERROR' not in trace
events=[json.loads(line.split('drag_event:',1)[1]) for line in trace.splitlines() if 'drag_event:' in line]
start=next(i for i,e in enumerate(events) if e['type']=='pointerdown')
end=next(i for i in range(start+1,len(events)) if events[i]['type']=='pointerup')
assert events[end]['dragging'] is True
assert any(e['type']=='blur' and e['dragging'] for e in events[start:end])
readme=root/'README.md';text=readme.read_text(encoding='utf-8')
if '## 拖动回弹修复' not in text:
 text+='\n## 拖动回弹修复\n\n修复原生键盘焦点交接的临时失焦被当成取消拖动、导致文件夹框恢复原位置的问题。交接完成后的真正失焦继续取消，松开左键才保存或合并。使用真实鼠标及真实 Win32/WebView2 焦点流程复现并复测；完整桌面子窗口长按合并与多屏不同 DPI 尚未实测。\n'
readme.write_text(text,encoding='utf-8')
dest=root/'deliverables/deskbox-reconstructed-windows-x64-fence-merge-drag-fix';dest.mkdir(exist_ok=True)
for name in ['deskbox-reconstructed.exe','WebView2Loader.dll']:
 shutil.copy2(root/'target/release'/name,root/'bin'/name)
 shutil.copy2(root/'bin'/name,dest/name)
shutil.copy2(readme,dest/'README.md')
archive=dest.with_suffix('.zip')
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
 for p in sorted(dest.iterdir()):z.write(p,dest.name+'/'+p.name)
exe=root/'bin/deskbox-reconstructed.exe'
with zipfile.ZipFile(archive) as z:assert z.read(dest.name+'/'+exe.name)==exe.read_bytes()
result={'runtime_archive':str(archive),'executable_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'real_mouse_focus_handoff_drag_saved':True,'first_observed_saved_position':[129,191],'second_automated_drag_interrupted_by_user_input':True,'pointer_regression_passed':True,'native_merge_menu_settings_passed':True,'desktop_child_window_full_flow_verified':False,'core_tests_recorded':40}
(root/'evidence/drag-fix-verification.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(result,ensure_ascii=False,indent=2))
