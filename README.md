# Ely GPUI Component

A component library for [GPUI](https://www.gpui.rs), the Rust UI framework behind Zed.
Every component ships in light and dark. The palette is warm and quiet. Color carries meaning, not decoration.

Every component runs live at https://elygpui.com, compiled to WebAssembly.

Status: early. `TASKS.md` tracks every component, chapter by chapter. To help, see `CONTRIBUTING.md`.

## Use

```toml
[dependencies]
ely-gpui-component = { git = "https://github.com/ZacharyZhang-NY/Ely-GPUI-Components" }
gpui = { git = "https://github.com/zed-industries/zed", rev = "1a28cff4b409169bac058bca40dfbfeb7621d19b" }
gpui_platform = { git = "https://github.com/zed-industries/zed", rev = "1a28cff4b409169bac058bca40dfbfeb7621d19b", features = ["font-kit"] }
```

```rust
use ely_gpui_component::{Assets, theme::{Mode, Theme}};
use gpui::App;

fn main() {
    gpui_platform::application().with_assets(Assets).run(|cx: &mut App| {
        ely_gpui_component::init(cx);
        Theme::set_mode(Mode::Dark, cx);
    });
}
```

`init` registers the fonts and the theme. It panics if `Assets` is not wired into the application.

## Gallery

```sh
cargo run --example gallery
cargo run --example gallery -- --page buttons
cargo run --example gallery -- --capture shots   # macOS: PNG of every page, light and dark
ELY_GALLERY_ASSETS=https://example.com/gallery/assets/ scripts/web.sh dist   # for the browser
```

In a browser, `?page=buttons&story=icon-button&theme=dark` draws one section alone; `examples/gallery/stories.json` lists them all.

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

## Build notes

- gpui compiles its Metal shaders at runtime here (`runtime_shaders`, on by default), so a full Xcode install is not required. gpui_platform needs `font-kit`, or macOS draws no text.
- CI tests the crate and builds the gallery on macOS, Windows and Linux, arm64 and x86_64. The capture tool needs macOS.

## License

MIT or Apache-2.0, at your option. Lucide icons: ISC. Inter, JetBrains Mono, IBM Plex Sans and the gallery's Noto Sans Hebrew: SIL Open Font License 1.1.
The gallery photos in `examples/gallery/assets` were generated for this project with gpt-image-2.
