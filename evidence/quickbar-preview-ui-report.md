# Drag preview inside the native Dock window

The source folder-frame ghost cannot paint above a separate topmost native Dock through CSS. The Dock now mirrors the active drag image in its own native window from Window-targeted `quickbar-drag-preview` events. Coordinates identify the cursor center; the visual image is 48×48 CSS pixels centered there. The overlay is inert, aria-hidden, pointer-events:none, and above menus/Dock. It has no data-hit marker and does not modify the base Dock layout.

The visual rectangle is explicitly appended to `quickbar_regions`, clipped to the WebView viewport with `radius:0`. Ordinary state refreshes and focus changes preserve the preview. Null payload or `quickbar-hide` clears it. Drag previews freeze automatic re-ranking. Diagnostic mode emits `quickbar_dom_report` with ok:true and message `QUICKBAR_DRAG_PREVIEW_RENDERED` once per drag; normal user mode does not emit this report. Test inspection includes preview visibility, geometry, icon, name, report flag and hit-marker absence.

`quickbar-pins-changed` forces immediate state refresh even while hover-frozen. Refresh work now serializes and coalesces forced requests arriving during an in-flight poll, including when that poll fails; a successful queued request supersedes the failed poll.

Validation: `node tests/quickbar-ui.cjs` passes 36 assertions. New tests cover full-size centered geometry, clipped screen edges, offscreen/invalid geometry, forced request coalescing and forced refresh surviving a failed poll. `node --check ui/quickbar.js` passes. New geometry tests were run failing before the helper was added. This report does not claim physical cross-native-window mouse validation or actual taskbar operations; those remain root integration checks.

## Drag insertion invitation

Added a noninteractive 60px insertion slot plus 12px spacing when the mirrored cursor enters the actual Dock bounds expanded by 24px horizontally, 36px above and 12px below. The base width is tracked separately so repeated preview events cannot add cumulative width. Empty 336px Dock expands to 408px on roomy screens. At the 68% cap a dashed insertion hint appears inside the existing Dock. The slot has no data-path/data-hit marker, is aria-hidden and inert, and never changes entries or calls pin/reorder commands. Null/leave retracts the invitation; the mirrored 48px image remains separately above it.

Expansion uses a 200ms width animation and retraction 160ms, with the existing motion/speed policies. During expansion, retraction and ordinary pin/remove width animations, native clipping receives the union of old/target centered Dock bounds (radius 22), then clears it after the latest animation finishes. The image region remains radius 0. Inspection exposes insertion state/base width/target width/slot visibility/animation union.

Latest validation: 43 assertions and JS syntax check pass. Added failing-first geometry tests cover nearby boundaries, leave/null, 68% cap, empty expansion and repeat-event stability. Physical mouse and native visual smoothness remain root integration checks.
