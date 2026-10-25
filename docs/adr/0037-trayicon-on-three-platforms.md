# 0037 TrayIcon on three platforms

Date: 2026-10-21. Status: accepted.

`TrayIcon` runs on macOS, Windows and Linux. `TrayLook` gives an SF Symbol for the macOS menu bar, a PNG for the rest and the name hovering shows. A left click picks the first item; on Windows a right click opens the menu at the pointer, and an icon lost to an Explorer restart comes back. Linux registers with `org.kde.StatusNotifierWatcher` off the UI thread and fails after 3 s without a host (GNOME without an extension), so the owner can keep its window instead.
