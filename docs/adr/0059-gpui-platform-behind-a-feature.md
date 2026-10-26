# 0059 gpui_platform behind a feature

Date: 2026-10-26. Status: accepted.

`ely-core` takes gpui_platform only behind `platform`, off by default (OPTIMIZATION.md O-02, O-03; B7, B8), and moves to 0.2.0. It used gpui_platform for nothing but turning on `font-kit`, `wayland` and `x11`, which an app sets in its own gpui_platform line to open a window at all, so `cargo tree -p ely-core -i zbus` printed gpui's Linux backend. A Button-only app now links no gpui backend through Ely; zbus comes with gpui's Linux window, which the app asks for. The risk the old choice guarded against, an app without `font-kit` drawing no text on macOS in silence, or one without `wayland` and `x11` on Linux, where gpui falls back to the same no-op text system, is closed by `init`, which fails naming those features when the platform's text system lists no Ely font. `ely-gpui-component` keeps `platform` and `runtime_shaders` on by default, as GPUI Kit 0.8.1 does.
