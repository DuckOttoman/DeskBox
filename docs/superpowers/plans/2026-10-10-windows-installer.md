# Windows installer plan

Explicit user authorization: turn current software into installer for other computers. Use current verified x64 payload unchanged, standard NSIS current-user installer for Windows10/11 x64, Chinese/English UI, selectable install directory, shortcuts, uninstall registration. No admin required for app installation. WebView2 Evergreen standalone x64 installer is embedded and runs only when absent, following Microsoft documented detection and offline workflow.

Preserve user data and shared WebView2 on uninstall. Delete only enumerated owned payload files and known shortcut/registry entries. Refuse replacing a running executable; do not kill user processes. Fresh installer installs do not include this computer's configuration, caches or files. No signing certificate provided, so unsigned setup disclosed.

1. Download workspace-local NSIS compiler and Microsoft offline runtime via official HTTPS. Verify Microsoft runtime signature. No system compiler installation.
2. Write failing installer contract tests, then installer script and reproducible packaging helper.
3. Compile production plus diagnostic variant with isolated app identifier/shortcuts/registry. Install diagnostic variant in explicit workspace evidence path, verify manifest, shortcuts/registry, launch media and UI smoke with isolated config, verify upgrade preserves unrelated file, uninstall preserves user sentinel and removes owned payload/registration. No missing-runtime system installation on this already-equipped development PC.
4. Inspect executable imports for external dependencies, payload hashes and setup output; document clean Windows VM limitation, package source and installer.

No Git repository exists. Build/test evidence replaces Git-only workflow. User has approved creating the installer; ordinary packaging choices do not need another permission request.
