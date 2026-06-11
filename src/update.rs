use crate::{
    audio::Command,
    message::Message,
    model::{Model, Param},
};

pub fn update(mut model: Model, message: Message) -> Option<(Model, Command)> {
    match message {
        Message::NextWaveform => model.waveform = model.waveform.next(),
        Message::SelectAttack => model.selected = Param::Attack,
        Message::SelectDecay => model.selected = Param::Decay,
        Message::SelectSustain => model.selected = Param::Sustain,
        Message::SelectRelease => model.selected = Param::Release,
        Message::Increment => {
            model.adsr.increment(&model.selected);
            let adsr = model.adsr;
            return Some((model, Command::SetAdsr(adsr)));
        }
        Message::Decrement => {
            model.adsr.decrement(&model.selected);
            let adsr = model.adsr;
            return Some((model, Command::SetAdsr(adsr)));
        }
        Message::Toggle => {
            model.is_on = !model.is_on;
            let command = if model.is_on {
                Command::PlayNote
            } else {
                Command::StopNote
            };
            return Some((model, command));
        }
        Message::Continue => (),
        Message::Quit => return None,
    };
    Some((model, Command::None))
}
