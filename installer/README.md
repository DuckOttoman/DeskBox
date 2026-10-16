# Windows installer

Target: Windows10/11 x64, current-user installation. Default folder: `%LOCALAPPDATA%\Programs\DeskBox`. Desktop/start-menu shortcuts and Windows uninstall registration are provided. Configuration stays in the app's normal user-data directory; uninstall retains data and the shared WebView2 runtime.

The setup contains the verified DeskBox executable, WebView2Loader, FFmpeg with notices, and Microsoft's signed x64 Evergreen standalone runtime installer. A missing WebView2 runtime triggers offline `/silent /install`, and installation aborts if registry detection still fails afterward. Existing runtime is reused. Source: https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution . The embedded installer URL is the x64 URL used by Tauri: https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-bundler/src/bundle/windows/util.rs .

Build app through build.ps1, prepare FFmpeg through tools/prepare_media_tools.py, then run `python tools/build_installer.py`. This downloads compiler/runtime through official HTTPS to workspace-local .tools and validates the Microsoft signature. Output: deliverables/DeskBox-0.2.0-windows-x64-setup.exe plus SHA256 manifest. No build tools are required on destination PCs.

For safe testing run `python tools/build_installer.py --test`, then `tools/test_installer.ps1`. It uses a distinct registry/shortcut/autostart identity and an explicitly checked workspace directory. `-InstallName '安装验证 app'` tests Unicode/spaces. Installed app diagnostics use their own config and generated media fixtures. Running executable overwrite is rejected, upgrade preserves unrelated files, uninstall deletes only enumerated app files and its own matching autorun entry.

The setup is not code-signed because no publisher certificate was supplied. Windows may show an unknown publisher prompt. Local installation/playback/upgrade/uninstall were tested; a clean Windows VM with WebView2 absent has not been tested. No x86/ARM-native installer is supplied.
# 联网安装版

构建：`python tools/build_installer.py --online`。产物为 `deliverables/DeskBox-0.2.0-windows-x64-online-setup.exe`，约9.4 MiB（含 CPU 采集组件），现有离线版保留。

首次安装需要联网下载约84 MiB的FFmpeg。同版本已安装且SHA256匹配时跳过下载。来源为imageio维护的FFmpeg二进制仓库，使用固定SHA256校验，不匹配则拒绝安装。仅在检测不到WebView2时，从Microsoft下载Evergreen引导程序，检查有效Microsoft Authenticode签名，然后由引导程序联网安装运行时。GitHub/Microsoft下载源必须可达，国内网络可能较慢。

组件准备完毕后才写入软件文件。失败可重试或取消，错误日志位于 `%TEMP%\DeskBoxReconstructed-download-error.txt`。安装后正常使用无需为这些组件保持联网。安装包尚未签名；Windows10/11 x64适用。

验证：`python tools/build_installer.py --online --test` 后执行 `tools/test_installer.ps1 -Online -InstallName '联网安装验证 app'`。本机已验证实际下载、安装、升级、播放和卸载；Microsoft引导程序下载与签名检查通过，缺少WebView2的全新虚拟机安装流程尚未实测。


## 安装识别（2026-10-15）

安装器读取当前用户／本机的32位和64位注册表视图，并检查记录中的主程序是否实际存在。失效记录不会遮蔽常见目录中的有效安装；用户明确指定的 `/D=` 始终保留。目录页之后展示安装检测摘要，包括目标路径、程序、WebView2、FFmpeg、CPU采集文件、PawnIO与全局采集服务。文件组按当前安装包的SHA256判断完整、一致或需更新／补齐；不同摘要不直接断言文件损坏，也可能是旧版本。相同的全局温度服务不重复请求授权。

WebView2按微软文档读取有效版本注册值；这不是完整的运行时损坏诊断。任意磁盘中的无注册便携副本不会遍历扫描，需要手动指定目录。未登记的 deskbox.exe 不作为自动覆盖目标。

诊断：`setup.exe /S /DETECT=C:\path\report.ini` 只生成检测报告并退出，不执行安装；`/D=` 如使用必须放在命令行最后。策略回归：`powershell -NoProfile -File tests/installer-detection.ps1`。编译安装器后，另用只读探测验证真实机器和显式目录，再执行隔离安装回归。
