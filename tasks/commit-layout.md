# Commit footer wrapping

Reported in https://github.com/ELY-Open-Source/ELYTerminal/issues/20.

The amend wrapper used `flex_1().min_w_0()`, giving it a zero flex basis.
A long branch could therefore take the row's width before flex wrapping moved
the button, squeezing the checkbox label into individual letters.

The wrapper now reserves its natural width without shrinking. The commit
button has a bounded flex wrapper and fills its row when wrapped. The shared
Checkbox and Button implementations are unchanged: long branch labels still
use Button's ellipsis and full accessible label. Commit readiness, counters,
amend state and keyboard/click handlers are unchanged.

This fix starts from `7f7a715d797d8450eed4f17ae8002b5427d01d28`, the revision
ELYTerminal 0.1.8 already pins, so adopting it requires no unrelated library
updates.

## Regression coverage

`cargo test --locked --lib --features test-support git::tests::commit`

- Content widths 143, 144, 160, 240, 320, 560 and 960 px, including repeated
  shrinking and expanding. The 143–144 px cases account for ELYTerminal's
  minimum 160 px panel after padding and its border.
- Short, very long and multibyte branches, with amend off and on.
- Label geometry compared with a standalone checkbox at the same width;
  normal word wrapping at an intrinsically narrow width is allowed, squeezing
  caused by the adjacent button is not.
- Control containment, nonoverlap, short controls sharing a row, and wrapped
  buttons occupying the available row width.
- Disabled commits, amend clicks, normal commits, plain Enter remaining a
  field action, and Ctrl/Cmd+Enter remaining the submit shortcut.
- Linux additionally runs the resize matrix with the native text system and
  bundled Inter font rather than only GPUI's synthetic text metrics.

## Native visual acceptance

In a disposable repository, use a staged change and branch
`feature/ui-amend-layout-with-a-very-long-branch-name`. Resize the Git panel
from wide to its minimum and back, with both enabled/disabled buttons and
amend states. Confirm readable labels, clickable controls, contained ellipses
and an accessible recent-commits section. Repeat with `main` and a multibyte
branch. Repeat on Windows at the user's display scale; headless geometry and
Linux text shaping are not a Windows visual test.
