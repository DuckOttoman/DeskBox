# Rename implementation plan

- [x] Add failing Rust tests for validation, no-overwrite filesystem operations and fence/learned-key remapping; add a frontend DOM interaction test.
- [x] Add src/rename.rs and state remapping; integrate serialized AppState rename and rename_item command.
- [x] Derive patched frontend bundle from preserved original; add readable dialog JS/CSS and reproducible patch tool.
- [x] Run regression and frontend tests, compile Release, run hidden WebView2 smoke test; review the affected changes.
- [x] Update provenance/README/verification and regenerate source and Windows runtime packages.

Verified: 8 core + 7 rename Rust tests passed; UI tests including pending-command keyboard focus passed; final Release build succeeded; hidden WebView2 context menu + file rename + original contract smoke exited 0. Review findings fixed and tested. Original software unchanged.
