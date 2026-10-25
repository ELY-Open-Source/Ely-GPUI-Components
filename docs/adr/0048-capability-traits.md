# 0048 Capability traits

Date: 2026-10-23. Status: accepted.

Capability traits (OPTIMIZATION.md O-11) in Ely's own words, after GPUI Kit's `Disableable` and `Selectable` (`gpui-base` 0.8) and Longbridge's `Sizable`: `Sizable::size(ControlSize)`, `Disableable::disabled(bool)`, `Selectable::{selected, is_selected}`, `Changeable::on_change` with a `Value<'a>` its handler takes. Components keep their own methods, so no caller changes; 30 size, 19 disabled, 5 selected and 80 change impls. A subscription card's `on_change` is a press, not a value, and has none.
