# Media background review

Backend implementer reviewed frontend: found poster/video opacity stacking; fixed by hiding recovered `.bg > .image` only while media has loaded successfully, restoring on suspension/error.

Independent reviewer found dynamic-to-static replacement retaining inline visibility:hidden. Cleanup now restores poster visibility. Old-binary native test reproduces failure in evidence/media-static-red/runtime.log at the static replacement assertion; new-build green must verify it. Reviewer re-read fix and confirmed no remaining blocking issue.

Review also noted cache retention grows disk usage after replacements/removals. Current cache stays intact to preserve content-merge restore references; limitation documented in README. No live cache deletion introduced. CPU/GPU percentage and physical battery-saver/locked desktop behavior have not been measured.

Earlier native test found incorrect assumed container `.layers`; inspected recovered FenceBox constants/rendering to confirm actual `.bg`, then corrected DOM selector. Real visible isolated WebView run verifies media resolution, real UI mode selector, clock advancement, collapsed source release, GIF and clear. Hidden smoke windows intentionally pause playback, so media test uses existing standalone diagnostic window option.
