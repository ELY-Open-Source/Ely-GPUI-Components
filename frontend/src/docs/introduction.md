# Introduction

Ely is a component library for [GPUI](https://www.gpui.rs), the Rust UI framework behind Zed. It holds 44 chapters, from primitives and forms to charts, a code editor, maps, a terminal and render surfaces. Every component draws in light and dark, and every one shows on the [components page](/components/): live, or as a native capture where a browser cannot run it.

The guides run in the order of the rail, each building on the one before.

## What Ely gives an app

- One theme for every component: light and dark palettes, high contrast, color-blind safe charts, and tokens for color, size, radius and shadow. Components read the theme, never literals.
- Keyboard first: `init` binds Tab and Shift-Tab, a root `FocusScope` keeps focus in the window, and every control that takes a press is a Tab stop with a focus ring.
- Motion that honors reduced motion: durations shorten to a millisecond and repeating motion holds still.
- The gallery, `cargo run -p ely-gallery`, shows every component on its chapter's page. The same gallery compiles to WebAssembly and runs the components page.

## What is stable

Ely is early. The library builds against GPUI from Zed's repository at one pinned revision, so its API moves when GPUI's does. It is not on crates.io; an app depends on it by git.

| Area | State |
| --- | --- |
| macOS | Tested. Every gallery page is captured in light and dark. |
| Windows and Linux | Builds are not tested yet. |
| The browser | The gallery runs through gpui_web on one thread, on WebGPU or WebGL2. |
| Terminal | Native only: its pseudo-terminal does not build for WebAssembly. |
| Tray icon, Dock badge, web view | macOS only. Elsewhere they return an error or say so. |
| Open entries | 8 of 1158 in `tasks/`: accessibility proofs, system notifications, pinch zoom and Windows' Mica. |

## A first window

The example opens a window with one counter and one Button. `cargo run -p ely-gallery --example docs_hello` runs it from a clone of the repository.

```rust example=hello
```

## License

MIT. The icons are Lucide's, under ISC. Inter, JetBrains Mono and IBM Plex Sans are under the SIL Open Font License 1.1.
