mod audio;
mod message;
mod model;
mod update;
mod view;

use std::io::Result;

use ratatui::DefaultTerminal;

use crate::{
    audio::{Audio, execute_command},
    message::handle_input,
    model::Model,
    update::update,
    view::view,
};

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let mut model = Model::new();
    let audio = Audio::new();
    loop {
        terminal.draw(|frame| view(&model, frame))?;
        let message = handle_input()?;
        let command;
        (model, command) = match update(model, message) {
            Some(result) => result,
            None => return Ok(()),
        };
        execute_command(command, &audio);
    }
}

fn main() -> Result<()> {
    ratatui::run(run)
}
