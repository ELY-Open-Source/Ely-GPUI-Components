# VoiceOver walk: form, menu, dialog

Walked 2026-10-26 on the Mac mini (macOS 27.0.1, VoiceOver in Chinese), the gallery at `2bd312c` built on Rust 1.98, each story opened alone with `--story`. VoiceOver moved by its own cursor (VO-Right, VO-Space) through AppleScript, and every phrase it spoke was read back from `last phrase`; synthetic keys from the SSH session reached VoiceOver as commands, not the app, so no walk below rests on them. Usage hints VoiceOver adds after a pause ("你当前在文本栏上…") are left out.

## Form: `Form / FormSection / FormField / FieldArray / DirtyIndicator`

| Step | Before | After the fix |
| --- | --- | --- |
| start | Ada Lovelace text field, has keyboard focus | Name text field, has keyboard focus |
| VO-Right | ada@example.com text field | Email text field. We never share it. |
| VO-Right | A line about you text field | Bio text field. Plain text, a few lines. |
| VO-Right | Add link button | Add link button |
| VO-Right | Discard button | Discard button. Unavailable |
| VO-Right | Save button | Save button. Unavailable |

Found: a `FormField` drew its label beside the field but named nothing, so VoiceOver read the value or the placeholder. Windows' audit had passed these fields, a value counting as a name there. Fixed: `FormField` lays its control out with its label and its error or description in scope (`forms::naming`); `Input` hands them to its `TextInput`, a name after the owner's own label and before the placeholder, and the line under the field as its description. `forms::tests::structure::a_field_names_the_input_inside_it_and_no_other` holds it.

## Menu: `DropdownMenu / SplitButton / OverflowMenu`

| Step | VoiceOver |
| --- | --- |
| start | Share button, has keyboard focus |
| VO-Right ×3 | Merge button; More options button; More button |
| VO-Space on Share, then VO-Down | Copy link (menu item) |
| VO-Down | Email |

Both items of the Share menu are read, in order, as menu items. Opening the menu is silent until the first move: see the gpui limits below.

## Dialog: `Dialog / Modal / AlertDialog / ConfirmDialog / PromptDialog / FullscreenDialog`

| Step | VoiceOver |
| --- | --- |
| start | Edit profile button, has keyboard focus |
| VO-Right ×2 | Session expired button; Delete project button |
| VO-Space | (nothing) |
| VO-Right ×2 | Rename button; Release notes button (behind the dialog) |
| VO-Right | Delete this project?, dialog |
| VO-Interact | Its pages, files and history go with it. This cannot be undone. |
| VO-Right ×2 | Cancel button; Delete project button |

Inside, the dialog reads its title, its line and both buttons. Found: the focus a dialog took on opening sat on a scope with no accessibility node (gpui logged it and fell back to the whole window). Fixed: a trapping `FocusScope` carries the node (`FocusScope::node`) for `Dialog`, `ActionSheet`, `Lightbox`, `Spotlight` and `Tour`, and a `Dialog` without `focus_first` moves focus to its first control a frame after it opens, Cancel in a confirm, as macOS's own alerts do; `overlays::tests::a_dialog_focuses_its_first_control_a_frame_after_it_opens` holds it. Walked again with the fix: the tree grows from 6 to 10 nodes on the press and gpui no longer reports a focused element without a node.

## gpui limits, open upstream

- VoiceOver does not follow focus gpui moves without a key press: opening the menu or the dialog leaves VoiceOver's cursor on the trigger, though gpui's tree holds the new focus. The user hears the open menu or dialog on the next move.
- gpui 0.3.10 marks no node modal, so VoiceOver's cursor reaches the page behind an open dialog.
- `Popover` holds only its owner's content and names no node; its focus stays silent until the user moves into the content.
