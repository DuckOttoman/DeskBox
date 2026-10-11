# File context menu rename

User request: add a rename action to the desktop item's right-click menu. Implement within the authorized reconstruction project; do not change the installed original executable.

Use an application input dialog, rather than Explorer's native rename UI. It uses existing keyboard-focus and hit-region behavior, preloads the current display name, selects the basename, and offers Confirm/Cancel and Enter/Escape. Errors remain visible with the input. System shell icons have a disabled rename action. Files, folders and shortcuts are supported. Shortcut extensions .lnk/.url are retained invisibly; regular file extensions are visible and editable.

Rust receives only item ID and new name, looks up the current managed item and validates a single Windows filename. Reject empty/whitespace names, separators, reserved device names, trailing space/dot, and control characters. Use the non-replacing Windows MoveFileW operation so collisions never overwrite another file. Case-only changes are supported. Existing file content is untouched.

Serialize scan and rename with the snapshot lock. On success rescan and replace the old path-derived item ID in its existing fence position, transfer learned name/path keys and preserve category choice. Persist the updated config before committing in-memory state. On scan/save failure attempt rollback and report any rollback failure. Publish the updated state event after unlocking.

Keep original frontend assets as provenance and load a derived patched JS bundle plus a small readable rename dialog module. Delivery verification records original and derived asset hashes separately.

Verify actual filesystem renames only using temporary fixture files in the workspace or temp directory, including collision/content preservation, invalid names, folder and shortcut behavior, case-only renames, and assignment remapping. Run a mocked DOM frontend interaction test, then Rust tests, Release build and the existing bounded original frontend smoke check. No user desktop file is renamed during tests.
