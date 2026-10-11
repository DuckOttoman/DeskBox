# Layout backup implementation plan
Goal: approved ball-menu layout backup with timestamp, restore/rename/delete.
Architecture: dedicated layout-backups.json with atomic writes and serialized operations; pure capture/restore helpers; widget dialog reused for themed UI.
Tech: Rust/serde, Tauri commands, recovered frontend extension.
- [x] tests first: exclude files and widget contents; preserve current contents; restore groups/missing-monitor clamping; metadata rename/delete and persistence.
- [x] implement src/layout_backup.rs and command boundary; independent backup file, config restored atomically, auto pre-restore backup.
- [x] ui/widgets.js management dialog and tools/patch_frontend.py ball entries. Dates shown per entry; frontend flush queued notes before operations.
- [x] native isolated UI/IPC and restart tests, independent review, release build.
- [x] online installer and replace-update ZIP, installation checks, source archive.
Constraints: no desktop file modifications, no rollback of notes/todos/photos/timer runtime or global settings; deleted widgets restore empty; missing display remaps primary; unavailable background cleared. User approved, including backup timestamp. No Git repository.
