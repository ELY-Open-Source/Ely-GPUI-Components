# Changelog

What changes for an app built on Ely, newest first. An entry names the crates it touches. Versions follow semver: below 1.0, a breaking change raises the minor version, and CI's `semver` job fails one that does not.

## Unreleased

- `ely-core`: adds the Spanish `es-ES` locale and Spanish component messages.

- `ely-documents`: in `MarkdownDocument`'s visual view, F6 and Shift+F6 move focus to the nearest stop outside the document, so a keyboard user leaves a long document in one key; they had stepped block to block as Tab does. With nothing outside, focus stays where it was.
- `ely-core`: a `Banner`'s action carries the debug selector `banner-action {label}`, so a test can press it.
- `ely-core`: each box of a `CheckboxGroup` carries the debug selector `check {value}`, as `ToggleGroup`'s buttons carry `toggle {value}`, so a test can press one box by its value.
- `ely-core`: a `Scrollbar` over a `ListState` reads the list's attainable end from its measured tail, so a held drag turned back from the bottom moves at once, and returning from the end keeps the thumb's size and range; an unmeasured gap had shrunk the range to the items measured.
- `ely-documents`: on macOS, a formula or diagram letter only the system's Last Resort font draws counts as having no font, so `formula_svg` and `diagram_svg` refuse it instead of writing Last Resort's placeholder box.
- `ely-documents`: a `TemplatePicker` as narrow as a `Dialog` gives its list at most half, so the preview's name and Use button both show; narrower still, the name is cut with an ellipsis, never the button. The button had been squeezed over the name.
- `ely-documents`: in `MarkdownDocument`'s visual view, a drag or Shift+Up/Down past the open block opens the blocks it crosses as one field, selected across them; copy, cut, delete and typing take the whole selection, written back as one undo step. A drag past the view's top or bottom scrolls it.
- `ely-documents`: bold, italic, strikethrough, underline and inline code over a selection that crosses blocks apply in each paragraph, heading or cell it touches; when all are styled, all lose it. Link and picture still refuse with `EditRefusal::Across`.
- `ely-core`: `TextInput::select_from(anchor, head)` selects in either direction, the caret at `head`.
- All crates: Rust 1.98 at least, the floor gpui 0.3.10 builds on, since its macOS crate calls `Option::map_or_default`; 1.95 had been declared, and failed on macOS.
- `ely-core`: a `FormField` names the text field it frames for screen readers, its label as the name and its error or description as the description, unless the field's owner named it.
- `ely-core`: a `Dialog` without `focus_first` focuses its first control a frame after it opens; `Dialog`, `ActionSheet`, `Lightbox`, `Spotlight` and `Tour` name the element that takes their focus. `FocusScope::node` names a scope.

## 2026-10-26: `ely-core` 0.2.0, every other crate 0.1.1

The first release on crates.io.

- 31 library crates: `ely-core` (chapters 1–14, theme, motion, i18n), one crate per vertical chapter from `ely-charts` to `ely-rendering`, `ely-desktop` (tray, Dock badge, Omarchy), and `ely-gpui-component`, which re-exports them all under their chapter names.
- gpui, gpui_platform and gpui_web from `gpui-pre` `=0.3.10`, the snapshot GPUI Kit 0.8.1 pins, so Ely and Kit share a window.
- `ely-core` 0.2.0: gpui_platform moves behind the `platform` feature, off by default, so a Button-only app links no gpui backend Ely chose; `runtime_shaders` turns it on, and `ely-gpui-component` keeps both on. `init` fails on a platform with no text system, naming the features that give one: `font-kit` on macOS, `wayland` and `x11` on Linux.
- Every public item documented, each component with an example that compiles and runs; `api/` holds each crate's public API.
- Every focus stop has a role and a name for screen readers; `gallery --audit` walks them on Windows.
- Messages in English, Chinese, German, French, Japanese and Hebrew; the last four machine translated, awaiting native review.
- Icons link by use and fonts by feature (`fonts`, on by default); fonts embed with `include_bytes!`, and `Combobox::languages` reads Ely's own ISO 639-1 table, so neither `rust-embed` nor `isolang` is a dependency.
