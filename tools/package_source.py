"""Package buildable sources without local toolchains or personal runtime data."""
import hashlib
import json
from pathlib import Path
import zipfile

root = Path(__file__).resolve().parents[1]
output = root / 'deliverables'
output.mkdir(exist_ok=True)
files = [root / name for name in ['Cargo.toml', 'Cargo.lock', 'build.rs', 'tauri.conf.json', 'README.md', 'README.en.md', 'LICENSE', 'build.ps1', '.gitignore', '.gitattributes']]
for directory in ['src', 'dist', 'frontend', 'ui', 'docs', 'icons', 'tests', 'capabilities', 'tools', 'installer']:
    files.extend(p for p in (root / directory).rglob('*') if p.is_file() and '__pycache__' not in p.parts)
for name in ['assets.json', 'modified-assets.json', 'classification-rules.json', 'commands.json', 'contract.md', 'verification.json', 'rename-review.md', 'native-fidelity-review.md', 'native-fidelity-pixel-comparison.json', 'parity-audit.md', 'context-menu-analysis.md', 'context-menu-review.md', 'fence-widget-interaction-analysis.md', 'fence-widget-interaction-review.md', 'settings-window-analysis.md', 'settings-window-review.md', 'rust-channel.toml']:
    path = root / 'evidence' / name
    if path.exists():
        files.append(path)
archive = output / 'deskbox-reconstructed-source.zip'
files.extend(root/'evidence'/name for name in ['layout-backup-review.md','layout-backup-verification.json','thermal-review.md','hidden-console-review.md','group-drag-review.md','group-drag-verification.json','online-installer-review.md','online-installer-verification.json','widgets-verification.json','widgets-review.md','widgets-weather-endpoints.json','weather-memory-review.md','weather-memory-verification.json'] if (root/'evidence'/name).exists())
files.append(root / 'evidence/learning-drag-monitor-analysis.md')
for name in ['fence-merge-analysis.md','fence-merge-verification.json','drag-fix-analysis.md','drag-fix-verification.json','merge-animation-fix-analysis.md','merge-animation-verification.json','media-backend-report.md','media-background-verification.json','media-background-review.md','icon-names-radius-verification.json','icon-names-radius-review.md','installer-verification.json','installer-payload-imports.json','installer-review.md']:
    path = root / 'evidence' / name
    if path.exists(): files.append(path)
with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as package:
    for path in sorted(files):
        package.write(path, 'deskbox-reconstructed/' + path.relative_to(root).as_posix())
print(json.dumps({'archive': str(archive), 'files': len(files), 'bytes': archive.stat().st_size,
                  'sha256': hashlib.sha256(archive.read_bytes()).hexdigest()}, indent=2))
