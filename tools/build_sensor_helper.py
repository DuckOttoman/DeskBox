"""Build pinned CPU collector using official release dependencies and Windows .NET compiler."""
from pathlib import Path
import urllib.request,json,zipfile,hashlib,subprocess,shutil,os
r=Path(__file__).resolve().parents[1];d=r/'.tools/downloads';d.mkdir(exist_ok=True)
o=urllib.request.build_opener(urllib.request.ProxyHandler({}))
def fetch(url,p):
 if not p.exists():p.write_bytes(o.open(url,timeout=90).read())
 return p
archive=fetch('https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/releases/download/v0.9.6/LibreHardwareMonitor.zip',d/'LibreHardwareMonitor-v0.9.6.zip')
assert hashlib.sha256(archive.read_bytes()).hexdigest()=='086d9f1b5a99e643edc2cfaaac16051685b551e4c5ac0b32a57c58c0e529c001','LHM official release digest mismatch'
out=r/'target/release/sensor-tools';out.mkdir(parents=True,exist_ok=True)
with zipfile.ZipFile(archive) as z:
 for n in z.namelist():
  if '/' not in n and n.endswith('.dll'):(out/n).write_bytes(z.read(n))
for n in ['LICENSE','THIRD-PARTY-NOTICES.txt']:
 p=fetch('https://raw.githubusercontent.com/LibreHardwareMonitor/LibreHardwareMonitor/v0.9.6/'+n,d/('LHM-'+n));shutil.copyfile(p,out/('LHM-'+n))
pawn=fetch('https://github.com/namazso/PawnIO.Setup/releases/download/2.2.0/PawnIO_setup.exe',d/'PawnIO-2.2.0-setup.exe')
assert hashlib.sha256(pawn.read_bytes()).hexdigest()=='1f519a22e47187f70a1379a48ca604981c4fcf694f4e65b734aaa74a9fba3032','PawnIO official release digest mismatch'
shutil.copyfile(pawn,out/'PawnIO_setup.exe')
(out/'PawnIO-NOTICE.txt').write_text('PawnIO 2.2.0 official unmodified installer. Publisher: namazso. Source: https://github.com/namazso/PawnIO.Setup/releases/tag/2.2.0 . Installation terms are provided by the official installer. Shared driver is not removed by DeskBox.',encoding='utf8')
deps={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in out.iterdir() if p.is_file() and p.name not in ['DeskBox.Sensor.exe','dependencies.json']}
rows=','.join('{"'+n+'","'+h+'"}' for n,h in deps.items())
(r/'sensor-helper/Dependencies.cs').write_text('using System.Collections.Generic; static class Dependencies {internal static readonly Dictionary<string,string> Hashes=new Dictionary<string,string>{'+rows+'};}',encoding='utf8')
compiler=Path(os.environ['SystemRoot'])/'Microsoft.NET/Framework64/v4.0.30319/csc.exe'
args=[str(compiler),'/nologo','/target:winexe','/platform:x64','/optimize+','/out:'+str(out/'DeskBox.Sensor.exe'),'/r:System.Windows.Forms.dll','/r:System.ServiceProcess.dll','/r:System.Web.Extensions.dll','/r:'+str(out/'LibreHardwareMonitorLib.dll'),str(r/'sensor-helper/TemperaturePolicy.cs'),str(r/'sensor-helper/Program.cs'),str(r/'sensor-helper/Dependencies.cs')]
subprocess.run(args,cwd=r,check=True)
(r/'sensor-helper/helper-sha256.txt').write_text(hashlib.sha256((out/'DeskBox.Sensor.exe').read_bytes()).hexdigest(),encoding='utf8')
(r/'installer/sensor-hash.nsh').write_text('!define SENSOR_HELPER_SHA256 \"'+hashlib.sha256((out/'DeskBox.Sensor.exe').read_bytes()).hexdigest()+'\"\n',encoding='utf8')
manifest={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in out.iterdir() if p.is_file()}
(out/'dependencies.json').write_text(json.dumps({'lhm':'0.9.6','pawnio':'2.2.0','files':manifest},indent=2),encoding='utf8')
(r/'installer/sensor-delete.nsh').write_text('\n'.join('Delete \"$INSTDIR\\sensor-tools\\'+p.name+'\"' for p in out.iterdir() if p.is_file())+'\nRMDir \"$INSTDIR\\sensor-tools\"\n',encoding='utf8')
print(json.dumps({'directory':str(out),'files':len(manifest),'lhm_digest':hashlib.sha256(archive.read_bytes()).hexdigest(),'pawnio_digest':hashlib.sha256(pawn.read_bytes()).hexdigest()},indent=2))
