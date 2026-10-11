# Ball Settings freeze and interaction audit

## Confirmed creation hazard

Original Ball menu invokes open_settings_window (frontend-readable.js around line 6152). The reconstruction handles this as a synchronous command and directly builds the settings WebView. Tauri 2.12.2 local WebviewWindowBuilder documentation explicitly warns that creating a WebView from a synchronous command or event handler deadlocks on Windows; it recommends async commands or separate threads. The tray settings callback has the same direct build path. The previous regression mode created settings inside setup, so all subsequent opens bypassed first creation.

The added isolated test starts without a settings window, drives the recovered ball context menu, then checks first creation, close/reopen, minimize/restore, repeated opens and backend responsiveness. No real desktop files/config are used.

## Additional audit finding

refresh holds the state snapshot mutex while scanning desktop entries, resolving shortcuts and extracting Shell images. The tray organize callback directly calls refresh on the UI event thread. Slow Shell operations can consequently block state commands and tray UI. This requires a separate responsiveness regression and filesystem-operation serialization to keep stale scans from undoing rename mappings.

## Regression evidence and corrections

First-open red fixture stopped at SETTINGS_OPEN_REQUEST and exceeded 18 seconds; no SETTINGS_CREATED appeared. The async/background correction completed first creation, close/reopen, minimize/restore and repeated opens. Settings creation uses a separate mutex to serialize simultaneous callers. The native tray handler dispatches the same creation routine on a worker.

Slow-scan red fixture measured get_state blocked for 3004ms during a 3-second scan callback. Refresh and rename now share a file-operation gate, perform scanning outside the snapshot mutex, and merge into the latest state under the snapshot lock. Rename failure keeps filesystem rollback. This avoids stale scans undoing path remapping while allowing ordinary state commands during Shell work.

Audit boundaries: original frontend resources remain the source of menu contents. Original backend classification priority and t:/u: learning behavior, physical drag/resize and hotplug parity remain unproven (see parity-audit.md). This audit does not claim complete equivalence to original Rust internals.
