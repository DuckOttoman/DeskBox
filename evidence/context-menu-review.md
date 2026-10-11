# Context menu review

Independent read-only review found no blocking issues in the measured-position patch, scoped scrolling CSS, preserved actions or lifecycle cleanup.

Original inline expansion and disabled semantics remain. ResizeObserver updates only changed dimensions and is disconnected before unmount; viewport resize listener is removed. The original full-window hit-region hold remains while any overlay menu is open.

Regression coverage: recovered file entry labels, full move destination list, submenu expansion bounds, final destination reachable by scrolling, real move_item via menu leaf and restored slot, menu dismissal after leaf, bottom-right placement, delete confirmation and no parent-item execution, rename and backend command contract. Runs use13 and60 fences with isolated fixture files.

Boundaries: hidden WebView2 tests use DOM click/scrollTop, not physical mouse wheel/touch input. Pure placement tests cover resized viewport dimensions; no real screen resize/hotplug is induced. Recycling actual user files is not part of this test.
