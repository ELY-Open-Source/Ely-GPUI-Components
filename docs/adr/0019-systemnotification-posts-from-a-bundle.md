# 0019 SystemNotification posts from a bundle

Date: 2026-09-28. Status: accepted.

SystemNotification is `prove`: gpui's `show_system_notification` posts only from an app bundle; unbundled, as `cargo run` starts the gallery, it logs and drops the call, and the web posts nothing.
