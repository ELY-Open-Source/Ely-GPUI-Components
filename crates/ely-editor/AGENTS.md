# AGENTS.md: ely-editor

Rules for `ely-editor`, on top of the root `AGENTS.md` and `crates/ely-core/AGENTS.md`.

## Rules

- A find box is `editor::FindWidget`, over code, a terminal or a document's pages. It shows the replace fold and the option toggles only when the owner handles them.
- Editor rows read their text through `CodeEditor::row_text`: a multi-line ghost cuts its line, and the rest follows the ghost's last line.
