"""Package widgets build only after native/restart verification."""
import hashlib,json,shutil,zipfile
from pathlib import Path
root=Path(__file__).resolve().parents[1]
log=(root/'evidence/weather-memory-regression-verified/runtime.log').read_text(encoding='utf8')
for marker in ['WIDGETS_NATIVE_SUCCESS','WIDGETS_RESTART_SUCCESS','widget_real_weather_webview_fetch']:
    assert marker in log,marker
assert 'FRONTEND_ERROR' not in log
assert 'Finished `release` profile' in (root/'evidence/weather-memory-delivery-build.log').read_text(encoding='utf8')
folder=root/'deliverables/deskbox-reconstructed-windows-x64-widgets';folder.mkdir(exist_ok=True)
for name in ['deskbox-reconstructed.exe','WebView2Loader.dll']:
    shutil.copy2(root/'target/release'/name,folder/name)
shutil.copytree(root/'target/release/media-tools',folder/'media-tools',dirs_exist_ok=True)
shutil.copy2(root/'README.md',folder/'README.md');shutil.copy2(root/'docs/widgets.md',folder/'小组件使用说明.md')
archive=folder.with_suffix('.zip')
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
    for p in folder.rglob('*'):
        if p.is_file():z.write(p,folder.name+'/'+p.relative_to(folder).as_posix())
result={'executable_sha256':hashlib.sha256((folder/'deskbox-reconstructed.exe').read_bytes()).hexdigest(),'portable_archive':str(archive),'native_widgets':True,'restart_persistence':True,'live_weather_webview':True,'physical_multiple_dpi_displays_tested':False,'default_album_directory':'executable-parent/Albums','album_source_files_modified':False}
(root/'evidence/widgets-verification.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf8')
print(json.dumps(result,ensure_ascii=False,indent=2))
