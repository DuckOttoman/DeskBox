# Native fidelity review closure

Read-only independent review checked Shell96, conditional alpha, configuration import, stable IDs and desktop visibility ownership.

Corrected findings:
- Desktop ownership now records (HWND, Explorer PID) under a mutex. A new window clears old ownership; already-hidden replacement windows are never restored by us. A shutdown flag prevents a racing worker from hiding after exit restoration.
- Native diagnostics reject missing or invalid PNG dimensions. Actual desktop tests verify visible -> hidden -> visible; PNG pixels compared against original cache.
- Added regression for ordered old hash-ID migration and per-name learning precedence.

Evidence: native-fidelity-core.log (20 passing tests), native-fidelity-final/native-check.json, native-fidelity-pixel-comparison.json, smoke-native-fidelity-final/runtime.log.

Boundaries: this is not proof of complete parity. Explorer restart/hotplug were reviewed in code, not induced on the user's session. Original classification priority, executable metadata and t:/u: learning remain unverified.
