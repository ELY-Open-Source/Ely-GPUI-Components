# Threat model

## What this project does and where untrusted input enters
Ely is a Rust component library for GPUI desktop and web apps. It runs inside the host app's process, so the host decides what is trusted. Treat as untrusted whatever an app's users or the network can hand a component:
- Markdown rendered by `documents` (pulldown-cmark) and text typed or pasted into fields (`forms::TextInput`, `editor::CodeEditor`).
- JSON and settings parsed with serde: saved layouts (`layout::workspace`), JSON editors and viewers (`devtools`), imported VS Code settings and themes (`settings::vscode`, `editor::settings`), GeoJSON (`maps::geojson`, `maps::choropleth`).
- Pictures decoded with `image` and QR frames read with `rqrr` (`media`, `misc::QrCodeScanner`).
- Terminal output and ANSI escape codes (`terminal`, `typography` `AnsiText` via vte).
- File names and paths from drops, archives and folder listings (`forms::files`, `files`, `lists::Folder::of`).

## Components that matter most / least
- Most: parsers and anything that turns external bytes into layout (markdown, JSON/GeoJSON, ANSI, images, QR, archive paths), and the few `unsafe` blocks that call AppKit (`shell::native`, `buttons::share`, `forms::color::dropper`).
- Less: pure drawing and theme code. `crates/ely-gallery/`, `frontend/` (the website), `compat/` and `scripts/` are out of scope.

## How to exercise it
`cargo test --lib --features test-support` runs the suite headless on gpui's test platform. Parsers can be driven directly from unit tests.

## How you rate severity
- Memory unsafety in an `unsafe` block, or a path traversal that reaches outside a chosen folder: high.
- A panic, unbounded allocation or hang reachable from untrusted input: medium (it takes down the host app).
- Wrong rendering with no security effect: out of scope.

## Anything to leave alone
- Panics from asserts on a caller's own arguments are by design (fail fast). Report only panics reachable from runtime data.
- The macOS-only code (AppKit, wry WebView) does not build in this Linux image.
