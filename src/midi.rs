use std::sync::mpsc;

use crate::message::Message;

const CLIENT_NAME: &str = "Terminal Synth";

struct ActiveMidiConnection {
    port_index: usize,
    port_name: String,
    _connection: midir::MidiInputConnection<()>,
}

pub struct Midi {
    message_tx: mpsc::Sender<Message>,
    active: Option<ActiveMidiConnection>,
}

fn connect(
    port_index: usize,
    message_tx: mpsc::Sender<Message>,
) -> Option<ActiveMidiConnection> {
    let midi_in = midir::MidiInput::new(CLIENT_NAME).ok()?;
    let ports = midi_in.ports();
    if port_index >= ports.len() {
        return None;
    }
    let port = &ports[port_index];
    let port_name = midi_in.port_name(&port).ok()?;
    let _connection = midi_in
        .connect(
            port,
            CLIENT_NAME,
            move |timestamp, message, _| {
                message_tx
                    .send(Message::Midi {
                        timestamp,
                        bytes: message.to_vec(),
                    })
                    .expect("failed to send MIDI message")
            },
            (),
        )
        .ok()?;
    Some(ActiveMidiConnection {
        port_index,
        port_name,
        _connection,
    })
}

impl Midi {
    pub fn new(message_tx: mpsc::Sender<Message>) -> Self {
        Self {
            message_tx,
            active: None,
        }
    }

    pub fn next_port(&mut self) {
        let msg_tx = self.message_tx.clone();
        self.active = match &self.active {
            Some(current) => connect(current.port_index + 1, msg_tx),
            None => connect(0, msg_tx),
        };
        let port_name = self.active.as_ref().map(|c| c.port_name.clone());
        self.message_tx
            .send(Message::SetPortName(port_name))
            .unwrap();
    }
}
