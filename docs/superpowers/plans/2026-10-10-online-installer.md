# Online installer

User authorized downloading both FFmpeg and WebView2 during installation. Preserve offline artifact and app binary. Online NSIS embeds only application, loader, notices, and a PowerShell dependency downloader. Download FFmpeg over HTTPS to temporary storage, verify pinned SHA256 before copying. Skip network when installed FFmpeg matches. Download WebView2 bootstrapper only when runtime detection fails, verify Microsoft Authenticode and install silently. Download failures allow retry/cancel before any app payload is replaced. Uninstall retains existing ownership rules.

1. Add failing online contract checks.
2. Add dependency script; parameterize existing NSIS/build helper with --online and independent outputs.
3. Verify successful real download, bad hash rejection, compile production and diagnostic installers.
4. Run isolated install/upgrade/media/UI/uninstall tests and preserve evidence; document size/network requirements and clean-VM limitation.
