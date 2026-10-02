use winit::{
    event::KeyEvent,
    keyboard::{Key as WinitKey, NamedKey},
};

use crate::mvu::{Key, Message};
use crate::view::{FrameDescription, Rect};

pub fn key_message(event: &KeyEvent, control_down: bool) -> Option<Message> {
    if !event.state.is_pressed() {
        return None;
    }

    if control_down {
        if is_shortcut(&event.logical_key, "o") {
            return Some(Message::OpenWorkspacePickerRequested);
        }
        if is_shortcut(&event.logical_key, "p") {
            return Some(Message::OpenFilePickerRequested);
        }
        if is_shortcut(&event.logical_key, "s") {
            return Some(Message::SaveFileRequested);
        }
        if is_shortcut(&event.logical_key, "k") {
            return Some(Message::ToggleCommandBar);
        }
    }

    match &event.logical_key {
        WinitKey::Named(key) => named_key_message(*key),
        WinitKey::Character(_) => None,
        WinitKey::Unidentified(_) | WinitKey::Dead(_) => None,
    }
}

fn is_shortcut(key: &WinitKey, character: &str) -> bool {
    matches!(key, WinitKey::Character(key) if key.eq_ignore_ascii_case(character))
}

pub fn key_text_message(event: &KeyEvent, control_down: bool) -> Option<Message> {
    if !event.state.is_pressed()
        || control_down
        || !matches!(
            event.logical_key,
            WinitKey::Character(_) | WinitKey::Named(NamedKey::Space)
        )
    {
        return None;
    }

    event
        .text
        .as_ref()
        .and_then(|text| text_message(text.to_string()))
}

fn named_key_message(key: NamedKey) -> Option<Message> {
    let key = match key {
        NamedKey::ArrowLeft => Key::Left,
        NamedKey::ArrowRight => Key::Right,
        NamedKey::ArrowUp => Key::Up,
        NamedKey::ArrowDown => Key::Down,
        NamedKey::Backspace => Key::Backspace,
        NamedKey::Delete => Key::Delete,
        NamedKey::Enter => Key::Enter,
        NamedKey::Home => Key::Home,
        NamedKey::End => Key::End,
        NamedKey::PageUp => Key::PageUp,
        NamedKey::PageDown => Key::PageDown,
        NamedKey::Escape => Key::Escape,
        _ => return None,
    };

    Some(Message::KeyPressed(key))
}

pub fn text_message(text: String) -> Option<Message> {
    if text.is_empty() {
        None
    } else {
        Some(Message::TextInput(text))
    }
}

pub fn menu_message(x: f32, y: f32) -> Option<Message> {
    if !(0.0..24.0).contains(&y) {
        return None;
    }

    match x {
        0.0..70.0 | 70.0..150.0 => Some(Message::ToggleCommandBar),
        150.0..260.0 => Some(Message::OpenSettings),
        _ => None,
    }
}

pub fn scrollbar_message(frame: &FrameDescription, x: f32, y: f32) -> Option<Message> {
    if let Some(scrollbar) = frame.vertical_scrollbar.as_ref()
        && contains(scrollbar.track, x, y)
        && !contains(scrollbar.thumb, x, y)
    {
        return Some(Message::Scrolled {
            vertical: if y < scrollbar.thumb.y { -10 } else { 10 },
            horizontal: 0,
        });
    }
    if let Some(scrollbar) = frame.horizontal_scrollbar.as_ref()
        && contains(scrollbar.track, x, y)
        && !contains(scrollbar.thumb, x, y)
    {
        return Some(Message::Scrolled {
            vertical: 0,
            horizontal: if x < scrollbar.thumb.x { -8 } else { 8 },
        });
    }
    None
}

fn contains(rect: Rect, x: f32, y: f32) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

#[cfg(test)]
mod tests {
    use super::{is_shortcut, menu_message, named_key_message, scrollbar_message, text_message};
    use crate::mvu::{Key, Message};
    use crate::view::{Color, FrameDescription, Rect, Scrollbar, Size};
    use winit::keyboard::NamedKey;

    #[test]
    fn named_keys_become_semantic_editor_messages() {
        assert_eq!(
            named_key_message(NamedKey::ArrowLeft),
            Some(Message::KeyPressed(Key::Left))
        );
        assert_eq!(
            named_key_message(NamedKey::Backspace),
            Some(Message::KeyPressed(Key::Backspace))
        );
        assert_eq!(named_key_message(NamedKey::F1), None);
    }

    #[test]
    fn empty_ime_commits_are_ignored() {
        assert_eq!(text_message(String::new()), None);
        assert_eq!(
            text_message("hello".into()),
            Some(Message::TextInput("hello".into()))
        );
    }

    #[test]
    fn control_o_is_reserved_for_opening_a_workspace() {
        assert!(is_shortcut(
            &winit::keyboard::Key::Character("o".into()),
            "o"
        ));
        assert!(is_shortcut(
            &winit::keyboard::Key::Character("s".into()),
            "s"
        ));
    }

    #[test]
    fn menu_regions_dispatch_semantic_actions() {
        assert_eq!(menu_message(20.0, 10.0), Some(Message::ToggleCommandBar));
        assert_eq!(menu_message(110.0, 10.0), Some(Message::ToggleCommandBar));
        assert_eq!(menu_message(200.0, 10.0), Some(Message::OpenSettings));
        assert_eq!(menu_message(200.0, 30.0), None);
    }

    #[test]
    fn scrollbar_track_clicks_dispatch_page_scrolls() {
        let frame = FrameDescription {
            viewport: Size {
                width: 100.0,
                height: 100.0,
            },
            background: Color(0, 0, 0, 255),
            gutter: Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            line_numbers: Vec::new(),
            text_runs: Vec::new(),
            selections: Vec::new(),
            cursors: Vec::new(),
            vertical_scrollbar: Some(Scrollbar {
                track: Rect {
                    x: 90.0,
                    y: 0.0,
                    width: 10.0,
                    height: 100.0,
                },
                thumb: Rect {
                    x: 90.0,
                    y: 30.0,
                    width: 10.0,
                    height: 20.0,
                },
            }),
            horizontal_scrollbar: None,
            menu_bar: Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            command_bar: None,
            settings_panel: None,
            overlay_text: Vec::new(),
        };
        assert_eq!(
            scrollbar_message(&frame, 95.0, 10.0),
            Some(Message::Scrolled {
                vertical: -10,
                horizontal: 0
            })
        );
        assert_eq!(
            scrollbar_message(&frame, 95.0, 80.0),
            Some(Message::Scrolled {
                vertical: 10,
                horizontal: 0
            })
        );
        assert_eq!(scrollbar_message(&frame, 95.0, 40.0), None);
    }
}
