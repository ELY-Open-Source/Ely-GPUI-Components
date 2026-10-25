# 0044 Icons by use, fonts by feature

Date: 2026-10-23. Status: accepted.

Icons embed by use, fonts by feature (OPTIMIZATION.md O-04). `IconName` turns from an enum into a type of named consts, `IconName::Check` as before, each pointing at its SVG's own static; `path()` leaves, `svg()` and `name()` stay, and the SVGs leave the asset source, so a host's own `icons/x.svg` is its own, GPUI Kit's too. A five-icon release binary holds those five SVGs and no other; without `fonts` it holds Plex alone and is 2.2 MB smaller.
