# 0056 WebView stays macOS-only

Date: 2026-10-26. Status: accepted.

OPTIMIZATION.md O-21 offers two ways: bring `misc::WebView` to Windows and Linux, or mark it macOS-only. It is marked: wry would lay WebView2 and WebKitGTK over gpui's window, each a native view with its own focus and clipping to prove, as the WKWebView was. Elsewhere the box says web views need macOS. Its doc says `Platforms: macOS.`, as `ShareButton`, `EyeDropper` and the Dock badge do; `scripts/platforms.py` holds every such line to the cfgs in its code.
