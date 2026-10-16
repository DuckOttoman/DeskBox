param([switch]$Resume,[string]$InstallName='installed',[switch]$Online)
$ErrorActionPreference='Stop'
$projectRoot=Split-Path -Parent $PSScriptRoot
$evidenceRoot=Join-Path $projectRoot 'evidence\installer-test'
$installTarget=Join-Path $evidenceRoot $InstallName
$resolvedTarget=[IO.Path]::GetFullPath($installTarget)
if (-not $resolvedTarget.StartsWith([IO.Path]::GetFullPath($evidenceRoot)+'\',[StringComparison]::OrdinalIgnoreCase)) {throw 'Unsafe diagnostic install target'}
$setup=Join-Path $evidenceRoot 'DeskBox-test-setup.exe'
if ($Online) {$setup=Join-Path $evidenceRoot 'DeskBox-online-test-setup.exe'}
$key='HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\DeskBoxReconstructed.InstallerTest'
if (Test-Path -LiteralPath $key) {
 if (-not $Resume -or (Get-ItemProperty -LiteralPath $key).InstallLocation -ne $installTarget) {throw 'Diagnostic registry key already exists or points outside test target'}
}
function Install-Test {
 $process=Start-Process -FilePath $setup -ArgumentList "/S /D=$installTarget" -PassThru -Wait -WindowStyle Hidden
 if ($process.ExitCode -ne 0) {throw "Install failed: $($process.ExitCode)"}
}
Install-Test
$payload=@('deskbox-reconstructed.exe','WebView2Loader.dll','media-tools\ffmpeg.exe','media-tools\FFmpeg-LICENSE.txt','media-tools\FFmpeg-VERSION.txt')
$payload += Get-ChildItem -LiteralPath (Join-Path $projectRoot 'target/release/sensor-tools') -File | ForEach-Object { 'sensor-tools\'+$_.Name }
foreach($relative in $payload) {
 $installed=Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $installTarget $relative)
 $expected=Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $projectRoot "target\release\$relative")
 if ($installed.Hash -ne $expected.Hash) {throw "Payload mismatch: $relative"}
}
$registered=Get-ItemProperty -LiteralPath $key
if ($registered.InstallLocation -ne $installTarget) {throw 'Install registry directory mismatch'}
$shell=New-Object -ComObject WScript.Shell
$desktop=[Environment]::GetFolderPath('Desktop')
$programs=[Environment]::GetFolderPath('Programs')
$shortcut=Join-Path $desktop 'DeskBox Installer Verification.lnk'
if ($shell.CreateShortcut($shortcut).TargetPath -ne (Join-Path $installTarget 'deskbox-reconstructed.exe')) {throw 'Desktop shortcut mismatch'}
if (-not (Test-Path -LiteralPath (Join-Path $programs 'DeskBox Installer Verification\DeskBox Installer Verification.lnk'))) {throw 'Start menu shortcut missing'}
Set-Content -LiteralPath (Join-Path $installTarget 'user-keep.txt') -Value 'preserve unrelated user file'
$locked=[IO.File]::Open((Join-Path $installTarget 'deskbox-reconstructed.exe'),[IO.FileMode]::Open,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
try {
 $blocked=Start-Process -FilePath $setup -ArgumentList "/S /D=$installTarget" -PassThru -Wait -WindowStyle Hidden
 if ($blocked.ExitCode -eq 0) {throw 'Locked application overwrite was not rejected'}
} finally {$locked.Dispose()}
Install-Test
if (-not (Test-Path -LiteralPath (Join-Path $installTarget 'user-keep.txt'))) {throw 'Upgrade removed unrelated file'}
$data=Join-Path $evidenceRoot ($(if ($Online) {'runtime-online-'} else {'runtime-final-'})+$InstallName+'-'+(Get-Date -Format 'HHmmss'))
New-Item -ItemType Directory -Force (Join-Path $data 'rename-fixtures') | Out-Null
Copy-Item -LiteralPath (Join-Path $projectRoot 'evidence\media-native\rename-fixtures\media-test.mp4'),(Join-Path $projectRoot 'evidence\media-native\rename-fixtures\media-test.gif') -Destination (Join-Path $data 'rename-fixtures')
$env:DESKBOX_DATA_DIR=$data
$env:DESKBOX_MERGE_CHECK_SCRIPT=Join-Path $projectRoot 'installer\media-webview-check.js'
$app=Start-Process -FilePath (Join-Path $installTarget 'deskbox-reconstructed.exe') -ArgumentList '--fence-merge-check','--drag-session' -PassThru -WindowStyle Hidden
Remove-Item Env:DESKBOX_DATA_DIR,Env:DESKBOX_MERGE_CHECK_SCRIPT
if (-not $app.WaitForExit(55000)) {Stop-Process -Id $app.Id -Force; throw 'Installed application diagnostic timed out'}
$log=Get-Content -LiteralPath (Join-Path $data 'runtime.log') -Raw -Encoding UTF8
if ($log -notmatch 'ICON_RADIUS_NATIVE_SUCCESS' -or $log -match 'FRONTEND_ERROR') {throw 'Installed application media/UI checks failed'}
$runKey='HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$runName='DeskBoxInstallerVerificationAutostart'
if (Get-ItemProperty -LiteralPath $runKey -Name $runName -ErrorAction SilentlyContinue) {throw 'Test autorun entry already exists'}
New-ItemProperty -LiteralPath $runKey -Name $runName -Value ((Join-Path $installTarget 'deskbox-reconstructed.exe')+' --autostart') -PropertyType String | Out-Null
$remove=Start-Process -FilePath (Join-Path $installTarget 'uninstall.exe') -ArgumentList '/S' -PassThru -Wait -WindowStyle Hidden
if ($remove.ExitCode -ne 0) {throw "Uninstall failed: $($remove.ExitCode)"}
$uninstallDeadline=[DateTime]::UtcNow.AddSeconds(15)
while ((Test-Path -LiteralPath $key) -and [DateTime]::UtcNow -lt $uninstallDeadline) {Start-Sleep -Milliseconds 100}
foreach($relative in $payload) {if (Test-Path -LiteralPath (Join-Path $installTarget $relative)) {throw "Uninstall retained owned payload: $relative"}}
if (Test-Path -LiteralPath $key) {throw 'Uninstall registry key remains'}
if (Test-Path -LiteralPath $shortcut) {throw 'Desktop shortcut remains'}
if (Get-ItemProperty -LiteralPath $runKey -Name $runName -ErrorAction SilentlyContinue) {throw 'Owned autorun entry remains'}
if (Test-Path -LiteralPath (Join-Path $installTarget 'uninstall.exe')) {throw 'Uninstaller remains'}
if (-not (Test-Path -LiteralPath (Join-Path $installTarget 'user-keep.txt'))) {throw 'Uninstall removed unrelated file'}
if (-not (Test-Path -LiteralPath (Join-Path $data 'config.json'))) {throw 'Uninstall removed app data'}
$result=@{install_directory=$installTarget;install=$true;payload_hashes=$true;shortcuts=$true;registry=$true;locked_upgrade_rejected=$true;upgrade_preserves_user_file=$true;installed_media_ui=$true;uninstall=$true;autorun_cleanup=$true;user_data_preserved=$true;missing_runtime_clean_vm_tested=$false}
$result | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $projectRoot $(if ($Online) {'evidence\online-installer-verification.json'} else {'evidence\installer-verification.json'})) -Encoding UTF8
Write-Output 'INSTALLER_TEST_OK: install / upgrade / installed video and UI / uninstall / user data preserved'
