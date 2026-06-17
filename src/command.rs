use crate::{
    audio::{Audio, AudioUpdate},
    midi::Midi,
    model::Waveform,
};

pub enum Command {
    NoteOn(f32),
    NoteOff,
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
        Command::NoteOn(freq) => {
            audio.audio_tx.send(AudioUpdate::NoteOn(freq)).unwrap()
        }
        Command::NoteOff => audio.audio_tx.send(AudioUpdate::NoteOff).unwrap(),
        Command::SetWaveform(waveform) => {
            audio
                .audio_tx
                .send(AudioUpdate::Waveform(waveform))
                .unwrap();
        }
        Command::SetCutoff(cutoff) => {
            audio.audio_tx.send(AudioUpdate::Cutoff(cutoff)).unwrap();
        }
        Command::SetAttack(attack) => {
            audio.audio_tx.send(AudioUpdate::Attack(attack)).unwrap();
        }
        Command::SetDecay(decay) => {
            audio.audio_tx.send(AudioUpdate::Decay(decay)).unwrap();
        }
        Command::SetSustain(sustain) => {
            audio.audio_tx.send(AudioUpdate::Sustain(sustain)).unwrap();
        }
        Command::SetRelease(release) => {
            audio.audio_tx.send(AudioUpdate::Release(release)).unwrap();
        }
        Command::NextPort => midi.next_port(),
        Command::None => (),
    }
}
