//! What an owner reads of the editor's state.

use super::{Buffer, CodeEditor, LineNumbers, Metrics};

impl CodeEditor {
    /// Its text by lines.
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Where the code sat when last painted.
    pub fn metrics(&self) -> Metrics {
        self.metrics
    }

    /// How the gutter numbers lines.
    pub fn numbers(&self) -> LineNumbers {
        self.options.numbers
    }

    /// Whether an edit is left to undo.
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }
}
