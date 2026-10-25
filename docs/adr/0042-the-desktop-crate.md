# 0042 The desktop crate

Date: 2026-10-23. Status: accepted.

The tray, the Dock badge and Omarchy move to `ely-desktop` (OPTIMIZATION.md O-03), at the owner's call on the split; `settings::follow_omarchy` becomes `desktop::follow_omarchy`, `shell::TrayIcon` `desktop::TrayIcon`. `zbus` stays in `ely-core`'s Linux tree: gpui's Linux backend (`gpui-pre-linux`: accesskit's AT-SPI, ashpd, oo7, notify-rust) links the same 5.19 the tray used, through `gpui_platform`, which core takes for `font-kit`, `wayland` and `x11`, as GPUI Kit 0.8.0 does.
