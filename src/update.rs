use crate::{
    audio::Command,
    message::Message,
    model::{Model, Param},
};

pub fn update(mut model: Model, message: Message) -> Option<(Model, Command)> {
    match message {
        Message::NextWaveform => {
            model.waveform = model.waveform.next();
            let waveform = model.waveform;
            return Some((model, Command::SetWaveform(waveform)));
        }
        Message::SelectCutoff => model.selected = Param::Cutoff,
        Message::SelectAttack => model.selected = Param::Attack,
        Message::SelectDecay => model.selected = Param::Decay,
        Message::SelectSustain => model.selected = Param::Sustain,
        Message::SelectRelease => model.selected = Param::Release,
        Message::Increment => {
            model.increment();
            let command = match model.selected {
                Param::Cutoff => Command::SetCutoff(model.cutoff),
                _ => Command::SetAdsr(model.adsr),
            };
            return Some((model, command));
        }
        Message::Decrement => {
            model.decrement();
            let command = match model.selected {
                Param::Cutoff => Command::SetCutoff(model.cutoff),
                _ => Command::SetAdsr(model.adsr),
            };
            return Some((model, command));
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
