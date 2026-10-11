"""Package verified media build including its decoding/transcoding helper."""
from pathlib import Path
import hashlib,json,shutil,zipfile
root=Path(__file__).resolve().parents[1]
assert 'Finished `release` profile' in (root/'evidence/icon-radius-build.log').read_text(encoding='utf-8')
log=(root/'evidence/icon-radius-green/runtime.log').read_text(encoding='utf-8')
checks=['media_eco_actual_480p','media_clock_advances','media_two_ui_modes','media_original_actual_resolution','media_no_poster_ghosting','media_collapsed_releases_source','media_gif_loaded','media_static_replacement_visible','media_clear_removes_decoder','ICON_RADIUS_NATIVE_SUCCESS','media_rounded_clip_matches_fence','media_radius_change_updates_clip','icon_names_hidden_preserve_items_and_tooltips','icon_names_show_restores_labels']
for check in checks:assert check in log,check
assert 'FRONTEND_ERROR' not in log
for run in ['icon-radius-merge','icon-radius-settings']:
    value=(root/'evidence'/run/'runtime.log').read_text(encoding='utf-8')
    assert 'SMOKE_RESULT success=true' in value and 'FRONTEND_ERROR' not in value,run
dest=root/'deliverables/deskbox-reconstructed-windows-x64-icon-names-radius';dest.mkdir(exist_ok=True)
for name in ['deskbox-reconstructed.exe','WebView2Loader.dll']:
    shutil.copy2(root/'target/release'/name,root/'bin'/name)
    shutil.copy2(root/'target/release'/name,dest/name)
shutil.copytree(root/'target/release/media-tools',dest/'media-tools',dirs_exist_ok=True)
shutil.copytree(root/'target/release/media-tools',root/'bin/media-tools',dirs_exist_ok=True)
shutil.copy2(root/'README.md',dest/'README.md')
archive=dest.with_suffix('.zip')
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
    for p in sorted(dest.rglob('*')):
        if p.is_file():z.write(p,dest.name+'/'+p.relative_to(dest).as_posix())
with zipfile.ZipFile(archive) as z:
    assert z.read(dest.name+'/deskbox-reconstructed.exe')==(root/'target/release/deskbox-reconstructed.exe').read_bytes()
result={'runtime_archive':str(archive),'executable_sha256':hashlib.sha256((dest/'deskbox-reconstructed.exe').read_bytes()).hexdigest(),'native_checks':checks,'cpu_gpu_percent_measured':False,'physical_lock_battery_saver_tested':False}
(root/'evidence/icon-names-radius-verification.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(result,ensure_ascii=False,indent=2))
