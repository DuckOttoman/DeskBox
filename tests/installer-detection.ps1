$ErrorActionPreference='Stop'
. "$PSScriptRoot/../installer/detect-installation.ps1" -NoRun
$root=Join-Path $PSScriptRoot '../evidence/install-detection-fixtures'
New-Item -ItemType Directory -Force "$root/stale","$root/actual" | Out-Null
[IO.File]::WriteAllText("$root/actual/deskbox-reconstructed.exe",'old')
$found=Find-DeskBoxInstall @([pscustomobject]@{Path="$root/stale";Registered=$true},[pscustomobject]@{Path="$root/actual";Registered=$false})
if($found.Path -ne "$root/actual" -or $found.State -ne 'installed'){throw 'Stale registry hid actual installation'}
$broken=Find-DeskBoxInstall @([pscustomobject]@{Path="$root/stale";Registered=$true})
if($broken.State -ne 'broken'){throw 'Residual registration must be distinguished'}
if((Find-DeskBoxInstall @()).State -ne 'absent'){throw 'Fresh machine not detected'}
$hash=(Get-FileHash -LiteralPath "$root/actual/deskbox-reconstructed.exe" -Algorithm SHA256).Hash
$files=@{'deskbox-reconstructed.exe'=$hash}
if((Get-FileGroupState "$root/actual" $files) -ne 'ready'){throw 'Healthy component not reused'}
[IO.File]::WriteAllText("$root/actual/deskbox-reconstructed.exe",'damaged')
if((Get-FileGroupState "$root/actual" $files) -ne 'update'){throw 'Changed component not detected'}
if((Get-FileGroupState "$root/stale" $files) -ne 'missing'){throw 'Missing component not detected'}
if((Select-RuntimeVersion @('0.0.0.0','invalid','155.0.4283.45')) -ne '155.0.4283.45'){throw 'Runtime registry variants rejected'}
if((Select-RuntimeVersion @('0.0.0.0','invalid')) -ne ''){throw 'Invalid runtime registration accepted'}
if(-not(Test-ServiceImage @('"C:\Program Files\DeskBox Sensor\DeskBox.Sensor.exe" --service') 'C:\Program Files\DeskBox Sensor\DeskBox.Sensor.exe')){throw 'Equivalent service path rejected'}
if(Test-ServiceImage @('"C:\Program Files\DeskBox Sensor\DeskBox.Sensor.exe" --wrong') 'C:\Program Files\DeskBox Sensor\DeskBox.Sensor.exe'){throw 'Wrong service mode accepted'}
Write-Output 'INSTALL_DETECTION_POLICY_OK' 
