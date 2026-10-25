# 0014 WebView on wry

Date: 2026-09-28. Status: accepted.

WebView proven with `wry` 0.57: a child WKWebView over the window, placed from a canvas in prepaint at its box each frame and hidden once the box leaves the view. It draws above everything gpui paints and gpui's clipping does not reach it; gpui's test windows have no native handle, so the capture proves it.
