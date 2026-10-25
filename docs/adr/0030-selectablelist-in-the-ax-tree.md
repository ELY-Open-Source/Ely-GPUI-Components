# 0030 SelectableList in the AX tree

Date: 2026-10-05. Status: accepted.

`SelectableList` is a list in the AX tree, named by `label`; each `ListItem` is a list item with its title as name, its description, its selection, and, in a selectable list, its place and count (`ListItem::place`). Before, the rows had no role and were missing from the AX tree.
