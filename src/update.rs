use crate::{audio::Command, message::Message, model::Model};

pub fn update(mut model: Model, message: Message) -> Option<(Model, Command)> {
    match message {
        Message::Toggle => model.is_on = !model.is_on,
        Message::Continue => (),
        Message::Quit => return None,
    }
    Some((model, Command::None))
}
