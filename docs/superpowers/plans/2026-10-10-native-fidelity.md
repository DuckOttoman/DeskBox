# Native fidelity fixes from original evidence

User reports visibly inferior icons and original desktop icons staying visible. Fix these before claiming parity. Original evidence: 195 96x96 icon-cache images plus 9 aspect-fit thumbnail images; original code at 0x140140819 requests IShellItemImageFactory::GetImage with SIZE(96,96), flags 0, followed by GetObject/GetDIBits and conditional alpha handling. Current code stretches ordinary SHGetFileInfo icons to 48x48. Original user config enables hideDesktopIcons, while legacy reconstruction config disables it.

- [x] Write failing pixel conversion, config compatibility/migration and native-preference tests.
- [x] Implement the original 96px Shell image factory and rounded conditional unpremultiplication; keep thumbnail aspect ratio and regenerate in a new cache directory.
- [x] Preserve original path-based IDs and versioned configuration. On first upgrade back up legacy reconstruction config, import original preferences/layout/widgets/backgrounds read-only, and preserve original per-name learning records.
- [x] Make desktop window discovery scoped and resilient; verify hide/restore outcomes, reapply desired state after Explorer recreates its view and expose errors rather than silently ignoring them.
- [x] Compile and perform actual Windows desktop hide/restore and icon extraction checks, then WebView2/rename regression checks. Compare pixels with available original cache images.
- [x] Review affected code and deliver updated binaries/source with exact verified scope and remaining differences.
