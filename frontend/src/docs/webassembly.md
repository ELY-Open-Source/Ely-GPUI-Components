# WebAssembly

Ely builds for `wasm32-unknown-unknown` through `gpui_web`, Zed's browser platform for GPUI. The components page runs Ely's own gallery that way, in a canvas inside the page.

## Check the build

```sh
rustup target add wasm32-unknown-unknown
RUSTC_BOOTSTRAP=1 cargo check --lib --target wasm32-unknown-unknown
```

`gpui_web` uses an unstable feature for its threads, as Zed's own web examples do, so the check needs `RUSTC_BOOTSTRAP=1`.

## Start an app in the browser

The gallery's [web entry](https://github.com/ELY-Open-Source/Ely-GPUI-Components/blob/main/examples/gallery/web.rs) shows the steps:

| Step | Why |
| --- | --- |
| `#[wasm_bindgen(start)]` on the entry | The browser runs it when the module loads. |
| `gpui_platform::web_init()` | Sends panics and logs to the browser console. |
| `WebPlatform::new_with_backend_and_font_fallback(false, WebBackendPreference::Auto, CanvasFontFallback::EmojiAndCjk)` | One thread, since a static site gets no cross-origin isolation; WebGPU where the browser has it, WebGL2 where it does not; the browser lends fonts for emoji and CJK alone. |
| `Application::with_platform(platform)` | Then `with_http_client(Arc::new(platform.fetch_http_client()))`, which fetches pictures by address, and `with_assets(Assets)` and `run`, as on the desktop. |

## Build the gallery

`ELY_GALLERY_ASSETS=<address>/ scripts/web.sh <out>` builds the gallery with the `web` profile, runs `wasm-bindgen` and `wasm-opt -Oz`, and copies its page beside it and its pictures into `<out>/assets/`. It needs the `wasm-bindgen` CLI at the version in `Cargo.lock` and binaryen's `wasm-opt`, and it fails when the wasm passes Cloudflare's 25 MiB. `ELY_GALLERY_ASSETS` is the address `<out>/assets/` is served from, ending in `/`: GPUI reads no files in a browser.

The gallery opens at `?page=<slug>&story=<title or slug>&theme=light|dark`. A story needs its page.

## Embed it in a page

| Message | Direction |
| --- | --- |
| `gallery.setTheme(mode)`, or `postMessage({ ely: "theme", theme })` | From the host, before the wasm runs or after. |
| `{ ely: "ready" }` | To the parent, once the gallery runs. |
| `{ ely: "failed", error }` | To the parent, after a start that failed three times. |

Until a press or a key reaches it, the embedded gallery keeps GPUI's hidden text field from taking focus, so a starting frame leaves the host page's focus alone. Tab inside GPUI cycles within its window; a host gives a way past the frame, as the components page does.

## What differs on the web

| Area | On the web |
| --- | --- |
| Threads | One in the gallery: more need a page served with cross-origin isolation. |
| Pictures | Fetched by address, never read from files. |
| Time | `web_time::Instant`; std's panics in a browser. |
| Paste | Arrives as the browser's paste event. Code that reads the clipboard itself awaits `read_from_clipboard_async`. |
| Accessibility | GPUI builds no accessibility tree in the browser. |
| Native only | The live terminal and the gallery's capture tool do not build for WebAssembly. |
| macOS only | The tray icon, the Dock badge and the web view return an error or say so, as on Windows and Linux. |
