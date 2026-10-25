# 0041 One Markdown parser, pulldown-cmark

Date: 2026-10-22. Status: accepted, partly superseded by 0051.

One Markdown parser, pulldown-cmark, at OPTIMIZATION.md's O-05. markdown-rs read 128 KiB of one-letter paragraphs in 1.84 s, its tokenizer quadratic in nodes with ten performance fixes open upstream; pulldown-cmark reads it in 20 ms. Ely builds its own tree from pulldown-cmark's events, checked against markdown-rs on CommonMark's and GFM's 1,324 spec examples: 1,222 alike, the rest spec-correct values or span ends. `documents::markdown_tree`, `MarkdownReader`, `alert` and `is_toc` leave the public API with markdown-rs's types; `AlignKind` stays.
