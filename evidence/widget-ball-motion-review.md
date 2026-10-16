# Widget and ball motion review
Implemented timer completion ring/bounce and floating-ball breathing/menu opening/closing. Completion follows successful atomic backend transition; actions remain immediate. Animations do not modify saved geometry. Ghost menu is inert and has no hit regions; cancellation removes it and releases the window-region hold.

Independent source review found recovered menu/root row CSS animations still active under reduced motion and clone rows replaying entrance. Fixed by disabling original ball-menu entrance CSS and ghost descendant animations, and suppressing remaining ball/menu CSS transitions when motion is paused/reduced. Native checks cover these computed styles.

Verified release build, duration policy, widget regression, isolated native completion/cleanup/repeated and rapid menu reopening, unchanged ball geometry, reduced-motion behavior, update ZIP hashes, and isolated install/upgrade/media UI/uninstall/user-data preservation. Physical mouse/multiple monitor checks and missing-runtime clean VM installation were not separately repeated. Existing normal installation and config were not changed. GitHub was not updated by this task.
