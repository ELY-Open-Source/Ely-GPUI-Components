# Theme

One global `Theme` holds the mode, the palette every component paints with, and the tokens for size, radius and shadow. Read it with `cx.theme()`; change it through `Theme`'s functions, and every window repaints.

## Read the theme

Bring `ActiveTheme` into scope and call `cx.theme()` on `App`, or on a `Context`, which derefs to `App`.

| Item | Holds |
| --- | --- |
| `colors` | The `Palette`: 36 named colors (`bg`, `surface`, `fg`, `fg_muted`, `accent`, `border`, `focus`, `danger` and the rest), with `chart`, `ansi` and `syntax`. |
| `text_size(TextSize)` | `Xs` 11px to `Display` 32px, times `font_scale`. |
| `radius(Radius)` | `Sm` 4px, `Md` 6px, `Lg` 8px, `Xl` 12px, times `radius_scale`. |
| `elevation(Elevation)` | Shadows for `Raised`, `Floating` and `Modal`; darker in dark mode. |
| `control_height(ControlSize)` | `Sm` 24px, `Md` 28px, `Lg` 32px; 4px less under `Density::Compact`, 4px more under `Comfortable`. |
| `mode()`, `is_dark()`, `high_contrast()`, `color_blind_safe()` | The current mode and switches. |

Sizes come back as `Rems`, so they follow the window's rem size.

## Change the theme

| Call | Does |
| --- | --- |
| `Theme::set_mode(mode, cx)` | Cross-fades the palette to `Mode::Light` or `Mode::Dark`. |
| `Theme::set_mode_now(mode, cx)` | Switches at once. Call it before a window opens to set its first frame. |
| `Theme::set_high_contrast(on, cx)` | Muted text and borders move further from the page, and glass turns opaque, in Ely's palettes. |
| `Theme::set_color_blind_safe(on, cx)` | Chart hues that stay apart under protanopia, deuteranopia and tritanopia. |
| `Theme::set_palette(mode, palette, cx)` | An owner's palette for one mode; `None` brings Ely's back. |
| `Theme::update(cx, edit)` | Edits the other fields: `density`, `font_scale`, `radius_scale`, `font_family`, `mono_family`, `reduced_motion`, `platform`. |

The color calls log at the `info` level, so a theme change shows in the app's log.

## An owner's palette

`cx.theme().palette()` returns the palette the shown mode fades to. Change a color by its field's name with `token_mut`, then hand the palette back for that mode. A name the palette lacks panics, naming it. High contrast and color-blind safe charts change Ely's own palettes, never an owner's.

## Chart hues

`colors.chart` holds eight hues. `HUE_NAMES` names them in order: Blue, Teal, Ochre, Rose, Violet, Green, Rust, Cyan. `colors.hue(index, owner)` returns one and panics past the eighth, naming `owner`, so an event or a label can wear the same hue a chart does.

## Omarchy

On Omarchy, Linux, an app can wear the system's theme. Every component paints from the palette, so all of them follow.

```rust
use ely_gpui_component::desktop::{follow_omarchy, omarchy_current_dir};

if let Some(current) = omarchy_current_dir() {
    follow_omarchy(current, cx)?;
}
```

| Call | Does |
| --- | --- |
| `omarchy_current_dir()` | `~/.local/state/omarchy/current`, where Omarchy keeps its theme whatever `XDG_STATE_HOME` says; `None` without Omarchy. |
| `follow_omarchy(dir, cx)` | Wears the theme now, on its own mode, then each switch once the folder is quiet for 250 ms. Ely's palette stands in for the other mode. Fails while following already, or on a folder it cannot watch. |
| `stop_omarchy(cx)` | Gives both modes Ely's palettes back. The mode stays the app's to set. |
| `read_omarchy_theme(dir)` | Reads the theme once, as an `OmarchyTheme`: its name, mode, palette and the hues it lacks. |

The reader resolves `theme/colors.toml` as Omarchy's `omarchy-theme-color` does. Colors move toward white or black until they pass `devtools::audit`, the pairs `A11yChecker` shows: body text 4.5:1 on every surface, links on the page, surface and sunken, code on the surface, sunken and hover, a status color on the page and its own tint, the accent's text on the accent and a tooltip's on the tooltip; the focus ring 3:1 on the page and surface, chart hues on the page. A theme no move can fix is refused. The terminal's sixteen colors are the ones Omarchy gives Ghostty. Omarchy's opaque selection becomes the clearest wash that shows the same color over the page, since Ely also draws a selection over content. A theme that cannot be read logs an error and the colors stay; a theme without `colors.toml` brings Ely's palettes. `follow_omarchy` and `stop_omarchy` watch files, so they are absent on the web.

## A card made of tokens

The example draws a card from tokens alone, with buttons that turn the mode, the contrast, the density and the accent.

```rust example=theme
```
