# Recovered frontend contract

Original bundle SHA256 is recorded in assets.json. frontend-readable.js is a formatting-only copy.

## Commands (exact original compiled wrapper)

```javascript
const te={getState:()=>_("get_state"),setRegion:e=>_("set_region",{rects:e}),updateFence:e=>_("update_fence",{fence:e}),createFence:e=>_("create_fence",{origin:e}),deleteFence:e=>_("delete_fence",{id:e}),moveItem:(e,t,n)=>_("move_item",{item:e,to:t,index:n}),itemAction:(e,t)=>_("item_action",{id:e,action:t}),organize:e=>_("organize_now",{full:e}),toggleFences:e=>_("toggle_fences",{origin:e}),createWidget:(e,t)=>_("create_widget",{kind:e,origin:t}),updateWidget:e=>_("update_widget",{widget:e}),deleteWidget:e=>_("delete_widget",{id:e}),setTheme:e=>_("set_theme",{theme:e}),updateSettings:e=>_("update_settings",{settings:e}),saveBall:(e,t)=>_("save_ball",{x:e,y:t}),openSettings:()=>_("open_settings_window"),resetLearned:()=>_("reset_learned"),quit:()=>_("quit_app"),keyboardFocus:e=>_("keyboard_focus",{on:e}),pickFenceImage:e=>_("pick_fence_image",{id:e}),clearFenceImage:e=>_("clear_fence_image",{id:e}),watchOutsideClick:e=>_("watch_outside_click",{enable:e})}
```

## Events

- state: {config, items: Item[], monitors: Monitor[], categories: Category[], wallpaper, ready}
- icons: object mapping item ID to cached image path
- fences-visible: {visible: boolean, origin: [physicalX,physicalY]}
- sysmon: cpuName, cpuUsage, cpuMhz, cpuTemp, memUsed, memTotal, gpuName, gpuUsage, gpuMhz, gpuTemp, vramUsed, vramTotal
- outside-click: payload is unused

## Coordinates and windows

Window labels: ball, settings, overlay-*. origin and save_ball use physical screen coordinates. Fence/widget coordinates use monitor-local logical units. set_region receives null (full window) or an array of physical client rects {x,y,w,h,r,ellipse}. An empty array means no interactive content.

## Known reconstruction choices

Backend control flow, classification precedence, configuration defaults, learned-key strategy and layout geometry are reconstructed. Rule data and original frontend bytes are extracted from the original binary.

## Added rename command

`rename_item {id, name}` renames a current scanned item, validates a single Windows filename, retains shortcut extensions and never replaces an existing destination. It preserves the fence slot while remapping path-derived IDs and individual learning keys. The derived frontend exposes a file context menu action with a keyboard-accessible input dialog. Original `dist` bytes remain preserved; the active assets and hashes are recorded in `modified-assets.json`.

