# AGENTS.md: ely-misc

Rules for `ely-misc`, on top of the root `AGENTS.md` and `crates/ely-core/AGENTS.md`.

## Stack

- Web views: `wry` 0.57 (Apache-2.0 OR MIT, default features off, macOS only) lays a child WKWebView over the window.

## Rules

- A wall clock reads `typography::format::now` and wakes its view at each whole second through a keyed task, only while the owner passes no moment; a stopwatch reads the executor's clock, so tests advance it (`misc::Clock`, `misc::Stopwatch`).
- A component that reads the host's frames reads one at a time off the main thread and passes over frames that come meanwhile; when a read ends it redraws, so the newest frame is read next (`misc::QrCodeScanner`).
- A native view laid over the window takes AppKit's first responder on a press, and gpui's view never asks for it back, so typed text would go to the native view for good. The component hands the keyboard back on every press gpui gets and when the view leaves (`misc::WebView`, through wry's `focus_parent`); `Step::NativeKey` proves it, since gpui's own dispatch skips AppKit.
