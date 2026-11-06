# 0062 Text selection and row actions

Date: 2026-11-05. Status: proposed.

Diff, blame and merge content uses a shared selection scope. Each scope owns its text, focus and byte endpoints; the visible fragments supply their layouts. Diff and merge columns have separate scopes. Copy joins selected text lines, excluding line numbers, controls and absent cells. Virtual lists share their scroll handle with the scope so a held drag can pass the viewport edge and empty cells.

A click still invokes the row action, including Shift-click for staging ranges. A drag selects text. On release the selection listener runs before row click handlers in the bubble phase, after their capture cleanup. This also prevents a release over the gutter from toggling a staging row. A new press updates the pointer so an earlier scrolling task cannot continue a new gesture from an old edge.

SelectableText keeps its existing single-range implementation and Shift-click extension, without the multi-row layout map. DescriptionList and ListItem expose opt-in selectable text. Existing editable controls keep their own selection behavior. Tests drive actual GPUI mouse and keyboard events; Windows checks also verify the clipboard and Git index independently.
