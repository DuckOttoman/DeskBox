param([string]$Output,[string]$InstallDir,[string]$AppKey='DeskBoxReconstructed',[switch]$NoRun)
$ErrorActionPreference='Stop'
function Find-DeskBoxInstall($Candidates) {
 foreach($candidate in $Candidates){try{if($candidate.Path -and (Test-Path -LiteralPath (Join-Path $candidate.Path 'deskbox-reconstructed.exe') -PathType Leaf -ErrorAction SilentlyContinue)){return [pscustomobject]@{Path=$candidate.Path;State='installed'}}}catch{}}
 foreach($candidate in $Candidates){if($candidate.Path -and $candidate.Registered){return [pscustomobject]@{Path=$candidate.Path;State='broken'}}}
 return [pscustomobject]@{Path='';State='absent'}
}
function Get-OwnedHash([string]$Path){
 $sha=[Security.Cryptography.SHA256]::Create();$stream=$null
 try{$stream=[IO.File]::OpenRead($Path);return [BitConverter]::ToString($sha.ComputeHash($stream)).Replace('-','')}finally{if($stream){$stream.Dispose()};$sha.Dispose()}
}
function Get-FileGroupState([string]$Directory,$Hashes){
 $missing=$false;$changed=$false
 foreach($entry in $Hashes.GetEnumerator()){
  $file=Join-Path $Directory $entry.Key
  if(-not(Test-Path -LiteralPath $file -PathType Leaf)){$missing=$true;continue}
  try{if((Get-OwnedHash $file) -ne $entry.Value){$changed=$true}}catch{$changed=$true}
 }
 if($missing){return 'missing'};if($changed){return 'update'};return 'ready'
}
function Select-RuntimeVersion($Versions){
 $valid=@(foreach($value in $Versions){$version=$null;if([Version]::TryParse([string]$value,[ref]$version) -and $version -gt [Version]'0.0.0.0'){$version}})
 if($valid.Count){return ($valid | Sort-Object -Descending | Select-Object -First 1).ToString()};return ''
}
function Test-ServiceImage($Values,[string]$ExpectedExecutable){
 foreach($value in $Values){if($value -match '^"([^"]+)"\s+--service$'){
  try{if([IO.Path]::GetFullPath($matches[1]) -eq [IO.Path]::GetFullPath($ExpectedExecutable)){return $true}}catch{}
 }};return $false
}
function Read-RegistryValues([string]$Path,[string]$Name){
 foreach($hive in @([Microsoft.Win32.RegistryHive]::CurrentUser,[Microsoft.Win32.RegistryHive]::LocalMachine)){
  foreach($view in @([Microsoft.Win32.RegistryView]::Registry64,[Microsoft.Win32.RegistryView]::Registry32)){
   $base=$null;$key=$null
   try{$base=[Microsoft.Win32.RegistryKey]::OpenBaseKey($hive,$view);$key=$base.OpenSubKey($Path);if($key){$v=$key.GetValue($Name);if($null -ne $v){[pscustomobject]@{Value=$v;Hive=$hive.ToString();View=$view.ToString()}}}}
   catch{}finally{if($key){$key.Dispose()};if($base){$base.Dispose()}}
  }
 }
}
if($NoRun){return}
try{
 $registration=@(Read-RegistryValues "Software\Microsoft\Windows\CurrentVersion\Uninstall\$AppKey" 'InstallLocation')
 $candidates=@(foreach($entry in $registration){[pscustomobject]@{Path=[Environment]::ExpandEnvironmentVariables($entry.Value);Registered=$true}})
 if($InstallDir){$candidates+= [pscustomobject]@{Path=$InstallDir;Registered=$false}}
 if($AppKey -eq 'DeskBoxReconstructed'){
  foreach($base in @([Environment]::GetFolderPath('LocalApplicationData'),$env:LOCALAPPDATA,$env:ProgramW6432,${env:ProgramFiles(x86)})){
   if(-not $base){continue}
   foreach($suffix in @('Programs\DeskBox','Programs\DeskBox Reconstructed','DeskBox','DeskBox Reconstructed')){$candidates+= [pscustomobject]@{Path=(Join-Path $base $suffix);Registered=$false}}
  }
 }
 $found=Find-DeskBoxInstall $candidates
 $manifest=Get-Content -LiteralPath (Join-Path $PSScriptRoot 'payload-hashes.json') -Raw -Encoding UTF8 | ConvertFrom-Json
 $app=@{};$sensor=@{};$media=@{};$global=@{}
 foreach($property in $manifest.psobject.Properties){
  if($property.Name.StartsWith('sensor-tools/')){$sensor[$property.Name]=$property.Value;if($property.Name -ne 'sensor-tools/dependencies.json'){$global[$property.Name.Substring(13)]=$property.Value}}
  elseif($property.Name -eq 'media-tools/ffmpeg.exe'){$media[$property.Name]=$property.Value}
  else{$app[$property.Name]=$property.Value}
 }
 $runtime=Select-RuntimeVersion @(Read-RegistryValues 'Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' 'pv' | ForEach-Object {$_.Value})
 $pawn=Select-RuntimeVersion @(Read-RegistryValues 'Software\Microsoft\Windows\CurrentVersion\Uninstall\PawnIO' 'DisplayVersion' | ForEach-Object {$_.Value})
 $target=if($InstallDir){$InstallDir}elseif($found.Path){$found.Path}else{Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'Programs\DeskBox'}
 $appState=Get-FileGroupState $target $app;$sensorState=Get-FileGroupState $target $sensor;$mediaState=Get-FileGroupState $target $media
 $protected=Join-Path ([Environment]::GetFolderPath('ProgramFiles')) 'DeskBox Sensor'
 # Installer launches this script in a 64-bit host; use machine-wide Program Files for the service.
 if($env:ProgramW6432){$protected=Join-Path $env:ProgramW6432 'DeskBox Sensor'}
 $serviceImage=@(Read-RegistryValues 'SYSTEM\CurrentControlSet\Services\DeskBoxCpuSensor' 'ImagePath' | ForEach-Object {$_.Value})
 $serviceImageValid=Test-ServiceImage $serviceImage (Join-Path $protected 'DeskBox.Sensor.exe')
 $globalState=Get-FileGroupState $protected $global
 $serviceReady=$serviceImageValid -and [bool]$pawn -and ($globalState -eq 'ready')
 $status=@{ready='已安装且与此安装包一致';missing='缺失／不完整，将补齐';update='已有其他版本或文件不同，将覆盖更新'}
 $exists=Test-Path -LiteralPath (Join-Path $target 'deskbox-reconstructed.exe') -PathType Leaf
 $mode=if($exists){'已检测到 DeskBox：覆盖更新／重新安装'}elseif($found.State -eq 'broken'){'发现残留安装记录：将补齐程序文件'}else{'未检测到 DeskBox：全新安装'}
 $lines=@('[Detection]',('ProtectedDir='+$protected),('ServiceImageValid='+$serviceImageValid),('GlobalFiles='+$globalState),('SuggestedPath='+$found.Path),('DetectedPath='+$found.Path),('DetectedState='+$found.State),('Mode='+$mode),('Target='+$target),('App='+$status[$appState]),('FFmpeg='+$status[$mediaState]),('SensorFiles='+$status[$sensorState]),('WebView2='+$(if($runtime){'已安装 '+$runtime+'，无需重复下载'}else{'未检测到有效运行时，将联网安装'})),('PawnIO='+$(if($pawn){'已安装 '+$pawn}else{'未安装，启用 CPU 温度时安装'})),('Service='+$(if($serviceReady){'已安装且组件一致，无需重复授权'}elseif($serviceImage.Count){'已有服务，组件需要更新／修复'}else{'尚未启用，可在安装时或小组件菜单中启用'})),('ServiceReady='+$(if($serviceReady){'1'}else{'0'})),('RuntimePresent='+$(if($runtime){'1'}else{'0'})))
 [IO.File]::WriteAllLines($Output,$lines,[Text.Encoding]::Unicode)
 exit 0
}catch{if($Output){[IO.File]::WriteAllLines($Output,@('[Detection]','Error='+$_.Exception.Message),[Text.Encoding]::Unicode)};exit 1}
