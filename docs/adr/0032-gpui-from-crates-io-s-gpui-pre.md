# 0032 gpui from crates.io's gpui-pre

Date: 2026-10-07. Status: accepted.

Gpui comes from crates.io's `gpui-pre` at the exact snapshot GPUI Kit pins, not from Zed's git. Two sources build two gpui crates whose types do not mix, so Ely and Kit could not share a window. The code is still Zed's; the publisher is not. This replaces the 2026-09-28 choice of Zed's git, and the pin moves when Kit's does.
