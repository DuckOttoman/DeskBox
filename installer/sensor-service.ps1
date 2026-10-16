param([Parameter(Mandatory=$true)][string]$HelperPath,[ValidateSet('Install','Remove')][string]$Action,[Parameter(Mandatory=$true)][string]$Hash)
$ErrorActionPreference='Stop'
try {
 if ((Get-FileHash -LiteralPath $HelperPath -Algorithm SHA256).Hash -ne $Hash) {throw 'Sensor helper verification failed'}
 $sensorArguments=if($Action -eq 'Install') {@('--install',$Hash)} else {@('--remove')}
 $sensorProcess=Start-Process -FilePath $HelperPath -ArgumentList $sensorArguments -Verb RunAs -PassThru -Wait -WindowStyle Hidden
 exit $sensorProcess.ExitCode
} catch {Write-Output $_.Exception.Message;exit 1}
