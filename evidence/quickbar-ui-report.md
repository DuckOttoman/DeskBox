# Quickbar UI implementation

Created dedicated transparent `ui/quickbar.html`, `ui/quickbar.js`, `ui/quickbar.css`; no existing UI files edited.

UI implements 60px cells / 48px icons / 12px gaps / 6px padding, bottom 12px centering, 68% cap, overflow consuming one slot, no blank filler, pinned indicators, disabled missing targets, explicit removal with undo, native file-reference pin drops, pinned drag reordering, safe text rendering, tooltip, light/dark/high-contrast/reduced-motion behavior, animation speed scaling, and frozen automatic refresh while interacting. Polling pauses while document is hidden. Explicit mutations refresh immediately. Native startup requires initial state, hit regions and drag event registration before `quickbar_ready`; failures render a visible status and console error.

Backend contract: `quickbar_state`, `quickbar_regions({regions:[{x,y,width,height}]})`, `quickbar_ready`, `quickbar_open({path})`, `quickbar_pin({paths})`, `quickbar_remove({path})`, `quickbar_undo({path})`, `quickbar_reorder({paths})`. Regions use logical CSS coordinates. Monitor `w` is assumed physical and divided by `scale`. Both settings.quickbarCount and settings.quickbar_count accepted. Tauri internal core bridge and Window-targeted event plugin listeners used.

Verification: `node tests/quickbar-ui.cjs` passes all 12 layout/freeze assertions; `node --check ui/quickbar.js` passes. Tests were executed first and initially failed because quickbar.js did not exist. No native runtime or visual integration claim is made; root integration must copy these assets into dist and verify Rust commands/native hit regions and event permissions.

Final integration: native OLE pinning retained; pointer-capture sorting replaces HTML drag to coexist with Windows WebView2 native drop. quickbar-hide event implements reduced-motion-aware exit animation. 25 policy assertions pass.
