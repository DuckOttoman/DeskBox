"""Build an offline current-user NSIS installer from the verified release payload."""
import argparse,hashlib,json,os,subprocess,urllib.request,zipfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
NSIS_URL='https://github.com/tauri-apps/binary-releases/releases/download/nsis-3/nsis-3.zip'
WEBVIEW_URL='https://go.microsoft.com/fwlink/?linkid=2124701'
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def download(url,path):
    if path.exists():return
    path.parent.mkdir(parents=True,exist_ok=True)
    opener=urllib.request.build_opener(urllib.request.ProxyHandler({}))
    temporary=path.with_suffix(path.suffix+'.partial')
    with opener.open(url,timeout=120) as response,temporary.open('wb') as out:
        while block:=response.read(1024*1024):out.write(block)
    temporary.replace(path)
def build(test=False,online=False):
    tools=ROOT/'.tools';compiler=tools/'nsis/nsis-3.08/makensis.exe'
    archive=tools/'downloads/nsis-3.zip';download(NSIS_URL,archive)
    if not compiler.exists():
        folder=tools/'nsis';folder.mkdir(exist_ok=True)
        with zipfile.ZipFile(archive) as z:
            for info in z.infolist():
                if not (folder/info.filename).resolve().is_relative_to(folder.resolve()):raise ValueError('Invalid NSIS archive path')
            z.extractall(folder)
    runtime=tools/'downloads/MicrosoftEdgeWebView2RuntimeInstallerX64.exe'
    if not online:download(WEBVIEW_URL,runtime)
    env=os.environ.copy();env['DESKBOX_WEBVIEW_INSTALLER']=str(runtime)
    env['PSModulePath']=os.path.join(os.environ['SystemRoot'],'System32','WindowsPowerShell','v1.0','Modules')
    command='$s=Get-AuthenticodeSignature -LiteralPath $env:DESKBOX_WEBVIEW_INSTALLER; $s.Status.ToString(); $s.SignerCertificate.Subject; if ($s.Status -ne "Valid" -or $s.SignerCertificate.Subject -notlike "*O=Microsoft Corporation*") {exit 1}'
    signed=subprocess.CompletedProcess([],0,'Verified by installer at download time','') if online else subprocess.run(['powershell','-NoProfile','-Command',command],env=env,capture_output=True,text=True)
    if signed.returncode:raise RuntimeError('Microsoft runtime signature verification failed: '+signed.stdout+signed.stderr)
    output=ROOT/('evidence/installer-test/DeskBox-online-test-setup.exe' if test and online else 'evidence/installer-test/DeskBox-test-setup.exe' if test else 'deliverables/DeskBox-0.2.0-windows-x64-online-setup.exe' if online else 'deliverables/DeskBox-0.2.0-windows-x64-setup.exe')
    output.parent.mkdir(parents=True,exist_ok=True)
    detect_files=['deskbox-reconstructed.exe','WebView2Loader.dll','media-tools/ffmpeg.exe']+['sensor-tools/'+p.name for p in sorted((ROOT/'target/release/sensor-tools').iterdir()) if p.is_file()]
    (ROOT/'installer/payload-hashes.json').write_text(json.dumps({p:digest(ROOT/'target/release'/p) for p in detect_files},indent=2),encoding='utf8')
    args=[str(compiler),'/V3','/INPUTCHARSET','UTF8',f'/DROOT={ROOT}',f'/DWEBVIEW={runtime.parent}',f'/DOUTPUT={output}']
    if online:args+=['/DONLINE']
    if test:args+=['/DAPP_KEY=DeskBoxReconstructed.InstallerTest','/DAPP_NAME=DeskBox Installer Verification','/DAUTOSTART_NAME=DeskBoxInstallerVerificationAutostart']
    args+=[str(ROOT/'installer/deskbox.nsi')]
    log=ROOT/('evidence/online-installer-test-build.log' if test and online else 'evidence/online-installer-build.log' if online else 'evidence/installer-test-build.log' if test else 'evidence/installer-build.log')
    with log.open('w',encoding='utf-8') as f:
        result=subprocess.run(args,cwd=ROOT,stdout=f,stderr=subprocess.STDOUT)
    if result.returncode:raise RuntimeError(f'NSIS build failed; see {log}')
    payload=['deskbox-reconstructed.exe','WebView2Loader.dll','media-tools/ffmpeg.exe','media-tools/FFmpeg-LICENSE.txt','media-tools/FFmpeg-VERSION.txt']
    payload += ['sensor-tools/'+p.name for p in sorted((ROOT/'target/release/sensor-tools').iterdir()) if p.is_file()]
    manifest={'installer':str(output),'installer_sha256':digest(output),'bytes':output.stat().st_size,'target':'Windows 10/11 x64','scope':'current user','signed_setup':False,'mode':'online' if online else 'offline','webview_url':'https://go.microsoft.com/fwlink/p/?LinkId=2124703' if online else WEBVIEW_URL,'webview_sha256':None if online else digest(runtime),'webview_publisher':signed.stdout.strip(),'nsis_url':NSIS_URL,'nsis_archive_sha256':digest(archive),'payload':{p:digest(ROOT/'target/release'/p) for p in payload}}
    (output.parent/(output.stem+'-manifest.json')).write_text(json.dumps(manifest,ensure_ascii=False,indent=2),encoding='utf-8')
    print(json.dumps(manifest,ensure_ascii=False,indent=2),flush=True)
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--test',action='store_true');parser.add_argument('--online',action='store_true');options=parser.parse_args();build(options.test,options.online)
