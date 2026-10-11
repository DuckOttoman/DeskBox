# DeskBox context menu fidelity

Goal: preserve recovered DeskBox menu entries, inline expansion, movement/recycle safeguards; repair verified display/click failures without replacing menus with Windows Shell menus.

Architecture: original dist remains byte-identical. Patch the PopMenu component only in derived frontend through tools/patch_frontend.py; keep readable viewport placement helper in ui/menu-layout.js and scoped CSS constraints. Existing native commands unchanged except a genuine bug if evidence establishes one.

- [x] Trace original PopMenu, file/fence/widget entry construction, action payloads and hit-region hold/release.
- [x] Reproduce complete menu, move destinations, delete confirmation, bottom-edge expansion with 13-fence fixture in real WebView2; obtain failing assertion before fixing.
- [x] Write unit regression for dynamic placement and oversized content. Implement measured placement with lifecycle cleanup; retain source actions, disabled flags and inline expansion. CSS confines long menus with scroll.
- [x] Verify full move click/refresh and confirmation-only recycling using isolated fixtures, dismissal and subsequent menu opening; original system items protected.
- [x] Review changes, compile, run regressions, package updated runtime/source and document exact tested scope.
