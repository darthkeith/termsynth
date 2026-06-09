use std::io::Result;

use crossterm::event::{self, KeyCode, KeyEventKind};

pub enum Message {
    Toggle,
    Continue,
    Quit,
}

fn key_to_message(key: KeyCode) -> Message {
    match key {
        KeyCode::Char(' ') => Message::Toggle,
        KeyCode::Char('q') => Message::Quit,
        _ => Message::Continue,
    }
}

pub fn handle_input() -> Result<Message> {
    let event::Event::Key(key) = event::read()? else {
        return Ok(Message::Continue);
    };
    if key.kind != KeyEventKind::Press {
        return Ok(Message::Continue);
    }
    Ok(key_to_message(key.code))
}
