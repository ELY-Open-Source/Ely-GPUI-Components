# 0060 Semver against releases

Date: 2026-10-26. Status: accepted.

`scripts/semver.py` compares each crate with its newest release on crates.io, cargo-semver-checks' own default, not with the base commit. The base commit stood in for crates.io while nothing was published; once all 31 crates were (O-13), it failed a second break inside one raised, unreleased version: removing `languages` from `ely-core` 0.2.0 after B7 had already raised it to 0.2.0 failed CI at `d77a171`, though the release carries both changes under the one raise. The base commit still picks which crates may have changed, so an untouched crate costs nothing; a crate with no release has nothing to compare.
