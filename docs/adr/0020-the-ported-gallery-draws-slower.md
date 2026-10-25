# 0020 The ported gallery draws slower

Date: 2026-09-29. Status: accepted.

The ported debug gallery draws its heaviest page about 1.6 times slower than on gpui 0.2.2 (tooling: 9 fps, worst frame 117 ms, against 14 fps and 72 ms), with gpui's accessibility tree on or off (`Application::new_inaccessible` measured the same). Ely's code did not change there; the frame cost is gpui's.
