# Weather and memory implementation plan
User approved: medium/large weather emoji; manual working-set trimming limited to this app and its WebView2 descendants, preserving original video. Additional user instruction: IP city lookup when creating a clock; manual fallback and no override of saved cities.
1. Add failing emoji and ownership-selection tests.
2. Add weather emoji and IP initialization with bounded requests and manual fallback; save only city coordinates, never raw IP.
3. Add Windows working-set adapter with process ancestry/creation-time checks, reporting and 10-second cooldown; add ball menu feedback.
4. Run core/JS/native WebView tests, review, build updated online installer and source.
