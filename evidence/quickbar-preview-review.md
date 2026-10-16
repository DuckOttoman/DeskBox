# Quickbar topmost drag preview / insertion cue review

Scope: source preview queue and cleanup, read-only native forwarding, pin refresh event, renderer preview geometry/native clipping, and horizontal insertion expansion/retraction. Reviewer did not edit source or operate native taskbars.

## Important finding

**P2 — Preserve terminal preview clear when an in-flight preview request rejects.** `D:/Users/Liang/AppData/Local/DeskBox/deskbox-reconstructed/ui/quickbar-bridge.js:6`: sendPreview uses one serial/coalesced queue, but its outer catch sets previewPending=null. A prior move can successfully show the dock ghost; a later request can reject (for example source item/frame membership changes during a drag); if pointer cleanup queues the terminal `{id:null,source:null,point:null}` while that request is pending, its rejection discards the clear. The dock retains the ghost, insertion cue and sourceDrag freeze until another preview/hide event. Catch individual invoke failures while continuing to drain the latest queued payload, or guarantee a terminal clear is attempted after failure. Swallowing an intermediate preview error is reasonable; discarding the final cleanup is not.

## Other reviewed contracts

No additional important issue established. Backend validates origin overlay, source frame membership/display and finite points; it maps source logical coordinates through physical monitor coordinates into the frameless dock and emits preview/null without config/file mutation. The 48px ghost is inert, pointer-events:none, drawn above dock content, and its visual clip is included with zero native radius. Insertion appends one 60px slot plus 12px gap, caps at 68% viewport, and uses union of old/target dock rectangles during width animation; generation checks avoid stale diagnostic callbacks. Source preview freezes normal refresh ordering, while successful pin emits a forced refresh through the queued refresh runner. Clearing preview retracts insertion and releases sourceDrag freeze.

## Verification

- `node tests/quickbar-ui.cjs`: 43 assertions passed.
- `node tests/quickbar-interaction.cjs`: QUICKBAR_FRAME_DRAG_AND_MENU_OK.

Native compositing, actual pointer capture across HWNDs and screenshot appearance remain the root agent's integration verification.

## Scoped fix confirmation

The reported P2 is resolved in the latest ui/quickbar-bridge.js. Failures are caught per invocation inside the drain loop, preserving any queued terminal clear or newer preview. A failed move without queued work schedules one clear; a failed clear does not self-retry indefinitely. Reviewer reran `node tests/quickbar-interaction.cjs`, including the added in-flight rejection/cancel regression: passed (QUICKBAR_FRAME_DRAG_AND_MENU_OK). No outstanding important finding remains in this preview/insertion review scope. Final rebuilt native verification remains root-owned.
