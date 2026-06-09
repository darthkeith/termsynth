mod message;
mod model;
mod update;
mod view;

use std::io::Result;

use ratatui::DefaultTerminal;

use crate::{message::handle_input, model::Model, update::update, view::view};

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let mut model = Model;
    loop {
        terminal.draw(|frame| view(frame))?;
        let message = handle_input()?;
        model = match update(message) {
            Some(model) => model,
            None => return Ok(()),
        }
    }
}

fn main() -> Result<()> {
    ratatui::run(run)
}
