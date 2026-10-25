# 0028 NumberInput takes a label

Date: 2026-10-05. Status: accepted.

`NumberInput::label` names the field for assistive technology without drawing it, through `TextInput::set_label` on the field's inner input, which lives across frames; a number field named only by a description list beside it was anonymous in the AX tree.
