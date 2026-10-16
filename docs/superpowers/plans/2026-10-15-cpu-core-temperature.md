# CPU Core Temperature Implementation Plan

**Goal:** Read live core minimum/maximum with independent per-widget modes and isolated privileged CPU-only collector.
**Architecture:** C# LibreHardwareMonitor collector runs either demand-start Windows service in protected Program Files or elevated portable worker. Rust reads fresh authenticated local responses; widget data stores display mode. No production config changes during diagnostics.

- [x] Pin official hardware library/driver dependencies, inspect current CPU sensor identifiers and service APIs; build helper and test sensor filtering/instantaneous aggregation before implementing.
- [x] Implement CPU-only collector, read-only restricted local IPC, expiry, service install/start/remove in protected paths; portable elevation path. Verify service access boundaries and failure behavior.
- [x] Rust sampler integrates current values/status without stale cache. Add right-menu modes and persistence, display core range, package/ACPI separate fallback. Regression tests for current min/max and malformed/expired input.
- [x] Build release; native UI and real hardware checks; test install/upgrade/uninstall; package update and buildable sources with dependency licenses. Record exact verified limitations. No GitHub upload.

Working in existing non-Git source directory; private GitHub clone remains separate. Do not reset source or overwrite personal runtime data. User has approved service/portable design, including required Windows authorization prompt.
