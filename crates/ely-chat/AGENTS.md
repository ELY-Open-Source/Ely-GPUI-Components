# AGENTS.md: ely-chat

Rules for `ely-chat`, on top of the root `AGENTS.md` and `crates/ely-core/AGENTS.md`.

## Rules

- A chat holds at its newest message through gpui's `list` with `ListAlignment::Bottom`: it sticks while scrolled to the end, and `scroll_to` one past the last item sticks it again; that call sends no scroll event, so the owner clears its own away state (`chat::MessageList`).
- A search step, a tool call or an agent's step says where it stands with `chat::StepState` and draws it with `chat::step_mark`.
