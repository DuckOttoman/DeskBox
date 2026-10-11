# Recovered DeskBox context menu logic

Original frontend is byte-preserved in dist. Readable rendering in evidence/frontend-readable.js lines 5820–5995 defines PopMenu; 6515–6558 file entries; 7678–7691 overlay menu coordinates.

File entries: open -> item_action/open; reveal (disabled for system) -> item_action/reveal; properties -> item_action/properties; move -> move_item(item.id,targetFence.id,null); recycle -> confirmation child -> item_action/recycle (disabled for system). Added rename in derived frontend precedes properties only. Folder/system and fence/widget menus are distinct.

Submenus are inline CSS grid expansion on click. They are not Windows Shell context menus. PopMenu toggles one selected child index; leaf action closes menu; pointerdown stops parent stage close. Disabled entries and separators ignore actions. Opening overlay menu acquires st() full-window hit region; closing releases it and refreshes fence hit regions. Position uses fixed 270px horizontal and 380px vertical thresholds, independent of expanded height. Original min-width208/max-width260. Adding rename increases baseline menu height34px. A move list with12 destinations adds360px; potentially exceeds original fixed380px placement assumption. Confirmed in real WebView2: a 13-fence configuration at viewport height1440 rendered the expanded menu top1059/bottom1635.903. Missing destinations were clipped past the desktop edge.

Diagnostic tests now check original entry labels, complete move destinations, submenu expansion and delete confirmation using an isolated copy of13-fence configuration, without recycling any desktop files.

## Fixed and verified

Derived PopMenu measures its offset dimensions through ResizeObserver and clamps to the viewport on size/viewport changes. Original entries, leaf actions, inline expansion and hold/release are preserved. Scoped max-height and overflow allow long lists to scroll. Observer and resize event handlers are cleaned up on unmount.

Real WebView2 checks passed with13 fences and60 fences: original entries, destination labels, expanded bounds, scroll-to-last destination and its move_item dispatch, restoration of original assignment/slot, leaf closes menu, bottom-right bounds, delete parent opens confirmation without executing, rename entry/input, and existing command contract. Logs: smoke-menu-final/runtime.log and smoke-menu-long/runtime.log. Initial failing bounds: smoke-menu-red/runtime.log. No real desktop file recycled.
