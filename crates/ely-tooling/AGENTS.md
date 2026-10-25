# AGENTS.md: ely-tooling

Rules for `ely-tooling`, on top of the root `AGENTS.md` and `crates/ely-core/AGENTS.md`.

## Rules

- gpui's inspector (debug builds) runs no mouse listener while picking and holds a pick on a press without a redraw, so Ely's panel requests a frame each render while picking (`tooling::install_inspector`). Its panel scrolls, so the box model it draws over the page goes through `primitives::raise`, which paints unclipped. The panel is a second root, out of the app's scope, so it owns Tab through a `FocusScope` of its own.
- A tool that measures redraws cannot move: any animation redraws the view it counts (`tooling::RenderCounter`). `AnyView::cached` renders again only when it or a view inside it notifies, its bounds, clip or text style change, or the window refreshes; gpui refreshes the window whenever input turns between the keyboard and the pointer. A meter that counts frames keeps its view drawing while shown (`tooling::FpsMeter`).
- A logger of input lays its canvas before what it holds, so its capture listeners run first and see what a child stops. Key downs come through `App::intercept_keystrokes`, which runs before bindings, and a key context on its box tells it focus is inside (`tooling::EventLogger`). It also hears a lone modifier's release as a keystroke, which the logger skips.
