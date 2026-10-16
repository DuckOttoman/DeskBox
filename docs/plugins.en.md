# DeskBox plugin development (API v1)

A .deskbox-plugin is a ZIP with manifest.json at its root. API v1 supports script-free themes and HTML/CSS/JS widgets. These examples require DeskBox 0.1.0.

## Build and install

Run from the source root (Python standard library only):

```powershell
python tools/package_plugin.py examples/plugins/clock deliverables/plugins/clock-1.0.0.deskbox-plugin
python tools/package_plugin.py examples/plugins/mist-theme deliverables/plugins/mist-1.0.1.deskbox-plugin
```

Open Settings → Plugins → Local import, select the package, review its source, author and permissions, then confirm installation. Alternatively paste a trusted HTTPS package URL in URL install. Preview never executes plugin code. Enable a widget and select it from Add widget. Only one theme can be active. Disabled widgets retain layout placeholders. Uninstall can retain or delete private settings. Publisher identity and source are self-declared, not verified identities.

The packager sorts paths and fixes ZIP timestamps and permissions; identical inputs produce identical bytes in the same Python/zlib environment. It prints SHA256. Development directories .git, node_modules, __pycache__, .venv, venv and .env/.env.* are excluded. Never include secrets or personal data. Limits: 20 MiB compressed, 50 MiB unpacked, 512 files, 10 MiB per file. Links, traversal, absolute paths, case collisions, Windows reserved names and executable files are rejected.

## Manifest

See ../../examples/plugins/clock/manifest.json for a complete widget manifest. Required fields: apiVersion=1, id, name, version, author, type, minDeskBoxVersion. Widgets also require entry. Use a stable lowercase reverse-domain id, such as org.example.clock. Versions have three numeric components. Themes load theme.json. Optional fields are description, homepage and updateUrl. Permissions:

```json
{"permissions":{"network":["https://api.example.com"],"systemMetrics":true}}
```

Network entries are HTTPS origins without paths or queries. Metrics require explicit systemMetrics permission. Settings are automatically isolated by plugin ID. Request only necessary permissions; update previews display permission changes. updateUrl returns {"id":"org.example.clock","version":"1.1.0","url":"https://example.com/clock-1.1.0.deskbox-plugin","sha256":"64 hexadecimal characters"}. Update checks are manual.

## Widget SDK

The host injects window.DeskBox before entry scripts. No external SDK is required. Request methods return Promises; subscriptions return unsubscribe functions.

| Method | Contract |
| --- | --- |
| ready() | Resolves to {theme,size:{width,height,form},visible} |
| getTheme() | Current host theme |
| onThemeChange(callback) | Theme changes |
| onResize(callback) | {width,height,form}, logical pixels; small/medium/large |
| onVisibilityChange(callback) | Boolean visibility; pause timers/animation when hidden |
| getSettings() | Current plugin's JSON settings |
| setSettings(value) | Persist JSON, maximum 256 KiB per plugin |
| getSystemMetrics() | Existing host samples; requires systemMetrics permission |
| fetch(url) | GET; resolves to {status,body,contentType}, with textual body |

The clock demonstrates initialization, resizing, visibility and persistent 12/24-hour preference. The host destroys instances on disable/uninstall; do not assume code continues running. Accept additional theme fields. Metrics follow the host sample structure and sensors may be absent; handle missing values.

The sandbox permits scripts but denies same-origin privileges, popups, top navigation, filesystem, shell and arbitrary Tauri commands. Direct network requests and external scripts/CDNs are blocked. Use SDK fetch: the host checks granted origins, HTTPS and public destinations at every redirect; local, private, link-local and metadata targets are denied. Time, response-size and concurrency limits apply. Display errors within the widget; do not bypass permissions.

## Themes

Mist contains JSON and no scripts. theme.json maps variables: --db-plugin-background, --db-plugin-color, --db-plugin-accent, --db-plugin-border, --db-plugin-radius, --db-plugin-shadow, --db-plugin-font. Values are restricted CSS strings: no semicolons, braces, HTML, url() or expression. Themes cannot inject selectors or override host layouts.

## Catalog and publishing

A catalog is {"schemaVersion":1,"entries":[...]}. Entries contain id, name, description, author, type, version, url and sha256; see ../../server/market/catalog.json. Configure an independent HTTPS catalog for discovery; direct URL/local installation always remains available. Publish immutable versioned package paths and verify SHA256. The catalog's IP URLs target this project's deployment; online installation requires deployment and a trusted certificate. Do not disable certificate verification.

Before release: inspect ZIP contents; preview/import; enable; add and resize; switch theme; hide/show; save settings and restart; disable/re-enable; uninstall with settings retained/deleted. Run python -m unittest discover -s tests -p test_plugin_packaging.py.

Theme JSON may also set top-level `"backgroundImage":"assets/mist.png"`. Only an existing package PNG/JPG/JPEG/WebP/GIF path is accepted; absolute paths, traversal, SVG and remote URLs are rejected. The host converts the checked resource to a raster data URL, without exposing arbitrary CSS URLs. Variable values also reject backslashes and @ to block CSS escape/import tricks. See Mist 1.0.1 for a complete example.

CSS resource functions url(), image-set(), image(), src() and expression are rejected after whitespace normalization. Use only backgroundImage for package raster resources; CSS values cannot authorize network loading.

Use classic `<script src="main.js">`, stylesheets and packaged images. Version 1 does not resolve JavaScript module imports; bundle dependencies into a classic script before packaging. CDN scripts and direct network access are blocked; use the authorized SDK fetch method.
