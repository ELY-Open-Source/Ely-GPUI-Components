# Icons, fonts and assets

`Assets` embeds Ely's fonts in the binary. Pass it to `with_assets`, or `Assets::before(other)` to serve Ely's files first and every other path from another source; `init` checks that each of Ely's files loads through the app's source, then registers the fonts. Icons travel in the code, not the source.

## Icons

Ely bundles 307 Lucide 1.48.0 icons, each an `IconName` const: `IconName::Check` carries `check.svg`, `svg()` returns its bytes and `name()` the Lucide name, `check`. `IconName::ALL` lists them all. Each SVG sits in a static of its own, so a release binary holds only the icons its code names.

| Call | Does |
| --- | --- |
| `Icon::new(name)` | Draws the icon at `IconSize::Md` in the theme's `fg`. |
| `Icon::from_path(path)` | Draws an SVG from the application's asset source with the same defaults. |
| `.size(IconSize::Xl)` | `Xs` to `Xxl`, from the theme's icon sizes. |
| `.color(color)` | Any color; components pass theme colors. |
| `.rotate(Radians(..))` | Turns the glyph inside its box. |
| `.scale(factor)` | Draws the glyph larger or smaller in the same box. |

GPUI's `svg()` paints with its own text color alone, so `Icon` always sets one.

## Adding an icon

From a clone of the repository:

1. `scripts/icons.sh <lucide-name>...` fetches Lucide 1.48.0 with npm and copies each SVG into `crates/ely-core/icons/`.
2. Add a line to the `icons!` list in `crates/ely-core/src/primitives/icon/names.rs`, such as `Rocket => "rocket",`.

## File icons

`FileIcon::file(name)` shows the icon a file's name suggests, and `FileIcon::folder(open)` a folder's. Ely maps extensions to icons: `rs` and `ts` to `FileCode`, `toml` to `FileCog`, `json` to `FileJson`, images, audio, video, archives and more. An app lays its own rules over that map:

| Call | Does |
| --- | --- |
| `IconTheme::default().name("Cargo.toml", IconName::Package)` | A file of that exact name, in any case. |
| `.extension("md", IconName::BookOpen)` | Files with that extension, given without its dot. |
| `.apply(cx)` | Serves every window from then on, and repaints. |

A name rule beats an extension rule, and both beat Ely's map.

## Fonts

`init` registers seven fonts from `Assets`: Inter Regular, Medium, SemiBold and Italic; JetBrains Mono Regular and Medium; IBM Plex Sans Regular. The theme draws text in Inter and code in JetBrains Mono, set by its `font_family` and `mono_family` fields. Inter and JetBrains Mono come with `ely-core`'s `fonts` feature, on by default; an app that turns it off registers its own fonts and sets those two fields. IBM Plex Sans always ships. On the web, GPUI answers the system UI font with IBM Plex Sans, and text inside SVGs falls back to it.

## An app's own assets

GPUI takes one asset source. An app with files of its own, or another library's, passes `Assets::before(theirs)`: Ely's paths load from Ely, every other path from `theirs`, whose errors pass through. `init` returns an error naming the first of Ely's files the source cannot load.

Use `Icon::from_path("app-icons/mark.svg")` to draw one of those SVG assets with Ely's themed sizing and color. It also accepts an owned `String` or `SharedString`. Paths are asset keys resolved by `AssetSource`; a path it lacks logs an error on first draw and shows a red broken-picture icon. SVGs render as single-color masks; `IconName` and `IconPicker` continue to list the bundled icons.

Compatible single-color SVGs can come from [Lucide](https://lucide.dev/), [Hugeicons](https://github.com/hugeicons/hugeicons), [Phosphor](https://github.com/phosphor-icons/core), [Remix Icon](https://github.com/Remix-Design/RemixIcon), or [Tabler](https://github.com/tabler/tabler-icons). The app supplies its chosen files and preserves their licenses; Ely does not download or register those catalogs.

The example below includes its asset source. Copy [`crates/ely-gallery/examples/assets/mark.svg`](https://github.com/ELY-Open-Source/Ely-GPUI-Components/blob/main/crates/ely-gallery/examples/assets/mark.svg) with it, or supply your own SVG.

## A window of icons

```rust example=icons
```
