# 0055 Publish to crates.io

Date: 2026-10-26. Status: accepted.

At the owner's word, the 31 library crates go to crates.io at their versions, `ely-core` first, then each crate after the crates it needs. Every crate stays unstable (`tasks/stability.md`); its 0.x version says so. Publishing needs a crates.io token on the publishing machine, which only the owner can make; until it is there, nothing is published.
