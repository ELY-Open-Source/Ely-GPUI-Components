# AGENTS.md: ely-rendering

Rules for `ely-rendering`, on top of the root `AGENTS.md` and `crates/ely-core/AGENTS.md`.

## Rules

- Pixels a host draws at a box's size go through `rendering::RenderSurface`: it asks for one BGRA frame at the box's device pixels in prepaint, draws again only for a new size or revision, and frees the old frame and, on leave, the last one. The revision covers all the frame reads, theme colors too.
