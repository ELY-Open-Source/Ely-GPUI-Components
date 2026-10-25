# 0007 Layouts persist as versioned JSON

Date: 2026-09-25. Status: accepted.

Layouts persist as versioned JSON through serde. Unknown fields are ignored. Newer versions, unknown panels and bad pane trees are refused, and restore is all or nothing.
