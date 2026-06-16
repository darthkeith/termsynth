use crate::{
    command::Command,
    message::Message,
    model::{Model, Param},
};

pub fn update(mut model: Model, message: Message) -> Option<(Model, Command)> {
    match message {
        Message::Midi { timestamp, bytes } => {
            model.last_midi = Some((timestamp, bytes));
        }
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
        Message::Adjust(adjust) => {
            model.adjust(adjust);
            let command = match model.selected {
                Param::Cutoff => Command::SetCutoff(model.cutoff),
                Param::Attack => Command::SetAttack(model.adsr.attack),
                Param::Decay => Command::SetDecay(model.adsr.decay),
                Param::Sustain => Command::SetSustain(model.adsr.sustain),
                Param::Release => Command::SetRelease(model.adsr.release),
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
        Message::Quit => return None,
        Message::NextPort => {
            return Some((model, Command::NextPort));
        }
        Message::SetPortName(port_name) => model.port_name = Some(port_name),
        Message::Continue => (),
    };
    Some((model, Command::None))
}
