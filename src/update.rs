use crate::{audio::Command, message::Message, model::Model};

pub fn update(mut model: Model, message: Message) -> Option<(Model, Command)> {
    let result = match message {
        Message::Toggle => {
            model.is_on = !model.is_on;
            let command = if model.is_on {
                Command::PlayNote
            } else {
                Command::StopNote
            };
            (model, command)
        }
        Message::Continue => (model, Command::None),
        Message::Quit => return None,
    };
    Some(result)
}
