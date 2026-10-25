# 0053 Every declared locale has the whole catalog

Date: 2026-10-24. Status: accepted.

Every declared locale has Ely's whole catalog (OPTIMIZATION.md O-20, D5), at the owner's word: German, French, Japanese and Hebrew machine translated, marked for native review; en-GB reads English; the lookup no longer falls back to English. The fixed English D5 lists moved into the catalog; other fixed English stays outside the copy rule. Words that served as values split: `RelativeRange`, `ClearAfter`, `messaging::Delivery`, the project, canvas and dashboard enums and `documents::BlockKind` give a stable `key()` or `id()` and take `cx` for their shown `label`, `words` or `word`; `ConfirmButton`, `Cascader`, `MultiSelect`, `TimePicker`, `DateTimePicker`, `InlineEdit` and `ThinkingIndicator` hold `Option` defaults the render reads from the catalog; `typography::relative_text` is public plumbing.
