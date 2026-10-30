use serde::{Deserialize, Serialize};

/// Represents a cursor position in (line, column), 0-indexed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

/// Represents a selection span between two positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    pub anchor: Position,
    pub active: Position,
}

impl Selection {
    pub fn point(pos: Position) -> Self {
        Self {
            anchor: pos,
            active: pos,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.active
    }

    pub fn start(&self) -> Position {
        if self.anchor < self.active {
            self.anchor
        } else {
            self.active
        }
    }

    pub fn end(&self) -> Position {
        if self.anchor < self.active {
            self.active
        } else {
            self.anchor
        }
    }
}

/// Represents an atomic edit operation for the Undo/Redo ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditOperation {
    Insert { offset: usize, text: String },
    Delete { offset: usize, deleted_text: String },
}

/// Text Buffer replicating the core of Fleet Andel / IntelliJ Editor:
/// - Fast random access, line mapping, and multi-caret management.
/// - Full transaction support with undo/redo capabilities.
pub struct EditorBuffer {
    text: String,
    line_starts: Vec<usize>,
    cursors: Vec<Selection>,
    undo_stack: Vec<EditOperation>,
    redo_stack: Vec<EditOperation>,
}

impl EditorBuffer {
    pub fn new(initial_text: &str) -> Self {
        let mut buffer = Self {
            text: initial_text.to_string(),
            line_starts: Vec::new(),
            cursors: vec![Selection::point(Position { line: 0, column: 0 })],
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };
        buffer.recompute_line_starts();
        buffer
    }

    /// Recompute line start byte offsets for O(log N) line-column lookups
    fn recompute_line_starts(&mut self) {
        self.line_starts.clear();
        self.line_starts.push(0);
        for (i, b) in self.text.bytes().enumerate() {
            if b == b'\n' {
                self.line_starts.push(i + 1);
            }
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub fn get_line(&self, line_index: usize) -> Option<&str> {
        if line_index >= self.line_starts.len() {
            return None;
        }
        let start = self.line_starts[line_index];
        let end = if line_index + 1 < self.line_starts.len() {
            // Trim trailing \n or \r\n
            let next_start = self.line_starts[line_index + 1];
            if next_start > 0 && self.text.as_bytes()[next_start - 1] == b'\n' {
                if next_start > 1 && self.text.as_bytes()[next_start - 2] == b'\r' {
                    next_start - 2
                } else {
                    next_start - 1
                }
            } else {
                next_start
            }
        } else {
            self.text.len()
        };
        Some(&self.text[start..end])
    }

    pub fn position_to_offset(&self, pos: Position) -> usize {
        if pos.line >= self.line_starts.len() {
            return self.text.len();
        }
        let line_start = self.line_starts[pos.line];
        let line_end = if pos.line + 1 < self.line_starts.len() {
            self.line_starts[pos.line + 1]
        } else {
            self.text.len()
        };
        (line_start + pos.column).min(line_end)
    }

    pub fn offset_to_position(&self, offset: usize) -> Position {
        let offset = offset.min(self.text.len());
        let line = match self.line_starts.binary_search(&offset) {
            Ok(exact) => exact,
            Err(next) => next.saturating_sub(1),
        };
        let column = offset.saturating_sub(self.line_starts[line]);
        Position { line, column }
    }

    /// Insert text at a specified byte offset
    pub fn insert_at(&mut self, offset: usize, inserted: &str) {
        let offset = offset.min(self.text.len());
        self.text.insert_str(offset, inserted);
        self.recompute_line_starts();

        self.undo_stack.push(EditOperation::Insert {
            offset,
            text: inserted.to_string(),
        });
        self.redo_stack.clear();
    }

    /// Delete text range [start, end)
    pub fn delete_range(&mut self, start: usize, end: usize) -> String {
        let start = start.min(self.text.len());
        let end = end.min(self.text.len()).max(start);
        let deleted = self.text[start..end].to_string();
        self.text.replace_range(start..end, "");
        self.recompute_line_starts();

        self.undo_stack.push(EditOperation::Delete {
            offset: start,
            deleted_text: deleted.clone(),
        });
        self.redo_stack.clear();
        deleted
    }

    /// Undo last edit operation
    pub fn undo(&mut self) -> bool {
        if let Some(op) = self.undo_stack.pop() {
            match op {
                EditOperation::Insert { offset, text } => {
                    let end = offset + text.len();
                    self.text.replace_range(offset..end, "");
                    self.recompute_line_starts();
                    self.redo_stack.push(EditOperation::Insert { offset, text });
                }
                EditOperation::Delete {
                    offset,
                    deleted_text,
                } => {
                    self.text.insert_str(offset, &deleted_text);
                    self.recompute_line_starts();
                    self.redo_stack.push(EditOperation::Delete {
                        offset,
                        deleted_text,
                    });
                }
            }
            true
        } else {
            false
        }
    }

    /// Redo last undone operation
    pub fn redo(&mut self) -> bool {
        if let Some(op) = self.redo_stack.pop() {
            match op {
                EditOperation::Insert { offset, text } => {
                    self.text.insert_str(offset, &text);
                    self.recompute_line_starts();
                    self.undo_stack.push(EditOperation::Delete {
                        offset,
                        deleted_text: text,
                    });
                }
                EditOperation::Delete {
                    offset,
                    deleted_text,
                } => {
                    let end = offset + deleted_text.len();
                    self.text.replace_range(offset..end, "");
                    self.recompute_line_starts();
                    self.undo_stack.push(EditOperation::Insert {
                        offset,
                        text: deleted_text,
                    });
                }
            }
            true
        } else {
            false
        }
    }

    pub fn cursors(&self) -> &[Selection] {
        &self.cursors
    }

    pub fn set_cursor(&mut self, pos: Position) {
        self.cursors = vec![Selection::point(pos)];
    }
}
