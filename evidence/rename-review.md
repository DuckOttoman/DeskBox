# Rename review

Independent read-only review of Rust rename, configuration remapping, frontend modal and native integration.

- Extensionless files initially failed to transfer per-name learning. A failing regression test reproduced it; remapping now transfers differing non-extension keys and retains shared extension rules.
- Modal Tab focus initially escaped to background controls. A failing UI regression test reproduced it; Tab/Shift+Tab wrap enabled dialog controls.
- Submission previously disabled every focusable control. The input now remains focused and read-only while submitting; a delayed-command UI test verifies Tab remains inside the dialog.

Follow-up reviewer confirmed both P2 findings corrected and reported no remaining blocking issue. Snapshot locking, rename rollback and original hit-region bridge were reviewed as consistent.

Verification uses real Windows MoveFileW operations on unique temporary files, plus a WebView2 smoke test that clicks the derived context menu and renames only its own fixture file. User desktop files are not test inputs.
