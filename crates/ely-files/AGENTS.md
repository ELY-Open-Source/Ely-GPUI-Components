# AGENTS.md: ely-files

Rules for `ely-files`, on top of the root `AGENTS.md` and `crates/ely-core/AGENTS.md`.

## Rules

- A file on its way is a `files::Transfer` drawn by one row in `files::transfer`: TransferQueue and DownloadManager share it, and its actions follow its state.
