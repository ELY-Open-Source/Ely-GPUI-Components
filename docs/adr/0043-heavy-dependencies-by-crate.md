# 0043 Heavy dependencies by crate

Date: 2026-10-23. Status: accepted.

Heavy dependencies go by crate (OPTIMIZATION.md O-02): an app that draws a Button takes `ely-core`, which pulls none of ratex-svg, mermaid-rs-renderer, alacritty_terminal, rqrr or notify; `isolang` sits behind `ely-core`'s `languages`, off by default. `scripts/check.sh` holds the six. `zbus` stays, through gpui's Linux backend, which any gpui window on Linux links.
