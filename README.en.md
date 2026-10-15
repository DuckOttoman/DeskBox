# DeskBox

English | [简体中文](README.md)

A Windows 10/11 x64 desktop organizer built with Rust, Tauri 2, and WebView2. Organize desktop files in folder boxes, add resizable widgets, and customize animated backgrounds.

## Features

- **Desktop organization:** classification by extension, shortcut target, keywords, and learned preferences; dragging, search, rename, reveal location, properties, and Recycle Bin actions.
- **Folder boxes:** individual styles, dimensions, collapse/hide controls, icon-label visibility, GIF/video backgrounds, and rounded clipping.
- **Grouping and content merging:** drag a box over another and release within 5 seconds to group their frames; release after 5 seconds to merge contents. Holding only previews the operation; moving away or cancelling does not commit it.
- **Group interaction:** click the title to switch members, drag the title to move the entire group, or hold the left button stationary for 2 seconds to detach the current member. Member properties remain independent. Content merges offer restore points.
- **Video modes:** energy-saving playback up to 854×480 at 24–30fps, or original resolution and frame rate. Animated backgrounds pause when hidden/collapsed and in relevant system states.
- **Widgets:** clock/weather, system status, countdown/Pomodoro, notes/tasks, and a photo album. Resize their borders to adapt between small, medium, and large forms.
- **Weather:** approximate IP-based city detection on creation, manual city selection, and weather emoji in medium/large forms.
- **Album:** add pictures through its menu; storage defaults to `Albums` beside the executable and can be changed.
- **System status:** CPU, memory, and supported NVIDIA metrics. Temperature uses Windows ACPI thermal-zone counters; firmware readings are not necessarily CPU core temperatures.
- **Memory optimization:** trims reclaimable working sets for DeskBox and its WebView2 processes and reports the result without reducing video quality.
- **Layout backups:** the floating-ball menu provides backup and restore management. Entries show their name, backup timestamp, and counts, and support renaming and deletion. Restoring first creates an automatic backup of the current layout.

Layout backups store box/widget positions, sizes, properties, and frame-group relationships, not desktop file lists. Restoring reclassifies the current desktop while retaining current notes, tasks, album contents, and timer runtime state. Deleted widgets return with empty contents. Backups are stored in `layout-backups.json` in the application’s user-data directory.

## Installation and updates

Use the generated online installer for a per-user installation. It downloads missing WebView2 and FFmpeg components in the background with console windows hidden. An existing WebView2 runtime is reused.

For an existing installation, use a replacement-update ZIP:

1. Exit DeskBox from the floating-ball or tray menu.
2. Replace `deskbox-reconstructed.exe` and `WebView2Loader.dll` in the installation directory with the ZIP’s files.
3. Keep `media-tools`, `Albums`, and the user-data directory, then restart.

The default installation path is `%LOCALAPPDATA%\Programs\DeskBox Reconstructed`. Settings use the separate application identifier `com.deskbox.reconstructed`. Exiting restores desktop icons hidden by this application.

Installers, update bundles, and build caches are excluded from this source repository and generated locally.

## Building from source

Requires Windows x64, Rust, and a linker: Visual Studio C++ Build Tools for MSVC, or MinGW-w64 for GNU. Release builds have been verified with Rust 1.99.0 GNU. Running the GUI requires WebView2 Runtime.

The repository includes build-ready `frontend/` assets; Node.js and npm are not required.

```powershell
.\build.ps1
```

Equivalent commands:

```powershell
cargo test --no-default-features --locked
cargo build --release --locked
```

The executable is written to `target/release/deskbox-reconstructed.exe`. `build.ps1` also copies it to `bin/`, together with the WebView2 DLL and media tools when available.

After changing extensions in `ui/`, regenerate the frontend first:

```powershell
python tools/patch_frontend.py
```

### Media tools and installer

Prepare FFmpeg using Python 3, then build the online installer:

```powershell
python -m pip install imageio-ffmpeg
python tools/prepare_media_tools.py
python tools/build_installer.py --online
```

The installer script prepares NSIS and requires network access during packaging. Output is written to `deliverables/`. Portable deployments must keep `media-tools/` and its license/version notices. Rust compilation itself does not require FFmpeg.

For the particular local TLS environment addressed by this project, `.\build.ps1 -LocalToolchain` uses tools under `.tools/` without changing the global Rust installation. Standard builds are preferred in ordinary environments.

## Repository structure

| Directory | Contents |
|---|---|
| `src/` | Rust backend, classification, layout, persistence, and Windows integration |
| `dist/` | Frontend assets recovered from the original executable |
| `frontend/` | Frontend embedded in the current application |
| `ui/` | Readable menu, merge-animation, media, and widget extensions |
| `tests/` | Core regressions and isolated WebView diagnostic drivers |
| `installer/` | NSIS setup and dependency download scripts |
| `tools/` | Build, frontend generation, and packaging utilities |
| `docs/` | Usage guides, designs, and implementation records |
| `evidence/` | Selected reconstruction evidence and verification records |

## Verification and limitations

Core tests, Release builds, isolated WebView interactions, layout-backup persistence across restarts, and installation/upgrade/uninstallation have been verified. Diagnostic data is isolated from the real desktop. Simulated pointer tests do not cover every physical mouse interaction, mixed-DPI multi-monitor setup, or hot-plug scenario.

Temperature and GPU metrics depend on hardware, firmware, and drivers; unavailable readings remain unavailable. Original-quality video decoding depends on WebView2 and system codecs. Multiple high-resolution video backgrounds increase resource usage. Old background caches are not automatically deleted on every replacement.

See the [widget guide](docs/widgets.md) and [layout-backup design](docs/superpowers/specs/2026-10-11-layout-backup-design.md) for more detail; these documents are currently in Chinese.

## License

The repository’s existing [GNU GPL v3 license](LICENSE) is retained. Third-party dependencies, recovered assets, and FFmpeg remain subject to their respective source licenses and accompanying notices.
