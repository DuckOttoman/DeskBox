"""Create a replace-in-place update bundle; never include user config or albums."""
from pathlib import Path
import hashlib,json,zipfile
import argparse
parser=argparse.ArgumentParser();parser.add_argument('--layout-backup',action='store_true');parser.add_argument('--motion',action='store_true');args=parser.parse_args()
root=Path(__file__).resolve().parents[1]
if args.motion:
 for role,marker in [('ball','BALL_MOTION_NATIVE_SUCCESS'),('timer','TIMER_MOTION_NATIVE_SUCCESS')]:
  log=(root/f'evidence/{role}-motion-final/runtime.log').read_text(encoding='utf8')
  assert marker in log and 'FRONTEND_ERROR' not in log
 assert 'Finished `release` profile' in (root/'evidence/widget-ball-motion-final-build.log').read_text(encoding='utf8')
elif args.layout_backup:
 log=(root/'evidence/layout-backup-final-native/runtime.log').read_text(encoding='utf8')
 assert 'LAYOUT_BACKUP_NATIVE_SUCCESS' in log and 'LAYOUT_BACKUP_RESTART_SUCCESS' in log and 'FRONTEND_ERROR' not in log
 assert 'Finished `release` profile' in (root/'evidence/layout-backup-final-build.log').read_text(encoding='utf8')
else:
 log=(root/'evidence/thermal-widget/runtime.log').read_text(encoding='utf8')
 assert 'THERMAL_WIDGET_RENDERED' in log and 'FRONTEND_ERROR' not in log
 samples=[json.loads(line.split('sysmon ',1)[1]) for line in log.splitlines() if line.startswith('sysmon ')]
 assert any(s.get('cpuTemp') is not None for s in samples)
 assert 'Finished `release` profile' in (root/'evidence/thermal-final-build.log').read_text(encoding='utf8')
files=['deskbox-reconstructed.exe','WebView2Loader.dll']
manifest={name:hashlib.sha256((root/'target/release'/name).read_bytes()).hexdigest() for name in files}
notes=r"""DeskBox 替换更新包（2026-10-11）
1. 通过悬浮球菜单退出 DeskBox，确认程序已经关闭。
2. 将本包的 deskbox-reconstructed.exe 和 WebView2Loader.dll 复制到现有安装目录，替换同名文件。
   默认目录：%LOCALAPPDATA%\Programs\DeskBox Reconstructed
3. 保留现有 media-tools、Albums 文件夹和配置文件，然后运行程序。
本包用于更新已经安装了重建版的电脑。首次安装请使用联网安装程序。
新增 CPU 温度采集使用原版所用的 Windows 热区计数器。读数取决于主板固件，不一定代表 CPU 核心温度；不支持时显示未提供。
"""
if args.layout_backup:notes+='\n此版新增布局备份/还原管理：每项显示备份时间，可还原、重命名、删除。备份保存在应用用户数据目录。\n'
if args.motion:notes+='\n此版增加番茄钟完成扩散圆环与轻微弹动、悬浮球闲置呼吸以及菜单展开/收回；遵循系统减少动态效果设置。\n'
output=root/('deliverables/DeskBox-Reconstructed-2026-10-11-motion-update.zip' if args.motion else 'deliverables/DeskBox-Reconstructed-2026-10-11-layout-backup-update.zip' if args.layout_backup else 'deliverables/DeskBox-Reconstructed-2026-10-11-update.zip')
with zipfile.ZipFile(output,'w',zipfile.ZIP_DEFLATED) as z:
 for name in files:z.write(root/'target/release'/name,name)
 z.writestr('更新说明.txt',notes.encode('utf8'))
 z.writestr('sha256.json',json.dumps(manifest,indent=2))
print(json.dumps({'update':str(output),'sha256':hashlib.sha256(output.read_bytes()).hexdigest(),'files':manifest},indent=2))
