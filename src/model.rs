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
    pub lines: Vec<String>,
    pub dirty: bool,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            dirty: false,
        }
    }
}

impl Document {
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn line_len(&self, line: usize) -> usize {
        self.lines
            .get(line)
            .map(|text| text.chars().count())
            .unwrap_or_default()
    }

    pub fn clamp_position(&self, position: Position) -> Position {
        let line = position.line.min(self.lines.len().saturating_sub(1));
        let column = position.column.min(self.line_len(line));

        Position { line, column }
    }

    pub fn insert_text(&mut self, position: &mut Position, text: &str) {
        *position = self.clamp_position(position.clone());

        for character in text.chars() {
            if character == '\n' {
                let remainder = split_at_char(&mut self.lines[position.line], position.column);
                self.lines.insert(position.line + 1, remainder);
                position.line += 1;
                position.column = 0;
            } else {
                let line = &mut self.lines[position.line];
                let byte_index = char_to_byte_index(line, position.column);
                line.insert(byte_index, character);
                position.column += 1;
            }
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

        if start.line == end.line {
            let line = &mut self.lines[start.line];
            let start_byte = char_to_byte_index(line, start.column);
            let end_byte = char_to_byte_index(line, end.column);
            line.replace_range(start_byte..end_byte, "");
        } else {
            let prefix = self.lines[start.line]
                .chars()
                .take(start.column)
                .collect::<String>();
            let suffix = self.lines[end.line]
                .chars()
                .skip(end.column)
                .collect::<String>();
            self.lines[start.line] = prefix + &suffix;
            self.lines.drain(start.line + 1..=end.line);
        }

        if start != end {
            self.dirty = true;
        }
    }

    pub fn backspace(&mut self, position: &mut Position) {
        *position = self.clamp_position(position.clone());

        if position.column > 0 {
            let line = &mut self.lines[position.line];
            let start = char_to_byte_index(line, position.column - 1);
            let end = char_to_byte_index(line, position.column);
            line.replace_range(start..end, "");
            position.column -= 1;
            self.dirty = true;
        } else if position.line > 0 {
            let current = self.lines.remove(position.line);
            position.line -= 1;
            position.column = self.line_len(position.line);
            self.lines[position.line].push_str(&current);
            self.dirty = true;
        }
    }

    pub fn delete(&mut self, position: &mut Position) {
        *position = self.clamp_position(position.clone());

        if position.column < self.line_len(position.line) {
            let line = &mut self.lines[position.line];
            let start = char_to_byte_index(line, position.column);
            let end = char_to_byte_index(line, position.column + 1);
            line.replace_range(start..end, "");
            self.dirty = true;
        } else if position.line + 1 < self.lines.len() {
            let next = self.lines.remove(position.line + 1);
            self.lines[position.line].push_str(&next);
            self.dirty = true;
        }
    }
}

fn char_to_byte_index(text: &str, column: usize) -> usize {
    text.char_indices()
        .nth(column)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

fn split_at_char(text: &mut String, column: usize) -> String {
    let byte_index = char_to_byte_index(text, column);
    text.split_off(byte_index)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EditorMode {
    #[default]
    Insert,
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
    pub cursor: Position,
    pub selection: Option<Selection>,
    pub shift_down: bool,
    pub viewport: Viewport,
    pub mode: EditorMode,
    pub needs_redraw: bool,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            document: Document::default(),
            cursor: Position::default(),
            selection: None,
            shift_down: false,
            viewport: Viewport::default(),
            mode: EditorMode::Insert,
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
    use super::Model;

    #[test]
    fn initial_model_has_one_empty_line_and_requests_redraw() {
        let model = Model::initial();

        assert_eq!(model.document.lines, vec![String::new()]);
        assert!(model.needs_redraw);
        assert!(!model.document.dirty);
    }
}
