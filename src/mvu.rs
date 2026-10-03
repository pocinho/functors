use crate::file_io::{FileError, LoadedFile, SavedFile};
use crate::model::{EditorState, Model, Position, Selection};
use crate::workspace::{Workspace, WorkspaceError};
use std::path::PathBuf;

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
    PageUp,
    PageDown,
    Escape,
}

pub const COMMAND_BAR_COMMANDS: [&str; 4] = ["Open File", "Open Workspace", "Save", "Settings"];

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    WindowResized { width: u32, height: u32 },
    ModifiersChanged { shift: bool },
    RedrawRequested,
    KeyPressed(Key),
    TextInput(String),
    Scrolled { vertical: i32, horizontal: i32 },
    ToggleCommandBar,
    OpenSettings,
    PointerPressed { position: Position },
    PointerDragged { position: Position },
    CloseRequested,
    OpenWorkspacePickerRequested,
    OpenWorkspaceRequested(PathBuf),
    WorkspaceOpened(Result<Workspace, WorkspaceError>),
    OpenFilePickerRequested,
    OpenFileRequested(PathBuf),
    FileOpened(Result<LoadedFile, FileError>),
    SaveFileRequested,
    FileSaved(Result<SavedFile, FileError>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    RequestRedraw,
    Exit,
    OpenWorkspace(PathBuf),
    OpenFile(PathBuf),
    SaveFile {
        path: PathBuf,
        text: String,
        expected_stamp: Option<crate::file_io::FileStamp>,
    },
    OpenWorkspacePicker,
    OpenFilePicker,
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
            if model.command_bar_open {
                model.command_bar_query.push_str(&text);
                model.command_bar_selection = 0;
            } else {
                consume_selection(&mut model);
                model.document.insert_text(&mut model.cursor, &text);
                if !text.is_empty() {
                    model.state = if model.file_path.is_some() {
                        EditorState::Active
                    } else {
                        EditorState::Empty
                    };
                }
            }
            model.needs_redraw = true;
            if !text.is_empty() {
                commands.push(Command::RequestRedraw);
            }
        }
        Message::Scrolled {
            vertical,
            horizontal,
        } => {
            model.viewport.vertical_offset = offset_by(model.viewport.vertical_offset, vertical);
            model.viewport.horizontal_offset =
                offset_by(model.viewport.horizontal_offset, horizontal);
            model.needs_redraw = true;
            commands.push(Command::RequestRedraw);
        }
        Message::KeyPressed(key) => {
            if model.command_bar_open && key == Key::Backspace {
                model.command_bar_query.pop();
                model.command_bar_selection = 0;
            } else if model.command_bar_open && key == Key::Escape {
                model.command_bar_open = false;
                model.command_bar_query.clear();
                model.command_bar_selection = 0;
            } else if model.command_bar_open && key == Key::Up {
                model.command_bar_selection = model.command_bar_selection.saturating_sub(1);
            } else if model.command_bar_open && key == Key::Down {
                model.command_bar_selection = (model.command_bar_selection + 1)
                    .min(COMMAND_BAR_COMMANDS.len().saturating_sub(1));
            } else if model.command_bar_open && key == Key::Enter {
                submit_command_bar(&mut model, &mut commands);
            } else if !model.command_bar_open {
                apply_key(&mut model, key);
                model.state = if model.file_path.is_some() {
                    EditorState::Active
                } else {
                    EditorState::Empty
                };
            }
            model.needs_redraw = true;
            commands.push(Command::RequestRedraw);
        }
        Message::ToggleCommandBar => {
            model.command_bar_open = !model.command_bar_open;
            if model.command_bar_open {
                model.settings_open = false;
            }
            model.command_bar_query.clear();
            model.command_bar_selection = 0;
            model.needs_redraw = true;
            commands.push(Command::RequestRedraw);
        }
        Message::OpenSettings => {
            model.command_bar_open = false;
            model.command_bar_query.clear();
            model.command_bar_selection = 0;
            model.settings_open = true;
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
        Message::OpenWorkspacePickerRequested => {}
        Message::OpenFilePickerRequested => {}
        Message::OpenWorkspaceRequested(path) => {
            model.state = EditorState::Loading;
            commands.push(Command::OpenWorkspace(path));
        }
        Message::WorkspaceOpened(result) => {
            match result {
                Ok(workspace) => {
                    model.workspace = Some(workspace);
                    model.workspace_error = None;
                    model.state = if model.file_path.is_some() {
                        EditorState::Active
                    } else {
                        EditorState::Empty
                    };
                }
                Err(error) => {
                    model.workspace_error = Some(error);
                    model.state = EditorState::Error;
                }
            }
            model.needs_redraw = true;
            commands.push(Command::RequestRedraw);
        }
        Message::OpenFileRequested(path) => {
            model.state = EditorState::Loading;
            commands.push(Command::OpenFile(path));
        }
        Message::FileOpened(result) => match result {
            Ok(file) => {
                model.document = crate::model::Document::from_text(&file.text, file.language);
                model.file_path = Some(file.path);
                model.file_stamp = Some(file.stamp);
                model.file_error = None;
                model.state = EditorState::Active;
                model.cursor = Position::default();
                model.selection = None;
                model.needs_redraw = true;
                commands.push(Command::RequestRedraw);
            }
            Err(error) => {
                model.file_error = Some(error);
                model.state = EditorState::Error;
                model.needs_redraw = true;
                commands.push(Command::RequestRedraw);
            }
        },
        Message::SaveFileRequested => {
            if let Some(path) = &model.file_path {
                commands.push(Command::SaveFile {
                    path: path.clone(),
                    text: model.document.text(),
                    expected_stamp: model.file_stamp.clone(),
                });
            }
        }
        Message::FileSaved(result) => match result {
            Ok(path) => {
                model.document.dirty = false;
                model.file_path = Some(path.path);
                model.file_stamp = Some(path.stamp);
                model.file_error = None;
                model.state = EditorState::Active;
                model.needs_redraw = true;
                commands.push(Command::RequestRedraw);
            }
            Err(error) => {
                model.file_error = Some(error);
                model.state = EditorState::Error;
                model.needs_redraw = true;
                commands.push(Command::RequestRedraw);
            }
        },
    }

    Transition { model, commands }
}

fn submit_command_bar(model: &mut Model, commands: &mut Vec<Command>) {
    let query = model.command_bar_query.trim().to_ascii_lowercase();
    let query = if query.is_empty() {
        COMMAND_BAR_COMMANDS[model.command_bar_selection].to_ascii_lowercase()
    } else {
        query
    };
    model.command_bar_open = false;
    model.command_bar_query.clear();
    model.command_bar_selection = 0;
    match query.as_str() {
        "settings" | "open settings" => model.settings_open = true,
        "open workspace" | "workspace" => commands.push(Command::OpenWorkspacePicker),
        "open file" | "file" => commands.push(Command::OpenFilePicker),
        "save" => {
            if let Some(path) = &model.file_path {
                commands.push(Command::SaveFile {
                    path: path.clone(),
                    text: model.document.text(),
                    expected_stamp: model.file_stamp.clone(),
                });
            }
        }
        _ => {}
    }
    model.needs_redraw = true;
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
        Key::PageUp => {
            let page = (model.viewport.height / 20).max(1) as usize;
            model.viewport.vertical_offset = model.viewport.vertical_offset.saturating_sub(page);
        }
        Key::PageDown => {
            let page = (model.viewport.height / 20).max(1) as usize;
            model.viewport.vertical_offset = model.viewport.vertical_offset.saturating_add(page);
        }
        Key::Escape => {}
    }
}

fn is_selection_key(key: Key) -> bool {
    matches!(
        key,
        Key::Left | Key::Right | Key::Up | Key::Down | Key::Home | Key::End
    )
}

fn offset_by(offset: usize, delta: i32) -> usize {
    if delta.is_negative() {
        offset.saturating_sub(delta.unsigned_abs() as usize)
    } else {
        offset.saturating_add(delta as usize)
    }
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
    use crate::file_io::{FileError, FileStamp, LoadedFile};
    use crate::model::{Model, Position, Selection};
    use crate::syntax::SyntaxLanguage;
    use crate::workspace::{Workspace, WorkspaceError};
    use std::path::PathBuf;

    #[test]
    fn initial_model_has_one_empty_line_and_requests_a_frame() {
        let model = Model::default();

        assert_eq!(model.document.lines(), vec![String::new()]);
        assert_eq!(model.cursor, Position::default());
        assert!(model.needs_redraw);
    }

    #[test]
    fn text_input_updates_document_cursor_and_redraw_command() {
        let transition = update(Model::default(), Message::TextInput("hello".into()));

        assert_eq!(transition.model.document.lines(), vec!["hello"]);
        assert_eq!(transition.model.cursor, Position { line: 0, column: 5 });
        assert!(transition.model.document.dirty);
        assert_eq!(transition.commands, vec![Command::RequestRedraw]);
    }

    #[test]
    fn command_bar_navigates_commands_and_submits_the_selected_item() {
        let model = update(Model::default(), Message::ToggleCommandBar).model;
        let model = update(model, Message::KeyPressed(Key::Down)).model;
        assert_eq!(model.command_bar_selection, 1);

        let transition = update(model, Message::KeyPressed(Key::Enter));

        assert!(!transition.model.command_bar_open);
        assert_eq!(transition.model.command_bar_selection, 0);
        assert_eq!(
            transition.commands,
            vec![Command::OpenWorkspacePicker, Command::RequestRedraw]
        );
    }

    #[test]
    fn command_bar_escape_clears_query_and_selection() {
        let model = update(Model::default(), Message::ToggleCommandBar).model;
        let model = update(model, Message::TextInput("save".into())).model;
        let model = update(model, Message::KeyPressed(Key::Down)).model;

        let transition = update(model, Message::KeyPressed(Key::Escape));

        assert!(!transition.model.command_bar_open);
        assert!(transition.model.command_bar_query.is_empty());
        assert_eq!(transition.model.command_bar_selection, 0);
    }

    #[test]
    fn menu_overlays_are_mutually_exclusive() {
        let settings = update(Model::default(), Message::OpenSettings);
        assert!(settings.model.settings_open);
        assert!(!settings.model.command_bar_open);
        assert_eq!(settings.commands, vec![Command::RequestRedraw]);

        let command_bar = update(settings.model, Message::ToggleCommandBar);
        assert!(command_bar.model.command_bar_open);
        assert!(!command_bar.model.settings_open);

        let settings_again = update(command_bar.model, Message::OpenSettings);
        assert!(!settings_again.model.command_bar_open);
        assert!(settings_again.model.settings_open);
    }

    #[test]
    fn enter_and_backspace_preserve_cursor_invariants() {
        let model = update(Model::default(), Message::TextInput("ab".into())).model;
        let model = update(model, Message::KeyPressed(Key::Enter)).model;
        let model = update(model, Message::TextInput("c".into())).model;
        let model = update(model, Message::KeyPressed(Key::Backspace)).model;

        assert_eq!(
            model.document.lines(),
            vec![String::from("ab"), String::new()]
        );
        assert_eq!(model.cursor, Position { line: 1, column: 0 });
    }

    #[test]
    fn delete_removes_text_at_cursor_and_joins_lines() {
        let mut model = update(Model::default(), Message::TextInput("ab\ncd".into())).model;
        model.cursor = Position { line: 0, column: 1 };

        let model = update(model, Message::KeyPressed(Key::Delete)).model;
        assert_eq!(model.document.lines(), vec!["a", "cd"]);

        let mut model = model;
        model.cursor = Position { line: 0, column: 1 };
        let model = update(model, Message::KeyPressed(Key::Delete)).model;
        assert_eq!(model.document.lines(), vec!["acd"]);
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
    fn workspace_request_declares_an_effect_and_failure_is_retained() {
        let path = PathBuf::from("missing-workspace");
        let transition = update(
            Model::default(),
            Message::OpenWorkspaceRequested(path.clone()),
        );
        assert_eq!(transition.commands, vec![Command::OpenWorkspace(path)]);

        let error = WorkspaceError::InvalidRoot(PathBuf::from("missing-workspace"));
        let transition = update(
            transition.model,
            Message::WorkspaceOpened(Err(error.clone())),
        );
        assert_eq!(transition.model.workspace, None);
        assert_eq!(transition.model.workspace_error, Some(error));
        assert_eq!(transition.commands, vec![Command::RequestRedraw]);
    }

    #[test]
    fn file_request_enters_loading_and_failure_enters_error_state() {
        let path = PathBuf::from("main.rs");
        let transition = update(Model::default(), Message::OpenFileRequested(path.clone()));

        assert_eq!(transition.model.state, crate::model::EditorState::Loading);
        assert_eq!(transition.commands, vec![Command::OpenFile(path.clone())]);

        let transition = update(
            transition.model,
            Message::FileOpened(Err(FileError::Read {
                path,
                message: "missing".into(),
            })),
        );
        assert_eq!(transition.model.state, crate::model::EditorState::Error);
    }

    #[test]
    fn scroll_messages_update_both_viewport_offsets() {
        let mut model = Model::default();
        model.viewport.vertical_offset = 4;
        model.viewport.horizontal_offset = 3;

        let transition = update(
            model,
            Message::Scrolled {
                vertical: -2,
                horizontal: 5,
            },
        );

        assert_eq!(transition.model.viewport.vertical_offset, 2);
        assert_eq!(transition.model.viewport.horizontal_offset, 8);
        assert_eq!(transition.commands, vec![Command::RequestRedraw]);
    }

    #[test]
    fn workspace_success_replaces_old_error() {
        let error = WorkspaceError::InvalidRoot(PathBuf::from("missing-workspace"));
        let model = update(Model::default(), Message::WorkspaceOpened(Err(error))).model;
        let workspace = Workspace {
            root: PathBuf::from("workspace"),
            entries: Vec::new(),
        };

        let transition = update(model, Message::WorkspaceOpened(Ok(workspace.clone())));
        assert_eq!(transition.model.workspace, Some(workspace));
        assert_eq!(transition.model.workspace_error, None);
    }

    #[test]
    fn opening_a_workspace_after_a_file_restores_active_state() {
        let model = Model {
            file_path: Some(PathBuf::from("main.rs")),
            state: crate::model::EditorState::Loading,
            ..Model::default()
        };
        let workspace = Workspace {
            root: PathBuf::from("workspace"),
            entries: Vec::new(),
        };

        let transition = update(model, Message::WorkspaceOpened(Ok(workspace)));

        assert_eq!(transition.model.state, crate::model::EditorState::Active);
    }

    #[test]
    fn opening_a_file_resets_document_state_and_selects_its_language() {
        let path = PathBuf::from("main.rs");
        let transition = update(
            Model::default(),
            Message::FileOpened(Ok(LoadedFile {
                path: path.clone(),
                text: "fn main() {}".into(),
                language: SyntaxLanguage::Rust,
                stamp: FileStamp {
                    length: 12,
                    modified: None,
                },
            })),
        );

        assert_eq!(transition.model.file_path, Some(path));
        assert_eq!(transition.model.document.text(), "fn main() {}");
        assert_eq!(transition.model.document.language, SyntaxLanguage::Rust);
        assert!(!transition.model.document.dirty);
        assert_eq!(transition.model.cursor, Position::default());
    }

    #[test]
    fn failed_save_keeps_dirty_state_and_reports_the_error() {
        let path = PathBuf::from("main.rs");
        let model = update(
            Model::default(),
            Message::FileOpened(Ok(LoadedFile {
                path: path.clone(),
                text: "fn main() {}".into(),
                language: SyntaxLanguage::Rust,
                stamp: FileStamp {
                    length: 12,
                    modified: None,
                },
            })),
        )
        .model;
        let model = update(model, Message::TextInput("!".into())).model;
        let transition = update(
            model,
            Message::FileSaved(Err(FileError::Write {
                path: path.clone(),
                message: "permission denied".into(),
            })),
        );

        assert!(transition.model.document.dirty);
        assert_eq!(transition.model.file_path, Some(path.clone()));
        assert_eq!(
            transition.model.file_error,
            Some(FileError::Write {
                path,
                message: "permission denied".into(),
            })
        );
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

        assert_eq!(transition.model.document.lines(), vec!["aXef"]);
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

        assert_eq!(transition.model.document.lines(), vec!["aXf"]);
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

        assert_eq!(transition.model.document.lines(), vec!["aef"]);
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

        assert_eq!(transition.model.document.lines(), vec!["a", "ef"]);
        assert_eq!(transition.model.cursor, Position { line: 1, column: 0 });
    }

    #[test]
    fn shift_navigation_extends_selection_without_mutating_text() {
        let model = update(Model::default(), Message::TextInput("abcd".into())).model;
        let model = update(model, Message::KeyPressed(Key::Home)).model;
        let model = update(model, Message::ModifiersChanged { shift: true }).model;
        let model = update(model, Message::KeyPressed(Key::Right)).model;
        let transition = update(model, Message::KeyPressed(Key::Right));

        assert_eq!(transition.model.document.lines(), vec!["abcd"]);
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

        assert_eq!(transition.model.document.lines(), vec!["abcd"]);
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
