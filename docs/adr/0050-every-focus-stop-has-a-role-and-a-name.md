# 0050 Every focus stop has a role and a name

Date: 2026-10-24. Status: accepted.

Every focus stop has a role and a name (OPTIMIZATION.md O-19, D4): `gallery --audit` walks 2,272 stops on 44 pages with none failing, from 846 faults in its first, shorter walk. Names need small API: `label` on `Knob`, `HotkeyInput`, `InlineEdit`, `GradientEditor`, `ListBox`, `MultiSelect`, `Rating`, `Slider`, `RangeSlider`, `UnitInput`, `PinInput` and `KanbanCard`, `name` on `Checkbox`, a name argument on `media_button`; `DashboardGrid::card` takes a `DashboardCard`, whose title names its tile; `ReactionPicker` opens from an `IconButton` with a tooltip. `SearchFilters` names the fields it is handed by the words it draws above them. The VoiceOver walk waits for the owner.
