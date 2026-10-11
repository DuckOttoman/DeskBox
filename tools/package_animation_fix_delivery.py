"""Package verified native-grid merge animation fix without runtime data."""
from pathlib import Path
import hashlib,json,shutil,zipfile
root=Path(__file__).resolve().parents[1]
assert 'Finished `release` profile' in (root/'evidence/animation-fix-build.log').read_text(encoding='utf-8')
for name in ['animation-fix-green','animation-fix-menu13','animation-fix-menu60','animation-fix-settings']:
 log=(root/'evidence'/name/'runtime.log').read_text(encoding='utf-8')
 assert 'SMOKE_RESULT success=true' in log and 'FRONTEND_ERROR' not in log,name
log=(root/'evidence/animation-fix-green/runtime.log').read_text(encoding='utf-8')
checks=['preview_slots_append_after_existing_items','preview_first_slot_follows_last_item','preview_icons_match_target_size','preview_has_no_body_particles','cancel_immediately_restores_in_source','held_contents_never_commits','contents_release_commits']
for check in checks:assert check in log,check
readme=root/'README.md';text=readme.read_text(encoding='utf-8')
if '## 内容合并动画修正' not in text:
 text+='\n## 内容合并动画修正\n\n预览图标使用目标框的真实网格与尺寸，从已有最后一项之后续排并自动换行。取消预览时删除动画副本，让图标直接在原框内恢复，避免框外返回闪现。源24px/目标64px及目标已有内容的原生WebView布局回归已通过；松开左键才提交的规则保留。\n'
readme.write_text(text,encoding='utf-8')
dest=root/'deliverables/deskbox-reconstructed-windows-x64-merge-animation-fix';dest.mkdir(exist_ok=True)
for name in ['deskbox-reconstructed.exe','WebView2Loader.dll']:
 shutil.copy2(root/'target/release'/name,root/'bin'/name)
 shutil.copy2(root/'bin'/name,dest/name)
shutil.copy2(readme,dest/'README.md')
archive=dest.with_suffix('.zip')
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
 for p in sorted(dest.iterdir()):z.write(p,dest.name+'/'+p.name)
exe=root/'bin/deskbox-reconstructed.exe'
with zipfile.ZipFile(archive) as z:assert z.read(dest.name+'/'+exe.name)==exe.read_bytes()
result={'runtime_archive':str(archive),'executable_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'native_webview_animation_checks':checks,'physical_long_hold_animation_verified':False}
(root/'evidence/merge-animation-verification.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(result,ensure_ascii=False,indent=2))
