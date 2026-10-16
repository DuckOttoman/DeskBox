# Quickbar read-only code review

Reviewed taskbar ownership/recovery, runtime, policy, application/command hooks, frontend bridge and UI against docs/superpowers/specs/2026-10-16-quick-access-bar-design.md. No native taskbar changes were performed. Source was being edited concurrently; findings refer to source read on 2026-10-16.

## Findings

1. **P2 — Filter learning to applications and application shortcuts.** `D:/Users/Liang/AppData/Local/DeskBox/deskbox-reconstructed/src/commands.rs:112` sets `launched` solely from `action == "open"`, and line 115 records every successful open. `src/quickbar_runtime.rs:110` likewise records successful opens of any pinned path. Consequently opening a desktop PDF, image or directory adds that document/directory to the automatic recommendation candidates. `quickbar::record_launch` only validates the path shape; `ranked` accepts all recorded entries. The agreed design restricts recommendation sources to desktop apps/application shortcuts opened through DeskBox. Add target eligibility detection before recording in both routes (including resolving shortcuts so document shortcuts are excluded); manual pin support remains broader.

2. **P2 — Retain per-monitor fullscreen suppression when focus moves to another display.** `D:/Users/Liang/AppData/Local/DeskBox/deskbox-reconstructed/src/taskbar.rs:38` considers only `GetForegroundWindow`. With a borderless fullscreen video/presentation on display A, clicking a normal application on display B makes fullscreen_on(A) false even while the fullscreen window remains visible on A. `Controller::tick` then re-shows its always-on-top quickbar over that fullscreen content. Inspect visible fullscreen candidates per monitor (with suitable exclusions/occlusion checks), or maintain monitor fullscreen state independently of only the current foreground window.

## Verification and positive checks

- `node tests/quickbar-ui.cjs`: passed, 17 assertions.
- `node tests/quickbar-bridge.cjs`: passed.
- Did not run Cargo concurrently with the root agent's build.
- Pin/remove/reorder operate on config references only; no move/delete/recycle call found in these paths.
- Recovery journal records process creation identity, original taskbar visibility/state; watchdog readiness precedes taskbar hiding. Stale HWNDs are re-enumerated for restore.
- Runtime quickbar locks are released before snapshot reads/mutations in the reviewed command routes; no concrete lock-order cycle identified.
- Application startup currently re-applies protected geometry after successful takeover and publishes it.
- Native crash recovery, monitor topology, mixed-DPI and real fullscreen behavior require root agent integration evidence; static review does not establish those pass.

## Scoped native re-review (later source revision)

The two original findings above have been addressed in the inspected source: learning now calls scanner::recommendation_allowed and pinned opens require matching desktop items; fullscreen inspection now walks visible Z order independently per monitor and excludes cloaked windows. The independent reservation HWND has its own message loop, transparent/noactivate styles, owner-thread destruction via WM_CLOSE, and AppBar removal before restoration. No additional concrete HWND lifecycle/thread deadlock was established by static inspection.

New findings in this revision:

3. **P2 — Bind watchdog loop/cleanup to its generation, not only the reused journal filename.** `D:/Users/Liang/AppData/Local/DeskBox/deskbox-reconstructed/src/taskbar.rs:92`: after startup token validation the watchdog loop only checks whether the common journal path exists. If disable removes the journal and re-enable recreates it before the old watchdog's 250ms wait finishes, the old watchdog remains alive observing the new generation until the entire application exits. Repeated fast toggles accumulate watchdog processes. Conversely its unconditional final deletion of the common `.ready` filename can remove a new watchdog's marker during re-enable, making an otherwise ready takeover time out. Validate journal token while waiting and ready token before cleanup, or use generation-specific files and bounded watcher shutdown.

4. **P2 — Recheck topology before permanently rolling back a reservation error.** `D:/Users/Liang/AppData/Local/DeskBox/deskbox-reconstructed/src/quickbar_runtime.rs:45`: topology checks occur every two seconds, but Controller::tick reserves/verifies work areas every 300ms. A resolution/DPI/display change between topology polls leaves the controller using old monitor geometry; reserve/verify can fail first, and the error branch disables the feature and persists hide_taskbar=false before topology recovery gets a chance. Re-evaluate topology on a tick error and rebuild if it changed, or ensure topology is checked before geometry assertions. This is a normal user topology transition, not an exceptional unrecoverable state.

Additional error-path observation: topology_tick's sync_monitors failure branch restores taskbars by the preceding disable but returns without persisting hide_taskbar=false or publishing failure. This is lower priority than findings 3–4; keep state/UI consistent if that operation can fail.

No native taskbar operations or source edits were performed during the re-review. No concurrent Cargo invocation was made while the root release build ran.

## Final scoped follow-up

Findings 3 and 4 are resolved in the latest inspected source: readiness paths include the session token; watchdog loops revalidate the current journal token each iteration and clean only their own readiness path. Runtime reservation errors immediately recheck topology before permanent rollback.

5. **P2 — Let transient Explorer absence bypass topology reconstruction.** `D:/Users/Liang/AppData/Local/DeskBox/deskbox-reconstructed/src/quickbar_runtime.rs:58` obtains zero taskbar targets while Explorer/taskbar windows are temporarily unavailable. It treats that as a topology change, disables the existing controller, then fails re-enable because no taskbar exists and persists hide_taskbar=false. This topology check executes before Controller::tick, so it bypasses the new no-shell wait branch at `src/taskbar.rs:121`. Reproduction condition: Explorer's restart gap overlaps the two-second topology polling interval. Explicitly defer topology reconstruction when the primary Explorer taskbar is unavailable, allowing the controller to hide/wait and re-register on return. Legitimate taskbar-monitor topology changes with an available primary taskbar can continue to rebuild normally.

The root agent reports supervised normal enable/disable/re-enable/exit and forced-main-crash scenarios passed in evidence/quickbar-native-normal-110521 and evidence/quickbar-native-crash-110541. These were reported by the root, not rerun by this reviewer. Latest source still needs the root's final rebuilt native rerun. No source edits/native operations were performed by this reviewer.

## Review closure

Finding 5 is resolved: topology_tick now explicitly returns false for empty taskbar target enumeration, preserving the existing Controller's hide/wait/re-register path during Explorer absence. All five reported P2 findings are addressed in the source inspected at review closure. No outstanding important finding remains within the requested review scope. Final rebuilt native verification remains with the root agent. The lower-priority sync_monitors error-reporting observation above was not elevated to a release blocker.

## Last focused change review

Reviewed the added named recovery mutex, pointer-capture ReorderSession flow, and quickbar-hide/toggle-off delay. No new important issue found in these changed areas. The mutex protects stale recovery, parent-death recovery, journal creation, and normal disable; its guard accepts abandoned ownership and releases/closes on the same synchronous scope. Session-specific readiness cleanup remains intact. Pointer reorder preserves a frozen pin snapshot, validates left-button completion, cancels on lost capture/pointer cancellation, suppresses the launch click after movement, and leaves backend set equality validation in place. Toggle-off holds CHANGING across the asynchronous animation wait and performs actual restoration afterward; normal shutdown still uses immediate disable.

Reviewer reran `node tests/quickbar-ui.cjs` for the final pointer-reorder changes: passed (25 assertions). Root reports six native pure tests pass; final release build/native rerun remain root-owned. No source edits or native taskbar operations were performed in this focused review.
