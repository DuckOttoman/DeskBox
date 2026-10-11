# Ball settings and UI responsiveness

- [x] Trace original ball settings action and native reconstruction call chain; inspect local Tauri documented synchronous creation deadlock.
- [x] Add isolated first-open regression without setup-created settings; verify freeze before fix.
- [x] Move IPC settings creation and tray creation off UI callbacks, serialize creation; test first open, close/reopen, minimize/restore and repeated calls.
- [x] Audit slow desktop scan locking and tray organize for additional UI blocking; make any confirmed fix separately with regression evidence.
- [x] Independent review, Release build, native menu regression, updated runtime/source archives and provenance verification.
