# Stability

Every published crate is unstable until this table says otherwise. A crate turns stable only when:

- the owner has read its public API, `api/<crate>.txt` at a commit named by its hash, and signed a record `tasks/signoffs/<crate>.md` written from `tasks/signoffs/TEMPLATE.md`, its verdict `stable`;
- a vertical crate, one past `ely-core`, also links at least one outside user's feedback, the address of an issue or a discussion, from someone who is not the owner or a model;
- `ely-gpui-component` turns stable only after every crate it re-exports.

A model never writes a sign-off record and never marks a crate stable. A crate at 1.0 or above must be stable here. `scripts/stability.py`, which `scripts/check.sh` runs, holds this table to every rule above.

| Crate | Status | Sign-off | Feedback |
|-------|--------|----------|----------|
| ely-account | unstable | | |
| ely-agent | unstable | | |
| ely-calendar | unstable | | |
| ely-canvas | unstable | | |
| ely-charts | unstable | | |
| ely-chat | unstable | | |
| ely-collab | unstable | | |
| ely-core | unstable | | |
| ely-dashboard | unstable | | |
| ely-debug | unstable | | |
| ely-desktop | unstable | | |
| ely-devtools | unstable | | |
| ely-documents | unstable | | |
| ely-editor | unstable | | |
| ely-files | unstable | | |
| ely-finance | unstable | | |
| ely-generative | unstable | | |
| ely-git | unstable | | |
| ely-gpui-component | unstable | | |
| ely-interaction | unstable | | |
| ely-mail | unstable | | |
| ely-maps | unstable | | |
| ely-media | unstable | | |
| ely-messaging | unstable | | |
| ely-misc | unstable | | |
| ely-onboarding | unstable | | |
| ely-project | unstable | | |
| ely-rendering | unstable | | |
| ely-settings | unstable | | |
| ely-terminal | unstable | | |
| ely-tooling | unstable | | |
