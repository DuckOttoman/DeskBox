# Icon names and rounded video background

Per-fence style.hideIconNames toggles through existing title context-menu bridge and update_fence. Latest fence is cloned before update, preserving items and other style fields. CSS root class hides labels, retaining tooltip/name data and ensuring merge preview clones inherit the target frame's label visibility. Recovered original assets are unchanged.

Root cause: recovered `.bg` has inherited border radius and overflow:hidden, while new video/blurred layer can be separately composited. Explicit clip-path on fixed background host uses the same --fb-radius as the outer frame, including configured 26px→28px changes. Clip is on host rather than expanded blurred layer. Guarded class addition avoids MutationObserver self-trigger loops. Cleanup removes the media host class and restores poster visibility.

Native diagnostics also encountered own monitor-sized overlay being classified as a fullscreen application when focused. Media policy now excludes current process from that fullscreen check, keeping external fullscreen/lock/power checks intact.

Evidence: icon-names-red.log fails missing menu option, icon-names-green.log passes reversible saved toggle without pre-save state mutation. Old native binary icon-radius-red/runtime.log fails media_rounded_clip_matches_fence. Final icon-radius-green/runtime.log verifies clip, live radius update, actual hide/show menus and computed label visibility/tooltips, video eco/original playback, GIF and static replacement. icon-radius-merge and icon-radius-settings both report SMOKE_RESULT success=true. Node menu/rename/pointer/timing/media tests pass. Release build successful. Tests use isolated data; no physical screenshot/pixel clipping comparison is claimed.
