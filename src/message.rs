use std::sync::mpsc;

use crossterm::event::{self, KeyCode, KeyEventKind};

use crate::model::Adjust;

pub enum Message {
    NextWaveform,
    SelectCutoff,
    SelectAttack,
    SelectDecay,
    SelectSustain,
    SelectRelease,
    Adjust(Adjust),
    Toggle,
    Continue,
    Quit,
}

fn key_to_message(key: KeyCode) -> Message {
    match key {
        KeyCode::Char('w') => Message::NextWaveform,
        KeyCode::Char('c') => Message::SelectCutoff,
        KeyCode::Char('a') => Message::SelectAttack,
        KeyCode::Char('d') => Message::SelectDecay,
        KeyCode::Char('s') => Message::SelectSustain,
        KeyCode::Char('r') => Message::SelectRelease,
        KeyCode::Char('k') => Message::Adjust(Adjust::Increase),
        KeyCode::Char('j') => Message::Adjust(Adjust::Decrease),
        KeyCode::Char(' ') => Message::Toggle,
        KeyCode::Char('q') => Message::Quit,
        _ => Message::Continue,
    }
}

pub fn handle_input(message_tx: &mpsc::Sender<Message>) -> bool {
    let Ok(event::Event::Key(key)) = event::read() else {
        return message_tx.send(Message::Continue).is_ok();
    };
    if key.kind != KeyEventKind::Press {
        return true;
    }
    message_tx.send(key_to_message(key.code)).is_ok()
}
