# Installation

## Requirements

- Rust 1.98 or later, the version Ely's `Cargo.toml` names.
- macOS, Windows or Linux. The crate's tests run on all three; on Linux, GPUI needs the `wayland` and `x11` features below.
- No full Xcode: GPUI compiles its Metal shaders at run time.

## Add the dependencies

Ely is not on crates.io. Depend on it by git, with GPUI at the snapshot Ely pins. `gpui-pre` republishes Zed's GPUI on crates.io, and GPUI Kit pins the same one:

```toml
[dependencies]
ely-gpui-component = { git = "https://github.com/ELY-Open-Source/Ely-GPUI-Components" }
gpui = { package = "gpui-pre", version = "=0.3.10" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.10", features = ["font-kit", "wayland", "x11"] }
```

| Line | Why |
| --- | --- |
| `ely-gpui-component` | The library. `Cargo.lock` records the commit; add `rev = "<commit>"` to hold it in `Cargo.toml` as well. |
| `gpui` | The same snapshot as Ely's, under the name `gpui`. Another builds a second GPUI, and its types do not match the ones Ely takes. |
| `gpui_platform` | Opens windows on each platform. Without `font-kit`, macOS falls back to GPUI's no-op text system and draws no text. Without `wayland` and `x11`, Linux runs headless and opens no window; building them needs `libxkbcommon-x11-dev` and `libfontconfig-dev`. |

## Shaders

`ely-gpui-component`'s default feature, `runtime_shaders`, turns on the same feature in `gpui_platform`, so GPUI compiles its Metal shaders at run time. With `default-features = false` the build runs `xcrun metal` instead, which needs Xcode's Metal compiler. `ely-core` alone turns on no `gpui_platform` feature: its `platform` feature, off by default, turns on `font-kit`, `wayland` and `x11`, or the app sets them in its own `gpui_platform` line, as above.

## Check the install

Copy the [first window](/docs/#a-first-window) into `src/main.rs` and run it:

```sh
cargo run
```

A window opens with a counter and a Press button. From a clone of the repository, `cargo run -p ely-gallery` opens every chapter.

## When it fails

| Symptom | Cause and fix |
| --- | --- |
| `init` error: `ely: the asset source lacks N of Ely's files, first <path>: <reason>` | The app's source cannot load every file Ely bundles. Pass `Assets`, or `Assets::before(yours)` when the app has a source of its own. |
| `init` error: `ely: embedded fonts failed to register` | GPUI's text system refused Ely's fonts. File a bug with the log. |
| `init` error: `ely: the platform has no text system` | `gpui_platform` lacks `font-kit` on macOS, or `wayland` and `x11` on Linux. `ely-core`'s `platform` feature turns on all three. |
| No window opens on Linux, and the log says nothing failed | `gpui_platform` lacks the `wayland` and `x11` features. |
| `mismatched types`, with the note ``there are multiple different versions of crate `gpui` in the dependency graph`` | `gpui` names another snapshot or a git source. Use `gpui-pre` at `=0.3.10`. |
| `metal shader compilation failed` | Default features are off and Xcode's Metal compiler is missing. Keep `runtime_shaders`. |
