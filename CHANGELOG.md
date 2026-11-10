# Changelog

What changes for an app built on Ely, newest first. An entry names the crates it touches. Versions follow semver: below 1.0, a breaking change raises the minor version, and CI's `semver` job fails one that does not.

## Unreleased

- `ely-documents`: `MarkdownDocument::replace` writes the open block back and closes it when the replacement moves that block's bounds without touching it: the blank line before it removed, a code fence opened above, a row added above the open cell. The field no longer stays open over text that is no block, or on another row.
- `ely-documents`: `ShortcodeRun` reads a block's `:name:` emoji as `MarkdownReader` does across inline HTML too: a tag or comment starts a new text, an open `<code>` keeps its text as written. An owner writing its own HTML from `markdown_tree` holds one per block.
- `ely-core`: a release build's `init` no longer lists every font family to prove the platform has a text system; on a Mac that list cost 50 ms of every start. A debug build still checks, and fails with the same message.
- `ely-documents`: `MarkdownDocument::replace` leaves an open block open when no edit reaches into it: the block moves with the edits before it, and typing goes on at its caret. An edit that reaches into the open block still writes it back and closes it. Before, any replacement closed the block and the keys that followed were lost.
- `ely-documents`: `Shortcodes` is public, so an owner writing its own HTML from `markdown_tree` reads `:name:` emoji as `MarkdownReader` does: one value per DOM text node, `split` at every element, `read` on its text.
- `ely-documents`: `$$` math reads as GitHub pairs it: `$$` alone then `a.$$` on the next line is math and no longer takes the `$$` into the formula or the next block into it; `$$a` then `$$` is math, no longer an empty block; an unclosed `$$` is text.
- `ely-documents`: `MarkdownDocument::point_match` shows a match as `show_match` does, in visual mode its block opened with the match selected, but leaves focus where it is, so a find box keeps its keys and Enter steps on.
- `ely-documents`: `MarkdownRenderer::selectable` lets the pointer select its text across blocks, paragraphs, headings, lists, quotes, table cells and code; the secondary C copies it, blocks joined by line ends and list items led by their marker (`3.`, `•`, `[x]`, two spaces per depth), and A selects all while it holds focus. A right press hands a `RightPress` with the selection and the link or picture under it to the owner, which builds its menu. New text drops the selection. Inline math and front matter stay unselectable. `HeldPicture::format` and `HeldPicture::bytes` read a held picture back, so an owner can copy or save it.
- All crates: gpui, gpui_platform and gpui_web from `gpui-pre` `=0.3.11`, the snapshot GPUI Kit 0.9.0 pins, so Ely and Kit still share a window; an app on Ely moves its own `gpui-pre` pins to `=0.3.11`. A breaking change, so each published crate's minor rises: `ely-core` 0.3.0, every other 0.2.0.
- `ely-gallery`: `--audit` also opens a `SearchableMenu`, arrows to a button checking a screen reader hears each marked row, and presses it through UI Automation's Invoke: it must run once and close the menu.
- `ely-core`: a `PromptDialog`'s field takes the dialog's label as its accessible name, and a failed check's reason as its description, so a screen reader names the field.
- `ely-core`: Shell code colors a `#` right after the `)` that closes `$(`, `$((`, `<(` or `>(` as part of the word, as Bash reads it; only a bare subshell's `)` ends a word before a comment. A `case` pattern's `)` inside `$(` no longer counts as the `)` that closes it; `case` and `esac` are keywords only as whole words where Bash takes a reserved word, so not as a pattern, and also after `fi`, `done`, `esac` or `}`.
- `ely-core`: a menu's icon buttons carry no selected state, so Windows offers their Invoke pattern to a screen reader; in a `SearchableMenu`, the arrows lend the panel focus so a reader hears the marked row or button, and typing or Backspace gives the filter focus back.
- `ely-core`: adds the Korean `ko-KR` locale and Korean component messages. The gallery's i18n page holds one catalog per listed locale, Spanish and Korean too; its table is sized to `LOCALES`, so a new locale without one does not build.
- `ely-documents`: a wheel over `MarkdownDocument`'s margins beside a narrowed text column, or over its bar's track, scrolls the text as far as one over the text, once; in the source view a line wheel keeps the code's own line height at any text size, and a sideways wheel (Shift and the wheel) scrolls the code down there as over the code. Split is unchanged.
- `ely-editor` 0.2.0: `Metrics` holds `wheel`, one wheel line as the code's list scrolls it, so a struct literal of `Metrics` names it.
- `ely-core`: `TreeDrag` is public, with `key` and `label`, so a drop past a `Tree`'s rows names the dragged node.
- `ely-core`: `SegmentedControl::on_context` hears a right press on a segment, with its value and the pointer, as `Tabs::on_context` does; the press stops on the segment, so the focus stays where it was.
- `ely-core`: adds the Vietnamese `vi-VN` locale and Vietnamese component messages; the gallery's i18n page holds a Vietnamese catalog.
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
