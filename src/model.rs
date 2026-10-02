use ropey::Rope;

use crate::file_io::{FileError, FileStamp};
use crate::workspace::{Workspace, WorkspaceError};

use crate::syntax::SyntaxLanguage;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub anchor: Position,
    pub focus: Position,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    text: Rope,
    pub dirty: bool,
    pub language: SyntaxLanguage,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            text: Rope::new(),
            dirty: false,
            language: SyntaxLanguage::from_extension(None),
        }
    }
}

impl Document {
    pub fn from_text(text: &str, language: SyntaxLanguage) -> Self {
        Self {
            text: Rope::from_str(text),
            dirty: false,
            language,
        }
    }

    pub fn text(&self) -> String {
        self.text.to_string()
    }

    pub fn line_count(&self) -> usize {
        self.text.len_lines()
    }

    #[cfg(test)]
    pub fn lines(&self) -> Vec<String> {
        (0..self.line_count())
            .map(|line| self.line_text(line))
            .collect()
    }

    pub fn line_text(&self, line: usize) -> String {
        let text = self.text.line(line).to_string();
        text.strip_suffix('\n').unwrap_or(&text).to_owned()
    }

    pub fn line_len(&self, line: usize) -> usize {
        self.line_text(line).chars().count()
    }

    pub fn clamp_position(&self, position: Position) -> Position {
        let line = position.line.min(self.line_count().saturating_sub(1));
        let column = position.column.min(self.line_len(line));

        Position { line, column }
    }

    pub fn insert_text(&mut self, position: &mut Position, text: &str) {
        *position = self.clamp_position(position.clone());
        let start_index = self.text.line_to_char(position.line) + position.column;

        for (offset, character) in text.chars().enumerate() {
            let char_index = start_index + offset;
            self.text.insert_char(char_index, character);
            *position = position_after_insert(position, character);
        }

        if !text.is_empty() {
            self.dirty = true;
        }
    }

    pub fn delete_range(&mut self, start: &Position, end: &Position) {
        let start = self.clamp_position(start.clone());
        let end = self.clamp_position(end.clone());
        if start > end {
            return;
        }

        let start_index = self.text.line_to_char(start.line) + start.column;
        let end_index = self.text.line_to_char(end.line) + end.column;
        self.text.remove(start_index..end_index);

        if start != end {
            self.dirty = true;
        }
    }

    pub fn backspace(&mut self, position: &mut Position) {
        *position = self.clamp_position(position.clone());

        if position.column > 0 {
            let line_start = self.text.line_to_char(position.line);
            let end = line_start + position.column;
            self.text.remove(end - 1..end);
            position.column -= 1;
            self.dirty = true;
        } else if position.line > 0 {
            let line_start = self.text.line_to_char(position.line);
            position.line -= 1;
            position.column = self.line_len(position.line);
            self.text.remove(line_start - 1..line_start);
            self.dirty = true;
        }
    }

    pub fn delete(&mut self, position: &mut Position) {
        *position = self.clamp_position(position.clone());

        if position.column < self.line_len(position.line) {
            let start = self.text.line_to_char(position.line) + position.column;
            self.text.remove(start..start + 1);
            self.dirty = true;
        } else if position.line + 1 < self.line_count() {
            let line_end = self.text.line_to_char(position.line + 1) - 1;
            self.text.remove(line_end..line_end + 1);
            self.dirty = true;
        }
    }
}

fn position_after_insert(position: &Position, character: char) -> Position {
    if character == '\n' {
        Position {
            line: position.line + 1,
            column: 0,
        }
    } else {
        Position {
            line: position.line,
            column: position.column + 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EditorMode {
    #[default]
    Insert,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EditorState {
    #[default]
    Empty,
    Loading,
    Active,
    Error,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub vertical_offset: usize,
    pub horizontal_offset: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    pub document: Document,
    pub file_path: Option<std::path::PathBuf>,
    pub file_stamp: Option<FileStamp>,
    pub file_error: Option<FileError>,
    pub workspace: Option<Workspace>,
    pub workspace_error: Option<WorkspaceError>,
    pub cursor: Position,
    pub selection: Option<Selection>,
    pub shift_down: bool,
    pub viewport: Viewport,
    pub mode: EditorMode,
    pub state: EditorState,
    pub command_bar_open: bool,
    pub command_bar_query: String,
    pub settings_open: bool,
    pub needs_redraw: bool,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            document: Document::default(),
            file_path: None,
            file_stamp: None,
            file_error: None,
            workspace: None,
            workspace_error: None,
            cursor: Position::default(),
            selection: None,
            shift_down: false,
            viewport: Viewport::default(),
            mode: EditorMode::Insert,
            state: EditorState::Empty,
            command_bar_open: false,
            command_bar_query: String::new(),
            settings_open: false,
            needs_redraw: true,
        }
    }
}

impl Model {
    pub fn initial() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{Document, Model, Position};

    #[test]
    fn initial_model_has_one_empty_line_and_requests_redraw() {
        let model = Model::initial();

        assert_eq!(model.document.lines(), vec![String::new()]);
        assert!(model.needs_redraw);
        assert!(!model.document.dirty);
    }

    #[test]
    fn multiline_edits_join_lines_and_clamp_document_boundaries() {
        let mut document = Document::default();
        let mut cursor = Position::default();
        document.insert_text(&mut cursor, "first\nsecond\nthird");

        assert_eq!(document.line_count(), 3);
        assert_eq!(cursor, Position { line: 2, column: 5 });

        document.delete_range(
            &Position { line: 0, column: 5 },
            &Position { line: 1, column: 1 },
        );
        assert_eq!(document.lines(), vec!["firstecond", "third"]);

        let clamped = document.clamp_position(Position {
            line: 99,
            column: 99,
        });
        assert_eq!(clamped, Position { line: 1, column: 5 });
    }

    #[test]
    fn empty_documents_and_boundary_deletes_remain_clean() {
        let mut document = Document::default();
        let mut cursor = Position {
            line: 99,
            column: 99,
        };

        document.backspace(&mut cursor);
        document.delete(&mut cursor);

        assert_eq!(cursor, Position::default());
        assert_eq!(document.lines(), vec![String::new()]);
        assert!(!document.dirty);
    }

    #[test]
    fn unicode_and_tabs_use_scalar_columns() {
        let mut document = Document::default();
        let mut cursor = Position::default();
        document.insert_text(&mut cursor, "é\t界");

        assert_eq!(document.line_len(0), 3);
        assert_eq!(cursor, Position { line: 0, column: 3 });
        assert_eq!(
            document.clamp_position(Position { line: 0, column: 8 }),
            cursor
        );
    }

    #[test]
    fn rope_handles_a_large_multiline_document_behind_the_document_api() {
        let text = (0..10_000)
            .map(|line| format!("line {line}\n"))
            .collect::<String>();
        let document = Document::from_text(&text, crate::syntax::SyntaxLanguage::PlainText);

        assert_eq!(document.line_count(), 10_001);
        assert_eq!(document.line_text(9_999), "line 9999");
        assert_eq!(document.line_text(10_000), "");
    }
}
