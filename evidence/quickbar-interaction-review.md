# Quickbar frame interaction and ball-menu review

Scope: ui/quickbar-bridge.js itemPointer/drop/menuEntry/toggle; tools/patch_frontend.py replacement of OverlayView Se; corresponding interaction test. Reviewer performed no source edits or native taskbar operations.

## Assessment

No established important regression found in the requested scope.

- Ball menu label derives from current published hideTaskbar state and toggling fetches fresh state before requesting its inverse. save's busy guard still serializes the settings request.
- The recovered Se retains selection/context-menu behavior and delegates the existing H/K/re drag and ordinary frame move logic through itemPointer.
- Pointer movement below six pixels does not begin a drag. Merely entering the pin area cannot persist anything. Completion requires the originating pointer's left-button pointerup; cancel, Escape, lost capture and genuine blur clean up without committing.
- Pinning requires membership of the item in the specified source frame. Pin command uses the existing path reference and leaves frame items intact; when a drop is accepted it bypasses the original frame-to-frame commit.
- Listener cleanup removes the same window capture listeners, releases pointer capture/hold and restores focusability. The handshake temporarily ignores blur while the native focus request is pending.

## Verification

`node tests/quickbar-interaction.cjs`: passed, QUICKBAR_FRAME_DRAG_AND_MENU_OK. This covers menu labels, focus-handshake blur, hover/no commit, right-button noncommit, left pin, source membership unchanged, cancellation, ordinary frame movement and genuine blur cancellation.

The test uses a simulated DOM/native focus bridge. Physical pointer capture across the separate dock HWND and actual native focus behavior remain the root agent's live integration verification; static review and synthetic dispatch do not establish those physical input cases pass.
