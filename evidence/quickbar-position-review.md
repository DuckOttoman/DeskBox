# Quickbar bottom-position fix review

Scope: latest src/quickbar_runtime.rs anchoring/visibility/lock-release changes and tools/test_quickbar.ps1 position assertions. Read-only source review; reviewer did not change sources or operate native taskbars.

## Assessment

No established important defect found in this scoped change. Dock WebViews remain hidden while the AppBar reservation changes rcWork. Once takeover succeeds, each dock uses physical monitor origin/bottom and its actual outer height, avoiding mixed logical/physical units or assumptions about outer frame height. Startup anchors both before and after noactivate show. Visible runtime ticks re-anchor and fullscreen-return ticks use the same positioning path. The Runtime mutex is released before WebView window APIs, avoiding holding that lock across UI dispatch. Errors after controller installation retain the existing disable/restore cleanup path.

The native regression now asserts dock outer bottom equals monitor bottom (within 2 physical pixels) both after enable and after fullscreen return, and still checks taskbar visibility, flags and work-area restoration. Root reported the original failing rectangle [0,1124,2560,1344] on a monitor bottom of 1440; these new assertions directly reject that 96px offset. Final rebuilt live execution remains root-owned.

Small verification detail: tick anchors before noactivate show whereas initial enable anchors before and after show. The fullscreen-return native position assertion is therefore useful evidence that Windows does not re-relocate that return path. Static inspection alone cannot prove Shell positioning behavior; no speculative blocker is raised absent a failing rerun.
