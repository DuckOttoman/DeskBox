# Quickbar vertical-position regression

The user reported the bar appearing above the original taskbar region. The new native geometry assertion reproduced the issue before the fix: monitor rectangle [0,0,2560,1440], dock window [0,1124,2560,1344]. The 96 physical pixel offset matched the AppBar reservation. Windows repositioned the ordinary top-level WebView into the reduced work area.

The dock now stays hidden until reservation stabilization, then anchors its physical outer rectangle to the monitor bottom before and after no-activate show. Periodic visibility handling rechecks the anchor, including return from fullscreen. Actual outer height is used, avoiding an extra work-area subtraction. Runtime locking ends before UI window operations.

The same regression now passes with dock [0,1220,2560,1440]. Native checks also pass after fullscreen return and repeated toggles, with taskbar flags, visibility and work areas restored on normal exit and forced main-process termination. Evidence: quickbar-native-normal-112929 and quickbar-native-crash-112947. Review: quickbar-position-review.md. Full Rust regression: 75 passed, 1 firmware-dependent test ignored. Mixed-DPI multi-monitor hardware and live Explorer restart remain untested.
