# 0027 Select and TextInput take a label

Date: 2026-10-05. Status: accepted.

`Select::label` and `TextInput::label` name the control for assistive technology without drawing anything; a text input with neither label nor placeholder had no name, and a select read only its value, so a field named by a description list beside it was anonymous in the AX tree.
