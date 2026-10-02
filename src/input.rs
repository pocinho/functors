use winit::{
    event::KeyEvent,
    keyboard::{Key as WinitKey, NamedKey},
};

use crate::mvu::{Key, Message};

pub fn key_message(event: &KeyEvent) -> Option<Message> {
    if !event.state.is_pressed() {
        return None;
    }

    match &event.logical_key {
        WinitKey::Named(key) => named_key_message(*key),
        WinitKey::Character(_) => None,
        WinitKey::Unidentified(_) | WinitKey::Dead(_) => None,
    }
}

pub fn key_text_message(event: &KeyEvent) -> Option<Message> {
    if !event.state.is_pressed()
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

#[cfg(test)]
mod tests {
    use super::{named_key_message, text_message};
    use crate::mvu::{Key, Message};
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
}
