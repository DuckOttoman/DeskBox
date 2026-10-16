"""Package verified CPU core-temperature update including the collector."""
from pathlib import Path
import hashlib,json,zipfile
r=Path(__file__).resolve().parents[1]
log=(r/'evidence/cpu-core-temperature-final-native-4/runtime.log').read_text(encoding='utf8')
assert 'CPU_CORE_TEMPERATURE_NATIVE_SUCCESS' in log and 'FRONTEND_ERROR' not in log
assert 'Finished `release` profile' in (r/'evidence/cpu-core-temperature-final-build.log').read_text(encoding='utf8')
assert hashlib.sha256((r/'target/release/sensor-tools/DeskBox.Sensor.exe').read_bytes()).hexdigest()==(r/'sensor-helper/helper-sha256.txt').read_text().strip()
files=['deskbox-reconstructed.exe','WebView2Loader.dll']+['sensor-tools/'+p.name for p in sorted((r/'target/release/sensor-tools').iterdir()) if p.is_file()]
manifest={p:hashlib.sha256((r/'target/release'/p).read_bytes()).hexdigest() for p in files}
notes="""DeskBox CPU 核心温度更新（2026-10-15）
退出 DeskBox，将本包的程序、WebView2Loader.dll 和整个 sensor-tools 文件夹复制到安装目录并替换。保留配置、Albums 与 media-tools。
系统状态小组件右键 → CPU 温度，可选择最低核心、最高核心、同时显示。
安装版首次使用：右键 → CPU 温度 → 启用／重试温度采集，确认 Windows 管理员授权；服务安装到 Program Files，普通权限 DeskBox 后续可按需启动。若已启用旧版采集服务，请再次执行启用以更新服务组件。
便携版：同一入口单独请求采集权限，采集器同样从受保护目录运行；DeskBox 主程序无需管理员运行。采集器空闲约一分钟自动退出。
数值是当前核心读数范围，不是历史极值；不支持逐核心时明确显示不可用，封装和 ACPI 信息单独标注。PawnIO 共享组件不随 DeskBox 自动删除。
"""
p=r/'deliverables/DeskBox-Reconstructed-2026-10-15-cpu-temperature-update.zip'
with zipfile.ZipFile(p,'w',zipfile.ZIP_DEFLATED) as z:
 for f in files:z.write(r/'target/release'/f,f)
 z.writestr('更新说明.txt',notes.encode('utf8'));z.writestr('sha256.json',json.dumps(manifest,indent=2))
print(json.dumps({'path':str(p),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()},indent=2))
