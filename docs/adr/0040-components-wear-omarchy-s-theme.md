# 0040 Components wear Omarchy's theme

Date: 2026-10-21. Status: accepted, partly superseded by 0042.

Every component wears Omarchy's theme when the app calls `settings::follow_omarchy`, at the owner's word: one call to opt in, `notify` to watch, the library alone (ElyOffice and ELYTerminal move later), no gallery story. The reader follows `omarchy-theme-color` key by key, and bash's `read`, which drops a last line without its newline. Three choices part from ElyOffice's copy: the terminal colors are Ghostty's from Omarchy's template (8 is `muted`), the selection is a wash that shows Omarchy's color over the page, and a color lifts toward white or black, whichever reads, per call. gpui's `set_window_appearance` is a no-op off macOS, so following leaves it alone. Following waits for 250 ms of quiet; on Linux inotify reports the watcher's own reads, and a test under WSL proves they start no read.
