# 0047 The verticals' public surface

Date: 2026-10-23. Status: accepted.

The verticals' public surface (C2): 59 items opened only for sibling crates carry `#[doc(hidden)]`; 28 no path reaches turn `pub(crate)`. A crate's `bind_keys` stays shown, since an app that takes `ely-editor`, `ely-terminal` or `ely-documents` without the umbrella crate calls it after `ely_core::init`.
