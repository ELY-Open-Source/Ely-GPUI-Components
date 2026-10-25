# 0051 The Markdown tree returns to the public API

Date: 2026-10-24. Status: accepted.

`documents::mdast`, `markdown_tree`, `MarkdownReader` (`joined`, `block`, `inline`, `note`, `label`), `alert` and `is_toc` return to the public API, on Ely's own tree: ElyMark exports HTML and plain text from the tree the drawing reads. `mdast::List::spread` marks a loose list, one whose items open paragraphs, as CommonMark's HTML wraps them.
