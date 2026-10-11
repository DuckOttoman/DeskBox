# Settings and scan responsiveness review

Independent reviewer confirmed a P1 synchronous WebView creation deadlock and a P2 state-lock/UI-thread scanning problem. Settings IPC is now async and uses spawn_blocking; native tray callbacks dispatch worker actions, and a separate creation mutex serializes first creation. Refresh and rename share filesystem_operation before briefly acquiring snapshot; Shell work occurs outside snapshot. No reverse lock order or new rollback/mapping issue was found.

Reviewer identified and resolved a test gap: unchanged scanned items skipped commit. Slow-scan diagnostic now requested=true, forcing merge/save/commit, and verifies concurrent edits plus rename mapping. Final slow-scan run passed with state roundtrip 1ms during a simulated 3-second scan. First-open ball/tray regressions passed, including concurrent requests, visible/ready state, close/reopen, minimized restoration and repeated calls. Native tray regression exercises the same dispatch function as the event callback; it does not automate a physical tray click.

Original frontend own menus remain unchanged. The audit does not establish all original Rust backend behavior or physical/multi-monitor interaction equivalence.
