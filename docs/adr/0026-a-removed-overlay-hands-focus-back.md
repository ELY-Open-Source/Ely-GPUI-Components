# 0026 A removed overlay hands focus back

Date: 2026-10-05. Status: accepted.

An overlay its owner removes without calling `Close` still hands focus back: `take_focus` records each holder and what it took focus from, `give_back` forgets it, and when the focused element leaves the tree the root scope walks back past every gone holder to the focus before them all, else takes it; nested overlays removed together return to what the outer one took focus from. Before, a dialog whose action set its owner's state to closed left focus on the root, and keys reached nothing until a click.
