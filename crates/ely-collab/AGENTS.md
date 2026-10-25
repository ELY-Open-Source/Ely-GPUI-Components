# AGENTS.md: ely-collab

Rules for `ely-collab`, on top of the root `AGENTS.md` and `crates/ely-core/AGENTS.md`.

## Rules

- A mark over a text field reads the field at paint time, from a canvas laid over it (`collab::RemoteCursor`): by then `TextInput::bounds_for` holds this frame's layout, and a still page never renders again to catch up.
