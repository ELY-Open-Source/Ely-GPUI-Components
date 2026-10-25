# AGENTS.md: kit

Ely beside GPUI Kit.

## Commands

- `cargo test --manifest-path compat/kit/Cargo.toml --target-dir target/kit` runs Ely beside GPUI Kit in one window: Tab through both, each library's dialog over the other's fields, one asset source. `scripts/check.sh` first resolves it afresh and holds `gpui-kit` to the newest release `cargo search` names, so a Kit release that moves its gpui pin fails resolution there, naming `gpui-pre`; Ely's pin then follows. Its lockfile stays out of git.
