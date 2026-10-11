param([Parameter(Mandatory=$true)][string]$Stage,[Parameter(Mandatory=$true)][string]$InstallDir,[int]$NeedWebView=0)
$ErrorActionPreference='Stop'
$env:PSModulePath=Join-Path $PSHOME 'Modules'
[Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12
$ffmpegHash='2ce797a0f88d7f067180338fb227f7b1928ea727bd9a4d7a1d022f7c52af71a3'
function Test-Hash([string]$Path,[string]$Expected) {
 return (Test-Path -LiteralPath $Path) -and ((Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash -eq $Expected)
}
function Download-File([string]$Url,[string]$Path) {
 $client=New-Object Net.WebClient
 try {$client.DownloadFile($Url,$Path)} finally {$client.Dispose()}
}
try {
 New-Item -ItemType Directory -Force -Path $Stage | Out-Null
 if (Test-Path -LiteralPath (Join-Path $Stage 'download-error.txt')) {Remove-Item -LiteralPath (Join-Path $Stage 'download-error.txt')}
 $installed=Join-Path $InstallDir 'media-tools\ffmpeg.exe'
 $ffmpeg=Join-Path $Stage 'ffmpeg.exe'
 if (-not (Test-Hash $installed $ffmpegHash)) {
  if (-not (Test-Hash $ffmpeg $ffmpegHash)) {
   Download-File 'https://raw.githubusercontent.com/imageio/imageio-binaries/master/ffmpeg/ffmpeg-win-x86_64-v7.1.exe' $ffmpeg
  }
  if (-not (Test-Hash $ffmpeg $ffmpegHash)) {throw 'FFmpeg SHA256 verification failed'}
 }
 if ($NeedWebView -eq 1) {
  $bootstrap=Join-Path $Stage 'MicrosoftEdgeWebview2Setup.exe'
  Download-File 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' $bootstrap
  $signature=Get-AuthenticodeSignature -LiteralPath $bootstrap
  if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notlike '*O=Microsoft Corporation*') {throw 'Microsoft signature verification failed'}
  $runtime=Start-Process -FilePath $bootstrap -ArgumentList '/silent','/install' -PassThru -Wait -WindowStyle Hidden
  if ($runtime.ExitCode -ne 0) {throw "WebView2 installer exit: $($runtime.ExitCode)"}
 }
 exit 0
} catch {
 $_ | Out-String | Set-Content -LiteralPath (Join-Path $Stage 'download-error.txt') -Encoding UTF8
 exit 1
}
