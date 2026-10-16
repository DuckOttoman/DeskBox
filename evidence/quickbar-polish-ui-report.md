# Theme glass Dock polish

Implemented the approved `docs/superpowers/specs/2026-10-16-quickbar-polish-design.md` within the dedicated quickbar HTML/JS/CSS and UI policy tests.

- Unified Dock, tooltip, context menu, overflow panel, undo notice and drop hint with a subtle diagonal highlight, 1px inset edge, soft shadow and themed glass surface. Actual persisted theme IDs supported: glass-dark, glass-light, minimal, neon, sakura, forest. Theme variables cover foreground, secondary text, edge, surface, hover and shadow.
- Empty Dock now displays a code-native SVG plus icon, “从文件夹框拖入图标” and secondary hint “固定常用项目，原位置保持不变”. Width is 336 logical pixels on roomy screens and capped to 68% of narrow screens; both labels ellipsize without horizontal overflow. The former black text pill is removed.
- Kept height 72px, bottom offset 12px, cells 60px, icon 48px, spacing 12px and padding 6px. The inset edge does not add border dimensions. Existing pointer ReorderSession, native OLE listeners, event contracts and readiness sequence were preserved. Existing animation and reduced-motion paths remain; high-contrast mode replaces glass with system surfaces and explicit boundaries.

Verification: `node tests/quickbar-ui.cjs` passes 28 assertions, including new wide/narrow empty-state width cases; `node --check ui/quickbar.js` passes. The new width tests were run failing first (helper missing), then passed after implementation. Native visual integration and physical cross-window mouse tests are not claimed by this UI-only report; root integration owns those checks and packaging.
