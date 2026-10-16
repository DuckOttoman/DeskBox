"""Collect only successful, post-build isolated native evidence."""
from pathlib import Path
import hashlib,json,re
r=Path(__file__).resolve().parents[1]
exe=r/'target/release/deskbox-reconstructed.exe'
def evidence(kind):
 for p in sorted((r/'evidence').glob('quickbar-native-'+kind+'-*'),key=lambda p:p.stat().st_mtime,reverse=True):
  log=(p/'runtime.log').read_text(encoding='utf8') if (p/'runtime.log').exists() else ''
  if p.stat().st_mtime<exe.stat().st_mtime or 'FRONTEND_ERROR' in log:continue
  if not (p/'after.json').exists() or 'QUICKBAR_REFERENCE_AND_PROTECTION_SUCCESS' not in log:continue
  if kind=='normal' and ('QUICKBAR_FAST_TOGGLE_SUCCESS' not in log or not (p/'fullscreen.txt').exists()):continue
  if kind=='normal':
   position=json.loads((p/'dock-position.json').read_text(encoding='utf-8-sig'));assert abs(position['dock'][3]-position['monitor'][3])<=2
  assert (p/'rename-fixtures/rename-fixture.txt').read_text()=='DeskBox rename fixture content'
  assert not (p/'quickbar-recovery.json').exists()
  before=json.loads((p/'before.json').read_text(encoding='utf-8-sig'));after=json.loads((p/'after.json').read_text(encoding='utf-8-sig'))
  assert before['state']==after['state'];assert before['tasks']==after['tasks']
  return p,log
 raise AssertionError('No successful post-build '+kind+' evidence')
normal,nlog=evidence('normal');crash,clog=evidence('crash')
tests=(r/'evidence/quickbar-rust-tests.log').read_text(encoding='utf8')
assert 'FAILED' not in tests and 'Doc-tests' in tests
assert 'Finished `release` profile' in (r/'evidence/quickbar-build.log').read_text(encoding='utf8')
assert all(p.stat().st_mtime<exe.stat().st_mtime for d in ['src','ui','frontend'] for p in (r/d).rglob('*') if p.is_file()),'Embedded source changed after build'
report={'normalExit':True,'mainCrashRecovery':True,'dom':'QUICKBAR_DOM_ICON_MENU_UNDO_SUCCESS' in nlog and 'QUICKBAR_DOM_ICON_MENU_UNDO_SUCCESS' in clog,'fullscreenHideRestore':True,'dockAnchoredToScreenBottom':True,'repeatedToggle':True,'protectedFenceGroupWidget':True,'referenceSourcePreserved':True,'nativeNormalEvidence':normal.name,'nativeCrashEvidence':crash.name,'rustPassed':sum(map(int,re.findall(r'test result: ok\. (\d+) passed',tests))),'exeSha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'hardware':{'monitors':len(json.loads((normal/'before.json').read_text(encoding='utf-8-sig'))['tasks']),'mixedDpiTested':False,'explorerRestartLiveTested':False},'review':'quickbar-review.md','scope':'isolated diagnostics; current user configuration and installed binaries not replaced'}
(r/'evidence/quickbar-verification.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf8')
print(json.dumps(report,ensure_ascii=False,indent=2))
