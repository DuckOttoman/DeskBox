$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
$folder=Join-Path $root 'evidence\online-download-failure'
New-Item -ItemType Directory -Force $folder | Out-Null
$source=Get-Content -LiteralPath (Join-Path $root 'installer\download-dependencies.ps1') -Raw
# Inject a successful but corrupt transport response; all production verification stays intact.
$source=$source.Replace('$client.DownloadFile($Url,$Path)', '[IO.File]::WriteAllText($Path,"corrupt downloaded content")')
$script=Join-Path $folder 'corrupt-transport.ps1'
Set-Content -LiteralPath $script -Value $source -Encoding UTF8
$process=Start-Process -FilePath 'powershell.exe' -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File',('"'+$script+'"'),'-Stage',('"'+$folder+'"'),'-InstallDir',('"'+(Join-Path $folder 'untouched')+'"')) -PassThru -Wait -WindowStyle Hidden
if ($process.ExitCode -ne 1) {throw 'Corrupt download accepted'}
if ((Get-Content -LiteralPath (Join-Path $folder 'download-error.txt') -Raw) -notmatch 'FFmpeg SHA256 verification failed') {throw 'Expected hash failure absent'}
if (Test-Path -LiteralPath (Join-Path $folder 'untouched')) {throw 'Failure modified install directory'}
Write-Output 'ONLINE_DOWNLOAD_FAILURE_OK: corrupt payload rejected before install directory writes'
