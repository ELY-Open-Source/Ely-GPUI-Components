# 0049 Containers

Date: 2026-10-23. Status: accepted.

Containers (OPTIMIZATION.md O-12): 87 components implement `ParentElement`, listed in `src/containers.rs`. Seven that collected children under their own adder (`SplitPane` and `ScrollSync` panes, `MultiChartLayout` cells, `FieldArray` and `SettingsSection` rows, `FlowChart` and `InfiniteCanvas` layers) take them through `ParentElement` too; `MapPopup`'s single `child` and `DashboardCard`'s `body` become children, as `Card`'s are.
