use std::sync::mpsc;

use midir::{MidiInput, MidiInputConnection};

use crate::message::Message;

const CLIENT_NAME: &str = "Terminal Synth";

pub struct Midi {
    message_tx: mpsc::Sender<Message>,
    port_index: usize,
    connection: Option<MidiInputConnection<()>>,
}

impl Midi {
    pub fn new(message_tx: mpsc::Sender<Message>) -> Self {
        let mut midi = Self {
            message_tx,
            port_index: 0,
            connection: None,
        };
        midi.connect().expect("failed to connect to MIDI input");
        midi
    }

    pub fn next_port(&mut self) {
        self.port_index += 1;
    }

    pub fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let midi_in = MidiInput::new(CLIENT_NAME)?;
        let ports = midi_in.ports();
        if ports.is_empty() {
            self.connection = None;
            return Ok(());
        }
        if self.port_index >= ports.len() {
            self.port_index = 0;
        }
        let port = &ports[self.port_index];
        self.message_tx
            .send(Message::SetPortName(midi_in.port_name(&port)?))
            .unwrap();
        let msg_tx = self.message_tx.clone();
        self.connection = midi_in
            .connect(
                port,
                CLIENT_NAME,
                move |timestamp, message, _| {
                    msg_tx
                        .send(Message::Midi {
                            timestamp,
                            bytes: message.to_vec(),
                        })
                        .expect("failed to send MIDI message")
                },
                (),
            )
            .map(Some)?;
        Ok(())
    }
}
