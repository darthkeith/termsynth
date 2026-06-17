use crate::{
    command::Command,
    message::Message,
    model::{Model, Param},
};

const STATUS_MASK: u8 = 0xF0;
const STATUS_NOTE_ON: u8 = 0x90;
const STATUS_NOTE_OFF: u8 = 0x80;

pub fn update(mut model: Model, message: Message) -> Option<(Model, Command)> {
    let command = match message {
        Message::Midi { timestamp, bytes } => {
            model.last_midi = Some((timestamp, bytes.clone()));
            let (status_byte, note_byte) = (bytes[0], bytes[1]);
            match status_byte & STATUS_MASK {
                STATUS_NOTE_ON => {
                    model.note_state = Some(note_byte);
                    let freq =
                        440.0 * 2.0f32.powf((note_byte as f32 - 69.0) / 12.0);
                    Command::NoteOn(freq)
                }
                STATUS_NOTE_OFF => {
                    if model.note_state == Some(note_byte) {
                        Command::NoteOff
                    } else {
                        Command::None
                    }
                }
                _ => Command::None,
            }
        }
        Message::NextWaveform => {
            model.waveform = model.waveform.next();
            Command::SetWaveform(model.waveform)
        }
        Message::SelectCutoff => {
            model.selected = Param::Cutoff;
            Command::None
        }
        Message::SelectAttack => {
            model.selected = Param::Attack;
            Command::None
        }
        Message::SelectDecay => {
            model.selected = Param::Decay;
            Command::None
        }
        Message::SelectSustain => {
            model.selected = Param::Sustain;
            Command::None
        }
        Message::SelectRelease => {
            model.selected = Param::Release;
            Command::None
        }
        Message::Adjust(adjust) => {
            model.adjust(adjust);
            match model.selected {
                Param::Cutoff => Command::SetCutoff(model.cutoff),
                Param::Attack => Command::SetAttack(model.adsr.attack),
                Param::Decay => Command::SetDecay(model.adsr.decay),
                Param::Sustain => Command::SetSustain(model.adsr.sustain),
                Param::Release => Command::SetRelease(model.adsr.release),
            }
        }
        Message::Toggle => match model.note_state {
            Some(_) => {
                model.note_state = None;
                Command::NoteOff
            }
            None => {
                model.note_state = Some(69);
                Command::NoteOn(440.0)
            }
        },
        Message::Quit => return None,
        Message::NextPort => Command::NextPort,
        Message::SetPortName(port_name) => {
            model.port_name = port_name;
            Command::None
        }
        Message::Continue => Command::None,
    };
    Some((model, command))
}
