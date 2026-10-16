Unicode true
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!include "WinVer.nsh"
!include "${ROOT}\installer\sensor-hash.nsh"
!include "FileFunc.nsh"
!include "nsDialogs.nsh"
!ifndef APP_NAME
!define APP_NAME "DeskBox"
!endif
!ifndef APP_KEY
!define APP_KEY "DeskBoxReconstructed"
!endif
!ifndef AUTOSTART_NAME
!define AUTOSTART_NAME "DeskBox Reconstructed"
!endif
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_KEY}"
!define WEBVIEW_KEY "Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
Name "${APP_NAME}"
OutFile "${OUTPUT}"
InstallDir "$LOCALAPPDATA\Programs\DeskBox"
InstallDirRegKey HKCU "${UNINSTALL_KEY}" "InstallLocation"
RequestExecutionLevel user
SetCompressor /SOLID zlib
VIProductVersion "0.2.0.0"
VIAddVersionKey /LANG=2052 "ProductName" "${APP_NAME}"
VIAddVersionKey /LANG=2052 "FileDescription" "DeskBox Windows x64 安装程序"
VIAddVersionKey /LANG=2052 "FileVersion" "0.2.0"
VIAddVersionKey /LANG=2052 "LegalCopyright" "DeskBox"
!define MUI_ICON "${ROOT}\icons\icon.ico"
!define MUI_UNICON "${ROOT}\icons\icon.ico"
!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_RUN "$INSTDIR\deskbox-reconstructed.exe"
!define MUI_FINISHPAGE_RUN_NOTCHECKED
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
Page custom DetectionPage
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH
!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "English"
Var RuntimePresent
Var DetectShell
Var DetectionDialog
Var DetectionResult
Var ManualDirectory

Function .onInit
 SetShellVarContext current
 ${IfNot} ${RunningX64}
  MessageBox MB_ICONSTOP "需要64位 Windows 10/11。 / Windows 10/11 x64 required." /SD IDOK
  SetErrorLevel 1
  Abort
 ${EndIf}
 ${IfNot} ${AtLeastWin10}
  MessageBox MB_ICONSTOP "需要 Windows 10 或更高版本。 / Windows 10 or newer required." /SD IDOK
  SetErrorLevel 1
  Abort
 ${EndIf}
 SetRegView 64
 StrCpy $ManualDirectory 0
 System::Call 'kernel32::GetCommandLineW() w.r1'
 ClearErrors
 ${GetOptions} $1 "/D=" $2
 ${IfNot} ${Errors}
  StrCpy $ManualDirectory 1
 ${EndIf}
 Call RefreshDetection
 ${If} $ManualDirectory == 0
  ReadINIStr $0 "$PLUGINSDIR\detected.ini" "Detection" "SuggestedPath"
  ${If} $0 != ""
   StrCpy $INSTDIR $0
  ${EndIf}
 ${EndIf}
 ; Read-only diagnostic entry point; no application files or registry entries are modified.
 ${GetParameters} $1
 ClearErrors
 ${GetOptions} $1 "/DETECT=" $2
 ${IfNot} ${Errors}
  Call RefreshDetection
  ClearErrors
  CopyFiles /SILENT "$PLUGINSDIR\detected.ini" "$2"
  ${If} ${Errors}
   SetErrorLevel 1
   Quit
  ${EndIf}
  SetErrorLevel 0
  Quit
 ${EndIf}
FunctionEnd

Function RefreshDetection
 InitPluginsDir
 SetOutPath "$PLUGINSDIR"
 File "${ROOT}\installer\detect-installation.ps1"
 File "${ROOT}\installer\payload-hashes.json"
 StrCpy $DetectShell "$WINDIR\Sysnative\WindowsPowerShell\v1.0\powershell.exe"
 IfFileExists "$DetectShell" +2 0
 StrCpy $DetectShell "$SYSDIR\WindowsPowerShell\v1.0\powershell.exe"
 nsExec::Exec '"$DetectShell" -WindowStyle Hidden -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\detect-installation.ps1" -Output "$PLUGINSDIR\detected.ini" -InstallDir "$INSTDIR" -AppKey "${APP_KEY}"'
 Pop $DetectionResult
 ${If} $DetectionResult != 0
  ReadINIStr $0 "$PLUGINSDIR\detected.ini" "Detection" "Error"
  MessageBox MB_ICONSTOP "安装检测失败，请重试。 / Installation detection failed. $0" /SD IDOK
  SetErrorLevel 1
  Abort
 ${EndIf}
 ReadINIStr $RuntimePresent "$PLUGINSDIR\detected.ini" "Detection" "RuntimePresent"
FunctionEnd

Function DetectWebView2
 Call RefreshDetection
FunctionEnd

Function DetectionPage
 Call RefreshDetection
 !insertmacro MUI_HEADER_TEXT "安装与组件检测" "确认现有安装、更新目标与依赖状态"
 nsDialogs::Create 1018
 Pop $DetectionDialog
 ${If} $DetectionDialog == error
  Abort
 ${EndIf}
 ReadINIStr $0 "$PLUGINSDIR\detected.ini" "Detection" "Mode"
 ${NSD_CreateLabel} 0 0 100% 18u "$0"
 Pop $0
 ReadINIStr $0 "$PLUGINSDIR\detected.ini" "Detection" "DetectedPath"
 ${NSD_CreateLabel} 0 20u 100% 24u "检测到的安装：$0"
 Pop $0
 ${NSD_CreateLabel} 0 45u 100% 24u "本次安装目录：$INSTDIR"
 Pop $0
 ReadINIStr $0 "$PLUGINSDIR\detected.ini" "Detection" "App"
 ${NSD_CreateLabel} 0 67u 100% 14u "DeskBox：$0"
 Pop $0
 ReadINIStr $0 "$PLUGINSDIR\detected.ini" "Detection" "WebView2"
 ${NSD_CreateLabel} 0 83u 100% 14u "WebView2：$0"
 Pop $0
 ReadINIStr $0 "$PLUGINSDIR\detected.ini" "Detection" "FFmpeg"
 ${NSD_CreateLabel} 0 99u 100% 14u "FFmpeg：$0"
 Pop $0
 ReadINIStr $0 "$PLUGINSDIR\detected.ini" "Detection" "SensorFiles"
 ${NSD_CreateLabel} 0 115u 100% 14u "CPU 采集文件：$0"
 Pop $0
 ReadINIStr $0 "$PLUGINSDIR\detected.ini" "Detection" "PawnIO"
 ${NSD_CreateLabel} 0 131u 100% 14u "PawnIO：$0"
 Pop $0
 ReadINIStr $0 "$PLUGINSDIR\detected.ini" "Detection" "Service"
 ${NSD_CreateLabel} 0 147u 100% 14u "温度服务：$0"
 Pop $0
 nsDialogs::Show
FunctionEnd

!macro CheckExecutableUnlocked PREFIX RELATIVE
 ${PREFIX}_retry:
 IfFileExists "$INSTDIR\${RELATIVE}" 0 ${PREFIX}_done
 System::Call 'kernel32::CreateFileW(w "$INSTDIR\${RELATIVE}", i 0x40000000, i 0, p 0, i 3, i 0, p 0) p.r0'
 ${If} $0 == -1
  MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "请先退出 DeskBox，然后重试。 / Close DeskBox before continuing." /SD IDCANCEL IDRETRY ${PREFIX}_retry
  SetErrorLevel 1
  Abort
 ${EndIf}
 System::Call 'kernel32::CloseHandle(p r0)'
 ${PREFIX}_done:
!macroend

Section "Install"
 !insertmacro CheckExecutableUnlocked "app" "deskbox-reconstructed.exe"
 !insertmacro CheckExecutableUnlocked "media" "media-tools\ffmpeg.exe"
 Call DetectWebView2
 !ifdef ONLINE
  InitPluginsDir
  SetOutPath "$PLUGINSDIR"
  File "${ROOT}\installer\download-dependencies.ps1"
  download_retry:
  DetailPrint "正在检查依赖，仅在缺失或不一致时下载 / Checking dependencies; downloading only when needed..."
  IntOp $2 1 - $RuntimePresent
  StrCpy $1 1
  ClearErrors
  nsExec::Exec '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -WindowStyle Hidden -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\download-dependencies.ps1" -Stage "$PLUGINSDIR" -InstallDir "$INSTDIR" -NeedWebView $2'
  Pop $1
  ${If} ${Errors}
   StrCpy $1 1
  ${EndIf}
  ${If} $1 != 0
   CopyFiles /SILENT "$PLUGINSDIR\download-error.txt" "$TEMP\${APP_KEY}-download-error.txt"
   MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "组件下载或校验失败。请检查网络后重试。错误日志：$TEMP\${APP_KEY}-download-error.txt / Dependency download or verification failed; check your connection." /SD IDCANCEL IDRETRY download_retry
   SetErrorLevel 1
   Abort
  ${EndIf}
  Call DetectWebView2
  ${If} $RuntimePresent == 0
   SetErrorLevel 1
   Abort "WebView2 安装未完成 / WebView2 installation incomplete."
  ${EndIf}
 !else
 ${If} $RuntimePresent == 0
  InitPluginsDir
  SetOutPath "$PLUGINSDIR"
  File "${WEBVIEW}\MicrosoftEdgeWebView2RuntimeInstallerX64.exe"
  DetailPrint "正在安装 Microsoft WebView2 / Installing Microsoft WebView2..."
  ClearErrors
  ExecWait '"$PLUGINSDIR\MicrosoftEdgeWebView2RuntimeInstallerX64.exe" /silent /install' $1
  ${If} ${Errors}
   SetErrorLevel 1
   Abort "WebView2 无法启动安装 / WebView2 installation could not start."
  ${EndIf}
  Call DetectWebView2
  ${If} $RuntimePresent == 0
   MessageBox MB_ICONSTOP "WebView2 安装未完成，请重试。 / WebView2 installation failed; please retry. Exit: $1" /SD IDOK
   SetErrorLevel 1
   Abort
  ${EndIf}
 ${EndIf}
 !endif
 SetOutPath "$INSTDIR"
 ClearErrors
 SetOutPath "$INSTDIR\sensor-tools"
 File "${ROOT}\target\release\sensor-tools\*"
 SetOutPath "$INSTDIR"
 File "${ROOT}\target\release\deskbox-reconstructed.exe"
 File "${ROOT}\target\release\WebView2Loader.dll"
 File "${ROOT}\README.md"
 File /oname=小组件使用说明.md "${ROOT}\docs\widgets.md"
 SetOutPath "$INSTDIR\media-tools"
 !ifdef ONLINE
  IfFileExists "$PLUGINSDIR\ffmpeg.exe" 0 ffmpeg_ready
  CopyFiles /SILENT "$PLUGINSDIR\ffmpeg.exe" "$INSTDIR\media-tools\ffmpeg.exe"
  ffmpeg_ready:
 !else
  File "${ROOT}\target\release\media-tools\ffmpeg.exe"
 !endif
 File "${ROOT}\target\release\media-tools\FFmpeg-LICENSE.txt"
 File "${ROOT}\target\release\media-tools\FFmpeg-VERSION.txt"
 ${If} ${Errors}
  SetErrorLevel 1
  Abort "复制程序文件失败 / Could not copy application files."
 ${EndIf}
 SetOutPath "$INSTDIR"
 WriteUninstaller "$INSTDIR\uninstall.exe"
 CreateDirectory "$SMPROGRAMS\${APP_NAME}"
 CreateShortcut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\deskbox-reconstructed.exe"
 CreateShortcut "$SMPROGRAMS\${APP_NAME}\卸载.lnk" "$INSTDIR\uninstall.exe"
 CreateShortcut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\deskbox-reconstructed.exe"
 WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "${APP_NAME}"
 WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "0.2.0"
 WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\deskbox-reconstructed.exe"
 WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
 WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '$\"$INSTDIR\uninstall.exe$\"'
 WriteRegStr HKCU "${UNINSTALL_KEY}" "QuietUninstallString" '$\"$INSTDIR\uninstall.exe$\" /S'
 WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
 WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
 WriteRegDWORD HKCU "${UNINSTALL_KEY}" "EstimatedSize" 96000
 Call RefreshDetection
 ReadINIStr $0 "$PLUGINSDIR\detected.ini" "Detection" "ServiceReady"
 ${If} $0 == 1
  Goto sensor_install_done
 ${EndIf}
 IfSilent sensor_install_done
 MessageBox MB_YESNO|MB_ICONQUESTION "是否启用真实 CPU 核心温度？将请求一次管理员授权，安装按需启动的采集服务。以后 DeskBox 无需管理员运行。也可稍后在系统状态小组件右键菜单中启用。" IDNO sensor_install_done
 InitPluginsDir
 SetOutPath "$PLUGINSDIR"
 File "${ROOT}\installer\sensor-service.ps1"
 nsExec::Exec '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -WindowStyle Hidden -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\sensor-service.ps1" -HelperPath "$INSTDIR\sensor-tools\DeskBox.Sensor.exe" -Action Install -Hash ${SENSOR_HELPER_SHA256}'
 Pop $0
 SetOutPath "$INSTDIR"
 sensor_install_done:
SectionEnd

Function un.onInit
 SetShellVarContext current
 SetRegView 64
FunctionEnd
Section "Uninstall"
 !insertmacro CheckExecutableUnlocked "unapp" "deskbox-reconstructed.exe"
 !insertmacro CheckExecutableUnlocked "unmedia" "media-tools\ffmpeg.exe"
 ReadRegStr $0 HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${AUTOSTART_NAME}"
 ${If} $0 == '$\"$INSTDIR\deskbox-reconstructed.exe$\"'
 ${OrIf} $0 == "$INSTDIR\deskbox-reconstructed.exe"
 ${OrIf} $0 == '$\"$INSTDIR\deskbox-reconstructed.exe$\" --autostart'
 ${OrIf} $0 == "$INSTDIR\deskbox-reconstructed.exe --autostart"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${AUTOSTART_NAME}"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run" "${AUTOSTART_NAME}"
 ${EndIf}
 Delete "$DESKTOP\${APP_NAME}.lnk"
 Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
 Delete "$SMPROGRAMS\${APP_NAME}\卸载.lnk"
 RMDir "$SMPROGRAMS\${APP_NAME}"
 IfSilent sensor_remove_done
 MessageBox MB_YESNO|MB_ICONQUESTION "是否同时移除 DeskBox CPU 温度采集服务？这会影响其他用户的 DeskBox；PawnIO 共享组件会保留。需要管理员授权。" IDNO sensor_remove_done
 InitPluginsDir
 SetOutPath "$PLUGINSDIR"
 File "${ROOT}\installer\sensor-service.ps1"
 nsExec::Exec '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -WindowStyle Hidden -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$PLUGINSDIR\sensor-service.ps1" -HelperPath "$INSTDIR\sensor-tools\DeskBox.Sensor.exe" -Action Remove -Hash ${SENSOR_HELPER_SHA256}'
 Pop $0
 SetOutPath "$TEMP"
 sensor_remove_done:
 !include "${ROOT}\installer\sensor-delete.nsh"
 Delete "$INSTDIR\deskbox-reconstructed.exe"
 Delete "$INSTDIR\WebView2Loader.dll"
 Delete "$INSTDIR\README.md"
 Delete "$INSTDIR\小组件使用说明.md"
 Delete "$INSTDIR\media-tools\ffmpeg.exe"
 Delete "$INSTDIR\media-tools\FFmpeg-LICENSE.txt"
 Delete "$INSTDIR\media-tools\FFmpeg-VERSION.txt"
 RMDir "$INSTDIR\media-tools"
 Delete "$INSTDIR\uninstall.exe"
 RMDir "$INSTDIR"
 DeleteRegKey HKCU "${UNINSTALL_KEY}"
SectionEnd
