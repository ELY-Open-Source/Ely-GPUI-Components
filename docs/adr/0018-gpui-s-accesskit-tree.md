# 0018 gpui's AccessKit tree

Date: 2026-09-28. Status: accepted.

Gpui builds an AccessKit tree on macOS, Windows and Linux, not the web, from elements with an id and a `role`, so the accessibility entries turn from `blocked` to `prove`. It still has no tray or badge API, so TrayIcon and DockBadge call AppKit.
