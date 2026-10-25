# 0017 The site shows components live

Date: 2026-09-28. Status: accepted, partly superseded by 0032.

The site shows components live, as gpui-kit.com does, not as screenshots. gpui 0.2.2 cannot build for the browser: Zed added gpui_web on 2026-02-26 and has published no gpui since 0.2.2. So gpui moves to Zed's repo by git at 1a28cff (T44b), the code gpui-pre 0.3.7 republishes; the source stays first-party, and this replaces the 2026-09-24 choice. The terminal stays native: alacritty_terminal's polling does not build for wasm32.
