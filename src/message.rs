use std::sync::mpsc;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::model::Adjust;

pub enum Message {
    Midi { timestamp: u64, bytes: Vec<u8> },
    NextWaveform,
    SelectCutoff,
    SelectAttack,
    SelectDecay,
    SelectSustain,
    SelectRelease,
    Adjust(Adjust),
    Toggle,
    Quit,
    NextPort,
    SetPortName(String),
    Continue,
}

fn key_to_message(key: KeyCode) -> Option<Message> {
    let msg = match key {
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
        KeyCode::Tab => Message::NextPort,
        _ => return None,
    };
    Some(msg)
}

fn handle_input(message_tx: &mpsc::Sender<Message>) -> bool {
    let Ok(event) = event::read() else {
        return false;
    };
    match event {
        Event::Key(key) => {
            if key.kind != KeyEventKind::Press {
                return true;
            }
            match key_to_message(key.code) {
                Some(msg) => message_tx.send(msg).is_ok(),
                None => true,
            }
        }
        Event::Resize(..) => message_tx.send(Message::Continue).is_ok(),
        _ => true,
    }
}

pub fn input_loop(message_tx: mpsc::Sender<Message>) {
    while handle_input(&message_tx) {}
}
