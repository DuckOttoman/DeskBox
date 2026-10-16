param([string]$Name='plugins-native')
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
$dir=Join-Path $root ('evidence/'+$Name+'-'+(Get-Date -Format 'HHmmss'))
New-Item -ItemType Directory -Force $dir | Out-Null
$env:DESKBOX_DATA_DIR=$dir
$env:DESKBOX_MERGE_CHECK_SCRIPT=Join-Path $root 'tests/plugin-native-webview.js'
$p=$null
try {
 $p=Start-Process (Join-Path $root 'target/release/deskbox-reconstructed.exe') -ArgumentList '--fence-merge-check','--drag-session' -WindowStyle Hidden -RedirectStandardError (Join-Path $dir 'stderr.log') -RedirectStandardOutput (Join-Path $dir 'stdout.log') -PassThru
 if(-not $p.WaitForExit(90000)){throw 'Plugin native diagnostic timed out'}
 $log=Get-Content (Join-Path $dir 'runtime.log') -Raw -Encoding UTF8
 if($log -match 'FRONTEND_ERROR' -or $log -notmatch 'PLUGIN_NATIVE_SUCCESS'){throw $log}
 Write-Output ('PASS '+$dir)
} finally {
 Remove-Item Env:DESKBOX_DATA_DIR,Env:DESKBOX_MERGE_CHECK_SCRIPT -ErrorAction SilentlyContinue
 if($p -and -not $p.HasExited){Stop-Process -Id $p.Id -Force}
}
