# Quick Access Bar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Add an optional themed quick-access dock with safe Windows taskbar recovery and shared layout protection.

**Architecture:** A native taskbar controller owns restoration, AppBar reservations and an independent watchdog. A pure Rust policy owns persisted references, usage scores and protected geometry. Dedicated dock WebViews render these references and dispatch existing Shell actions; recovered settings and drag geometry use small explicit bridges.

**Tech Stack:** Rust, Tauri 2, public Win32 APIs, Vue bridge, JavaScript/CSS, Python build tools.

## Global Constraints

- Default off; count 12, integer 10…MAX or MAX; MAX=max(12,floor(logical width*.68/72)).
- Each originally taskbar-bearing monitor gets a dock; normal windows retain dock access; fullscreen hides it per monitor.
- Cell60, icon48, gap12, edge margin12; horizontal padding6; overflow never discards pinned paths.
- No Explorer injection, termination or administrator requirement. Restore original visibility/autohide on disable, exit and main-process crash.
- Protect fences, groups and widgets in preview and persistence; preserve layout contents and current user files.
- Usage only from successful DeskBox dispatch, 14-day half-life. Removing references never deletes files.
- Respect theme, animSpeed and reduced motion. No GitHub push.

### Task 1: Native taskbar ownership
Files: create `src/taskbar.rs`, `tests/taskbar-policy.rs`; integrate later in `src/app.rs` and `src/main.rs`.
Interface: `taskbar::Controller::enable(directory, docks)` returns an owned guard; `disable`, `tick`, `recover`, `watchdog_main` provide idempotent restoration and crash recovery. Docks carry HWND and Monitor.
- [x] Add tests for monitor matching, reservation calculations and journal identity before implementation.
- [x] Run tests, observe missing module failure, implement Win32 enumeration, AppBar reservation, saved state and watchdog handshake.
- [x] Verify recovery without changing the real taskbar first; record native limitations explicitly.

### Task 2: Persisted policy and shared bounds
Files: create `src/quickbar.rs`, `tests/quickbar.rs`; modify `src/model.rs`, `src/lib.rs`, `src/app.rs`, `src/commands.rs`.
Interface: `QuickbarConfig` stores pins, exclusions, aggregate usage; `limit(width)`, `record_launch`, `pin`, `remove`, `entries` implement policy. `protect(config, monitors)` clamps frames to published effective work area.
- [x] Add failing tests for defaults, max/DPI, dedupe, decay, exclusion/repin, overflow and group/widget bounds.
- [x] Implement pure policy and run `python tools/cargo_local.py test --no-default-features --locked`.
- [x] Connect commands, successful Shell dispatch, global geometry normalization, monitor publication and native guard lifecycle.

### Task 3: Dock UI and settings
Files: create `ui/quickbar.html`, `ui/quickbar.js`, `ui/quickbar.css`, `ui/quickbar-bridge.js`, `tests/quickbar.cjs`; modify `tools/patch_frontend.py`, `ui/widgets.js`, capabilities and frontend bridge.
Interface: `quickbar_state`, `quickbar_pin`, `quickbar_remove`, `quickbar_open`, `quickbar_ready`; window labels `quickbar-*`. State events carry themed config and resolved reference entries. Shared `DeskBoxQuickbar.bounds()` constrains previews.
- [x] Test slot/overflow and stable ranking while interacting, then implement dedicated dock UI.
- [x] Add behavior checkbox/count and restore shortcut, external/native and internal reference-only drag pinning, menus/undo/missing paths.
- [x] Implement theme/reduced-motion animations and clamp fence/widget previews.

### Task 4: Native verification and delivery
Files: native test driver, `docs/quick-access-bar.md`, evidence report and packaging helper.
- [x] Patch frontend, run JS/Rust tests, release compile, isolated WebView checks.
- [x] Announce temporary native taskbar tests; verify disable, normal exit and forced main termination restore state, keeping guard alive.
- [x] Review source changes for ownership/races, rerun affected checks after fixes.
- [x] Build online setup, full replacement ZIP and source archive; report measured checks and any untested multi-screen hardware scenarios.

No commits are made here: this active reconstructed source directory is not a Git checkout. Isolated runtime fixtures protect current configuration. Source review compares explicit files and saved evidence.
