# Media Background Plan

Global constraints: approved spec ../specs/2026-10-10-media-background-design.md. No original dist or user files/config changes. No git repo; edit current workspace and use per-plan evidence/ledger instead of Git-only helpers. Skill-directed independent tasks may use isolated-context agents, disjoint files. Interface chosen before implementation.

- [x] Task 1 backend (agent): core media.rs, tests/media.rs, lib.rs export, commands.rs import/picker/mode/clear/policy, win.rs low-overhead power policy. FFmpeg and owned paths, atomic publication, output validation, failure cleanup. Interface as spec. Do not change app.rs registration/diagnostic branch, frontend or packaging. TDD and full core tests.
- [x] Task 2 frontend/integration (controller): ui/media-background.js/.css with lifecycle, patch_frontend.py asserted style panel hooks, backend registration in app.rs. Node tests before implementation. Existing static UI and native focus maintained.
- [x] Task 3 native validation (controller): generate isolated media fixtures, native driver + smoke flag, policy/import/lifecycle controls, verify actual output resolution/fps, legacy regression and release build.
- [x] Task 4 reviews/packaging: read-only task and final reviews, correct blocking findings, portable FFmpeg notices/runtime helper, current artifacts and source, honest performance limits.

No new model settings required; per-fence style JSON backward compatible. Agent reports edits/tests/evidence to evidence/media-backend-report.md. Core media paths and command names coordinated before testing; UI receives asset converter An through bind rather than private Tauri URL reconstruction.
