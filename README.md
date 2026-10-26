# Ely GPUI Component

A component library for [GPUI](https://www.gpui.rs), the Rust UI framework behind Zed.
Every component ships in light and dark. The palette is warm and quiet. Color carries meaning, not decoration.

Every component runs live at https://elygpui.com, compiled to WebAssembly.

Status: early. `TASKS.md` tracks every component, chapter by chapter. To help, see `CONTRIBUTING.md`.

## Use

```toml
[dependencies]
ely-gpui-component = { git = "https://github.com/ELY-Open-Source/Ely-GPUI-Components" }
gpui = { package = "gpui-pre", version = "=0.3.10" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.10", features = ["font-kit", "wayland", "x11"] }
```

```rust
use ely_gpui_component::{Assets, theme::{Mode, Theme}};
use gpui::App;

fn main() {
    gpui_platform::application().with_assets(Assets).run(|cx: &mut App| {
        ely_gpui_component::init(cx).expect("Ely failed to start");
        Theme::set_mode(Mode::Dark, cx);
    });
}
```

`init` registers the fonts and the theme. It returns an error if the application's asset source cannot load one of Ely's files.

## With GPUI Kit

Ely runs beside Longbridge's [GPUI Kit](https://gpui-kit.com) in one window, on the GPUI snapshot Kit 0.8.1 pins. Pass `Assets::before(gpui_kit::assets::Assets)` as the one asset source, call both `init`s, open the window with `gpui_kit::open_window`, and wrap the view in Ely's root `FocusScope`. The [guide](https://elygpui.com/docs/gpui-kit) covers themes, focus and the inspector; `compat/kit` holds a whole app and the tests behind each claim.

## On Omarchy

`desktop::follow_omarchy(dir, cx)` dresses every component in Omarchy's current theme and follows each switch; `desktop::omarchy_current_dir()` finds the folder, and `stop_omarchy` gives Ely's palettes back. The [theme guide](https://elygpui.com/docs/theme) says how the colors map.

## Gallery

```sh
cargo run -p ely-gallery
cargo run -p ely-gallery -- --page buttons
cargo run -p ely-gallery -- --capture shots   # macOS: PNG of every page, light and dark
ELY_GALLERY_ASSETS=https://example.com/gallery/assets/ scripts/web.sh dist   # for the browser
```

In a browser, `?page=buttons&story=icon-button&theme=dark` draws one section alone; `crates/ely-gallery/stories.json` lists them all.

## Website

```sh
cd frontend
pnpm install
ELY_GALLERY_ASSETS=https://<site>/gallery/assets/ pnpm gallery   # the wasm gallery, into public/gallery
pnpm dev        # or: pnpm build && pnpm preview; both fetch the showcase at showcase.lock's commit
ELY_SHOWCASE=../../Ely-GPUI-Showcases pnpm dev   # the showcase from a local checkout
node scripts/og.mjs   # with pnpm preview up: public/og.png from the hero
node scripts/favicon.mjs   # public/favicon.svg, apple-touch-icon.png and ../logo.png from the mark
pnpm deploy     # Cloudflare, through wrangler
```

## How it is built

Ely is built by AI coding agents. A person sets each goal and decides what ships; agents write the code, its tests and these docs, and other agents review them.

- Work runs as task items. One agent writes an item and another model reviews it, round after round, up to five; every round is in [`tasks/reviews.md`](tasks/reviews.md). Some items end on a FAIL row there: their last fixes went in after the cap, unreviewed. These reviews are models reading models; no person has signed a crate yet.
- No crate is stable until the owner signs its public API and, past `ely-core`, an outside user has given feedback. [`tasks/stability.md`](tasks/stability.md) keeps the ledger; today no crate is stable.
- [`scripts/check.sh`](scripts/check.sh) gates every change: formatting, lints, tests, the docs build, the web lint, the panic list and the house rules. CI runs it on Linux, Windows and macOS, beside checks on the public API, semver, licenses, the minimum Rust and pixels.
- Each `expect`, `panic!` and `unreachable!` in library code is listed with its reason in [`scripts/panics.tsv`](scripts/panics.tsv); [`tasks/panics.md`](tasks/panics.md) records how each chapter moved to failing visibly instead.
- The rules the agents follow are in [`AGENTS.md`](AGENTS.md) and each crate's own; decisions are in [`docs/adr/`](docs/adr/README.md).

## Build notes

- gpui compiles its Metal shaders at runtime here (`runtime_shaders`, on by default in `ely-gpui-component`), so a full Xcode install is not required. `ely-core` alone turns on no gpui backend; the app's own gpui_platform does. gpui_platform needs `font-kit`, or macOS draws no text, and `wayland` and `x11`, or Linux opens no window.
- CI runs `scripts/check.sh` and builds the gallery on GitHub-hosted Linux, Windows and macOS. The capture tool shoots pages on all three; only macOS plays the scripted states.

## License

MIT. Lucide icons: ISC. Inter, JetBrains Mono, IBM Plex Sans and the gallery's Noto Sans Hebrew: SIL Open Font License 1.1.
The gallery photos in `crates/ely-gallery/assets` were generated for this project with gpt-image-2.
