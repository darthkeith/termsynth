use crate::{message::Message, model::Model};

pub fn update(message: Message) -> Option<Model> {
    match message {
        Message::Continue => Some(Model),
        Message::Quit => None,
    }
}
