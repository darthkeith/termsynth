use std::sync::atomic::Ordering;

use crate::{
    audio::{Audio, ParamUpdate},
    midi::Midi,
    model::Waveform,
};

pub enum Command {
    PlayNote,
    StopNote,
    SetWaveform(Waveform),
    SetCutoff(f32),
    SetAttack(f32),
    SetDecay(f32),
    SetSustain(f32),
    SetRelease(f32),
    NextPort,
    None,
}

pub fn execute_command(command: Command, audio: &Audio, midi: &mut Midi) {
    match command {
        Command::PlayNote => audio.is_on.store(true, Ordering::Relaxed),
        Command::StopNote => audio.is_on.store(false, Ordering::Relaxed),
        Command::SetWaveform(waveform) => {
            audio
                .param_tx
                .send(ParamUpdate::Waveform(waveform))
                .unwrap();
        }
        Command::SetCutoff(cutoff) => {
            audio.param_tx.send(ParamUpdate::Cutoff(cutoff)).unwrap();
        }
        Command::SetAttack(attack) => {
            audio.param_tx.send(ParamUpdate::Attack(attack)).unwrap();
        }
        Command::SetDecay(decay) => {
            audio.param_tx.send(ParamUpdate::Decay(decay)).unwrap();
        }
        Command::SetSustain(sustain) => {
            audio.param_tx.send(ParamUpdate::Sustain(sustain)).unwrap();
        }
        Command::SetRelease(release) => {
            audio.param_tx.send(ParamUpdate::Release(release)).unwrap();
        }
        Command::NextPort => {
            midi.next_port();
            midi.connect().unwrap();
        }
        Command::None => (),
    }
}
