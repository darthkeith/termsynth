mod audio;
mod message;
mod midi;
mod model;
mod update;
mod view;

use std::{io::Result, sync::mpsc, thread};

use ratatui::DefaultTerminal;

use crate::{
    audio::{Audio, execute_command},
    message::{Message, input_loop},
    midi::Midi,
    model::Model,
    update::update,
    view::view,
};

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let mut model = Model::new();
    let audio = Audio::new();
    let (message_tx, message_rx) = mpsc::channel::<Message>();
    let mut midi = Midi::new(message_tx.clone());
    thread::spawn(move || input_loop(message_tx));
    loop {
        terminal.draw(|frame| view(&model, frame))?;
        let mut message = message_rx.recv().expect("input thread disconnected");
        loop {
            let (next_model, command) = match update(model, message) {
                Some(result) => result,
                None => return Ok(()),
            };
            model = next_model;
            execute_command(command, &audio, &mut midi);
            match message_rx.try_recv() {
                Ok(next_msg) => message = next_msg,
                Err(_) => break,
            }
        }
    }
}

fn main() -> Result<()> {
    ratatui::run(run)
}
