# 0046 Core's public surface

Date: 2026-10-23. Status: accepted.

Core's public surface (OPTIMIZATION.md O-09, C1): the 135 items `ely-core` opened from `pub(crate)` only for its sibling crates carry `#[doc(hidden)]`, as do the text actions they re-export; four methods on crate-private types turn `pub(crate)`. rustdoc shows 1,004 item pages for core.
