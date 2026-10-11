# Fence Merge Implementation Plan

> Agentic workers: use subagent-driven-development for isolated implementation tasks and review.

Goal: implement the approved persistent grouping, five-second content merge previews, detach and one-shot restore.
Architecture: preserve Fence records; add flat groups and restore records in Config. Frontend module owns pointer lifecycle, menus and temporary animation; Rust commands atomically mutate persisted state.
Tech stack: current Rust/Tauri, recovered Vue frontend, plain JS/CSS modules. No new external dependencies.

Global constraints: use approved spec ../specs/2026-10-10-fence-merge-design.md. No disk file moves. No implementation changes to original dist. Same visible monitor/WebView operation. 5000ms real time. Preview never commits.

## Tasks
- [x] Backend: failing tests in tests/fence_merge.rs; src/fence_merge.rs and Config structs implement group, select, detach, merge, restore, delete/remap handling. Commands group_fences/select_fence/detach_fence/merge_fence_contents/restore_fence_merge. Preserve scanning and legacy config tests.
- [x] Frontend: failing timer tests; ui/fence-merge.js and CSS implement drag state machine, original hold/busy/snap integration, title switch menu and right-click restore submenu, theme previews and cleanup.
- [x] Integration: patch_frontend.py adds precise asserted hooks; Rust command registration; verify frontend renders only active group member with container geometry without losing own styles. Reconcile refresh and rename.
- [x] Verification: core, timing, native isolated WebView merge/restore/menu checks and existing regressions. Release build, code review, package and report real-pointer/hardware test limits accurately.

Interfaces: frontend bind({state,invoke,hold,region,busy}); visible(fence), geometry(fence), pointer(event,fence,snap,save), title(event,fence), menu(entries,fence). Backend receives stable source/target/member/group/point IDs and validates live config. Merge records store source Fence, previous target item order, timestamp and record ID; restore consumes most recent applicable record.

Steps per task: write regression → observe failure → implement → run focused tests → inspect integration → update checklist. No git repository exists; do not initialize one solely to commit work.
