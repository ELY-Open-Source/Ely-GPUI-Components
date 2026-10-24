//! Ely's documents for GPUI: Markdown read, drawn and edited, block pages, math, diagrams, page viewers, print previews and readers.
#![deny(missing_docs)]

use ely_core::{
    buttons, data_display, feedback, forms, i18n, layout, menus, motion, navigation, overlays,
    primitives, shell, theme, typography,
};
use ely_editor as editor;

mod blocks;
#[cfg(test)]
mod clock;
mod contents;
mod counts;
mod diagram;
mod document;
mod epub;
mod format;
mod history;
mod knowledge;
mod link;
mod markdown;
mod math;
mod modes;
mod pages;
mod picture;
mod print;
mod reading;
mod render;
mod rich;
mod suggest;
mod templates;
#[cfg(all(test, feature = "test-support"))]
mod tests;
mod thumbs;
mod todo;
mod toolbar;
mod tree;
mod viewer;

use gpui::App;

pub use blocks::source;
pub use blocks::{Align, BlockData, BlockEditor, BlockEvent, BlockKind, Media};
pub use contents::outline;
pub use counts::{ReadingTime, WordCount};
pub use diagram::{Diagram, diagram_svg};
pub(crate) use diagram::{flatten, options as svg_options};
pub use document::{
    Attachment, BlockCommand, DocumentEvent, DocumentHeading, DocumentLook, DocumentTarget,
    EditRefusal, InlineFormat, LARGEST_READ, MarkdownDocument, TableEdit, TableRefusal,
    TargetBlock,
};
pub use epub::{Chapter, EpubReader};
pub use format::{Format, format};
pub use history::Change;
pub use history::{PageHistoryDiff, Version, VersionHistory};
pub use knowledge::{Backlink, Backlinks, Favorite, Favorites, TrashBin, Trashed};
pub use link::LinkEditor;
pub use markdown::file_tree as markdown_tree;
pub use markdown::markdown_highlights;
pub use markdown::mdast::{self, AlignKind};
pub use math::{Latex, MathError, formula_svg};
pub(crate) use math::{failed, typeset};
pub use modes::{MarkdownEditor, MarkdownMode};
pub use pages::{PageCover, PageIcon};
pub use picture::{HeldPicture, Pictures};
pub use print::{Margins, Paper, PrintPreview, PrintSettings, sheet};
pub use reading::{Footnote, Glossary, ReadingProgress, TooltipTerm, ZenMode};
pub use render::MarkdownRenderer;
pub use rich::RichTextEditor;
pub use templates::{Template, TemplatePicker};
pub use thumbs::PageThumbnailList;
pub use todo::{Checklist, TodoItem};
pub use toolbar::FixedFormatToolbar;
pub use tree::{AlertKind, Inline, Node as MarkdownBlock, Style};
pub use tree::{Reader as MarkdownReader, alert, is_toc};
pub use viewer::step as zoom_step;
pub use viewer::{DocPage, DocumentViewer, PageHit, PageNote};

/// Binds the documents keys. `init` calls it.
pub fn bind_keys(cx: &mut App) {
    rich::bind_keys(cx);
    document::bind_keys(cx);
}

/// Core's test setup with this chapter's keys.
#[cfg(all(test, feature = "test-support"))]
fn init_for_tests(cx: &mut gpui::App) {
    ely_core::init_for_tests(cx);
    editor::bind_keys(cx);
    bind_keys(cx);
}
