# Per-fence icon names and media radius

User explicitly requests reversible hide/show icon-name entry in fence menus and video corners matching fence radius. Implement this approved scope directly; no extra approval for ordinary implementation choices. Existing workspace is not a Git repo.

Use per-fence style.hideIconNames, default false, persisted through existing update_fence. Toggle label hides/shows accordingly; clone latest fence before saving. Hide only name labels using a per-fence class, preserving original filename, tooltip, selection and drag handlers. Keeping label markup lets merge preview clones follow destination label visibility. Existing style merging/restoration already retains per-fence styles.

Recovered background `.bg` already has inherited radius and overflow:hidden. New video layer has no compositor clip, so add explicit rounded clip-path to media background host using same --fb-radius variable as fence. Clip the fixed host rather than blurred/inset video child, preserving blur and geometry. Remove class when media removed.

Verification: failing menu unit test; old-binary native rounded clip repro; new native menu hide/show and actual DOM labels plus per-fence persistence, existing media/regression tests. Compile and provide new portable/source packages with FFmpeg notices.
