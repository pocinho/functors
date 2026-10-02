use crate::model::{Model, Position, Selection};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    Backspace,
    Delete,
    Enter,
    Home,
    End,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    WindowResized { width: u32, height: u32 },
    ModifiersChanged { shift: bool },
    RedrawRequested,
    KeyPressed(Key),
    TextInput(String),
    PointerPressed { position: Position },
    PointerDragged { position: Position },
    CloseRequested,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    RequestRedraw,
    Exit,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Transition {
    pub model: Model,
    pub commands: Vec<Command>,
}

pub fn update(mut model: Model, message: Message) -> Transition {
    let mut commands = Vec::new();

    match message {
        Message::WindowResized { width, height } => {
            model.viewport.width = width;
            model.viewport.height = height;
            model.needs_redraw = true;
            commands.push(Command::RequestRedraw);
        }
        Message::ModifiersChanged { shift } => model.shift_down = shift,
        Message::RedrawRequested => model.needs_redraw = false,
        Message::TextInput(text) => {
            consume_selection(&mut model);
            model.document.insert_text(&mut model.cursor, &text);
            model.needs_redraw = true;
            if !text.is_empty() {
                commands.push(Command::RequestRedraw);
            }
        }
        Message::KeyPressed(key) => {
            apply_key(&mut model, key);
            model.needs_redraw = true;
            commands.push(Command::RequestRedraw);
        }
        Message::PointerPressed { position } => {
            let position = model.document.clamp_position(position);
            model.cursor = position.clone();
            model.selection = Some(Selection {
                anchor: position.clone(),
                focus: position,
            });
            model.needs_redraw = true;
            commands.push(Command::RequestRedraw);
        }
        Message::PointerDragged { position } => {
            let position = model.document.clamp_position(position);
            model.cursor = position.clone();
            if let Some(selection) = &mut model.selection {
                selection.focus = position;
            }
            model.needs_redraw = true;
            commands.push(Command::RequestRedraw);
        }
        Message::CloseRequested => commands.push(Command::Exit),
    }

    Transition { model, commands }
}

fn apply_key(model: &mut Model, key: Key) {
    if model.shift_down && is_selection_key(key) {
        let anchor = model
            .selection
            .as_ref()
            .map(|selection| selection.anchor.clone())
            .unwrap_or_else(|| model.cursor.clone());
        move_cursor(model, key);
        model.selection = (anchor != model.cursor).then_some(Selection {
            anchor,
            focus: model.cursor.clone(),
        });
        return;
    }

    let selection_consumed =
        matches!(key, Key::Backspace | Key::Delete | Key::Enter) && consume_selection(model);
    model.selection = None;
    match key {
        Key::Left | Key::Right | Key::Up | Key::Down | Key::Home | Key::End => {
            move_cursor(model, key)
        }
        Key::Backspace | Key::Delete if selection_consumed => {}
        Key::Backspace => model.document.backspace(&mut model.cursor),
        Key::Delete => model.document.delete(&mut model.cursor),
        Key::Enter => model.document.insert_text(&mut model.cursor, "\n"),
    }
}

fn is_selection_key(key: Key) -> bool {
    matches!(
        key,
        Key::Left | Key::Right | Key::Up | Key::Down | Key::Home | Key::End
    )
}

fn move_cursor(model: &mut Model, key: Key) {
    match key {
        Key::Left => {
            if model.cursor.column > 0 {
                model.cursor.column -= 1;
            } else if model.cursor.line > 0 {
                model.cursor.line -= 1;
                model.cursor.column = model.document.line_len(model.cursor.line);
            }
        }
        Key::Right => {
            let line_length = model.document.line_len(model.cursor.line);
            if model.cursor.column < line_length {
                model.cursor.column += 1;
            } else if model.cursor.line + 1 < model.document.line_count() {
                model.cursor.line += 1;
                model.cursor.column = 0;
            }
        }
        Key::Up => {
            model.cursor.line = model.cursor.line.saturating_sub(1);
            model.cursor = model.document.clamp_position(model.cursor.clone());
        }
        Key::Down => {
            model.cursor.line = (model.cursor.line + 1).min(model.document.line_count() - 1);
            model.cursor = model.document.clamp_position(model.cursor.clone());
        }
        Key::Home => model.cursor.column = 0,
        Key::End => model.cursor.column = model.document.line_len(model.cursor.line),
        _ => {}
    }
}

fn consume_selection(model: &mut Model) -> bool {
    let Some(selection) = model.selection.take() else {
        return false;
    };

    let (start, end) = if selection.anchor <= selection.focus {
        (selection.anchor, selection.focus)
    } else {
        (selection.focus, selection.anchor)
    };
    if start == end {
        return false;
    }

    model.document.delete_range(&start, &end);
    model.cursor = start;
    true
}

#[cfg(test)]
mod tests {
    use super::{Command, Key, Message, update};
    use crate::model::{Model, Position, Selection};

    #[test]
    fn initial_model_has_one_empty_line_and_requests_a_frame() {
        let model = Model::default();

        assert_eq!(model.document.lines, vec![String::new()]);
        assert_eq!(model.cursor, Position::default());
        assert!(model.needs_redraw);
    }

    #[test]
    fn text_input_updates_document_cursor_and_redraw_command() {
        let transition = update(Model::default(), Message::TextInput("hello".into()));

        assert_eq!(transition.model.document.lines, vec!["hello"]);
        assert_eq!(transition.model.cursor, Position { line: 0, column: 5 });
        assert!(transition.model.document.dirty);
        assert_eq!(transition.commands, vec![Command::RequestRedraw]);
    }

    #[test]
    fn enter_and_backspace_preserve_cursor_invariants() {
        let model = update(Model::default(), Message::TextInput("ab".into())).model;
        let model = update(model, Message::KeyPressed(Key::Enter)).model;
        let model = update(model, Message::TextInput("c".into())).model;
        let model = update(model, Message::KeyPressed(Key::Backspace)).model;

        assert_eq!(
            model.document.lines,
            vec![String::from("ab"), String::new()]
        );
        assert_eq!(model.cursor, Position { line: 1, column: 0 });
    }

    #[test]
    fn delete_removes_text_at_cursor_and_joins_lines() {
        let mut model = update(Model::default(), Message::TextInput("ab\ncd".into())).model;
        model.cursor = Position { line: 0, column: 1 };

        let model = update(model, Message::KeyPressed(Key::Delete)).model;
        assert_eq!(model.document.lines, vec!["a", "cd"]);

        let mut model = model;
        model.cursor = Position { line: 0, column: 1 };
        let model = update(model, Message::KeyPressed(Key::Delete)).model;
        assert_eq!(model.document.lines, vec!["acd"]);
    }

    #[test]
    fn movement_keys_clamp_at_line_and_document_edges() {
        let model = update(Model::default(), Message::TextInput("abc\ndef".into())).model;
        let model = update(model, Message::KeyPressed(Key::Up)).model;
        let model = update(model, Message::KeyPressed(Key::Home)).model;
        let model = update(model, Message::KeyPressed(Key::Right)).model;
        let model = update(model, Message::KeyPressed(Key::Down)).model;
        let model = update(model, Message::KeyPressed(Key::End)).model;
        let model = update(model, Message::KeyPressed(Key::Right)).model;

        assert_eq!(model.cursor, Position { line: 1, column: 3 });
    }

    #[test]
    fn resize_updates_viewport_without_platform_types() {
        let transition = update(
            Model::default(),
            Message::WindowResized {
                width: 800,
                height: 600,
            },
        );

        assert_eq!(transition.model.viewport.width, 800);
        assert_eq!(transition.model.viewport.height, 600);
        assert_eq!(transition.commands, vec![Command::RequestRedraw]);
    }

    #[test]
    fn pointer_press_moves_cursor_and_requests_redraw() {
        let model = update(Model::default(), Message::TextInput("abc\ndef".into())).model;
        let transition = update(
            model,
            Message::PointerPressed {
                position: Position {
                    line: 1,
                    column: 99,
                },
            },
        );

        assert_eq!(transition.model.cursor, Position { line: 1, column: 3 });
        assert_eq!(
            transition.model.selection,
            Some(Selection {
                anchor: Position { line: 1, column: 3 },
                focus: Position { line: 1, column: 3 },
            })
        );
        assert_eq!(transition.commands, vec![Command::RequestRedraw]);
    }

    #[test]
    fn pointer_drag_extends_selection_from_the_pressed_position() {
        let model = update(Model::default(), Message::TextInput("abcdef".into())).model;
        let model = update(
            model,
            Message::PointerPressed {
                position: Position { line: 0, column: 1 },
            },
        )
        .model;
        let transition = update(
            model,
            Message::PointerDragged {
                position: Position { line: 0, column: 4 },
            },
        );

        assert_eq!(transition.model.cursor, Position { line: 0, column: 4 });
        assert_eq!(
            transition.model.selection,
            Some(Selection {
                anchor: Position { line: 0, column: 1 },
                focus: Position { line: 0, column: 4 },
            })
        );
    }

    #[test]
    fn text_input_replaces_a_selected_range() {
        let model = update(Model::default(), Message::TextInput("abcdef".into())).model;
        let model = update(
            update(
                model,
                Message::PointerPressed {
                    position: Position { line: 0, column: 1 },
                },
            )
            .model,
            Message::PointerDragged {
                position: Position { line: 0, column: 4 },
            },
        )
        .model;

        let transition = update(model, Message::TextInput("X".into()));

        assert_eq!(transition.model.document.lines, vec!["aXef"]);
        assert_eq!(transition.model.cursor, Position { line: 0, column: 2 });
        assert_eq!(transition.model.selection, None);
    }

    #[test]
    fn text_input_replaces_a_multiline_selection() {
        let model = update(Model::default(), Message::TextInput("ab\ncd\nef".into())).model;
        let model = update(
            model,
            Message::PointerPressed {
                position: Position { line: 0, column: 1 },
            },
        )
        .model;
        let model = update(
            model,
            Message::PointerDragged {
                position: Position { line: 2, column: 1 },
            },
        )
        .model;

        let transition = update(model, Message::TextInput("X".into()));

        assert_eq!(transition.model.document.lines, vec!["aXf"]);
        assert_eq!(transition.model.cursor, Position { line: 0, column: 2 });
    }

    #[test]
    fn backspace_removes_a_selected_range() {
        let model = update(Model::default(), Message::TextInput("abcdef".into())).model;
        let model = update(
            update(
                model,
                Message::PointerPressed {
                    position: Position { line: 0, column: 1 },
                },
            )
            .model,
            Message::PointerDragged {
                position: Position { line: 0, column: 4 },
            },
        )
        .model;

        let transition = update(model, Message::KeyPressed(Key::Backspace));

        assert_eq!(transition.model.document.lines, vec!["aef"]);
        assert_eq!(transition.model.cursor, Position { line: 0, column: 1 });
    }

    #[test]
    fn enter_replaces_a_selected_range_with_a_newline() {
        let model = update(Model::default(), Message::TextInput("abcdef".into())).model;
        let model = update(
            update(
                model,
                Message::PointerPressed {
                    position: Position { line: 0, column: 1 },
                },
            )
            .model,
            Message::PointerDragged {
                position: Position { line: 0, column: 4 },
            },
        )
        .model;

        let transition = update(model, Message::KeyPressed(Key::Enter));

        assert_eq!(transition.model.document.lines, vec!["a", "ef"]);
        assert_eq!(transition.model.cursor, Position { line: 1, column: 0 });
    }

    #[test]
    fn shift_navigation_extends_selection_without_mutating_text() {
        let model = update(Model::default(), Message::TextInput("abcd".into())).model;
        let model = update(model, Message::KeyPressed(Key::Home)).model;
        let model = update(model, Message::ModifiersChanged { shift: true }).model;
        let model = update(model, Message::KeyPressed(Key::Right)).model;
        let transition = update(model, Message::KeyPressed(Key::Right));

        assert_eq!(transition.model.document.lines, vec!["abcd"]);
        assert_eq!(transition.model.cursor, Position { line: 0, column: 2 });
        assert_eq!(
            transition.model.selection,
            Some(Selection {
                anchor: Position { line: 0, column: 0 },
                focus: Position { line: 0, column: 2 },
            })
        );
    }

    #[test]
    fn movement_clears_selection_without_deleting_text() {
        let model = update(Model::default(), Message::TextInput("abcd".into())).model;
        let model = update(
            update(
                model,
                Message::PointerPressed {
                    position: Position { line: 0, column: 1 },
                },
            )
            .model,
            Message::PointerDragged {
                position: Position { line: 0, column: 3 },
            },
        )
        .model;

        let transition = update(model, Message::KeyPressed(Key::Left));

        assert_eq!(transition.model.document.lines, vec!["abcd"]);
        assert_eq!(transition.model.cursor, Position { line: 0, column: 2 });
        assert_eq!(transition.model.selection, None);
    }

    #[test]
    fn close_requests_exit_without_mutating_model() {
        let model = Model::default();
        let transition = update(model.clone(), Message::CloseRequested);

        assert_eq!(transition.model, model);
        assert_eq!(transition.commands, vec![Command::Exit]);
    }
}
