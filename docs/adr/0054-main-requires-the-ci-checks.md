# 0054 Main requires the CI checks

Date: 2026-10-26. Status: accepted.

At the owner's word, `main` is protected: a pull request merges only once `gate` on Linux, Windows and macOS, `api`, `semver`, `deny`, and `msrv` and `pixels` on all three pass (OPTIMIZATION.md O-18, O-24, O-25). Force pushes and deletion are refused. Admins are not held to it, so the owner's direct pushes land and CI judges them after; `bench` and `site` stay advisory. Checks need not run on the latest `main`, since a pixel leg takes over an hour.
