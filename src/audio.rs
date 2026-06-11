use std::{
    f32::consts::PI,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use cpal::{
    Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::model::Adsr;

const FREQ: f32 = 440.0;

pub enum EnvelopeStage {
    Attack(f32),
    Decay(f32),
    Sustain,
    Release(f32),
    Idle,
}

pub struct Audio {
    _stream: Stream,
    is_on: Arc<AtomicBool>,
}

pub enum Command {
    PlayNote,
    StopNote,
    None,
}

impl EnvelopeStage {
    fn amplitude(&self, sustain: f32) -> f32 {
        match self {
            Self::Attack(t) => *t,
            Self::Decay(t) => 1.0 + *t * (sustain - 1.0),
            Self::Sustain => sustain,
            Self::Release(t) => (1.0 - *t) * sustain,
            Self::Idle => 0.0,
        }
    }
}

impl Audio {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no output device available");
        let config = device
            .default_output_config()
            .expect("no default output config");
        match config.sample_format() {
            cpal::SampleFormat::F32 => (),
            _ => panic!("unsupported sample format"),
        }
        let is_on = Arc::new(AtomicBool::new(false));
        let write_sin = {
            let is_on = is_on.clone();
            let mut phase = 0.0;
            let mut envelope_stage = EnvelopeStage::Idle;
            let adsr = Adsr::new();
            let sample_rate = config.sample_rate() as f32;
            let phase_increment = 1.0 / sample_rate;
            let attack_increment = 1.0 / (adsr.attack * sample_rate);
            let decay_increment = 1.0 / (adsr.decay * sample_rate);
            let release_increment = 1.0 / (adsr.release * sample_rate);
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if is_on.load(Ordering::Relaxed) {
                    match envelope_stage {
                        EnvelopeStage::Release(_) | EnvelopeStage::Idle => {
                            let t = envelope_stage.amplitude(adsr.sustain);
                            envelope_stage = EnvelopeStage::Attack(t);
                        }
                        _ => (),
                    };
                } else {
                    match envelope_stage {
                        EnvelopeStage::Attack(_) | EnvelopeStage::Decay(_) => {
                            let amplitude =
                                envelope_stage.amplitude(adsr.sustain);
                            let t = 1.0 - (amplitude / adsr.sustain);
                            envelope_stage = EnvelopeStage::Release(t);
                        }
                        EnvelopeStage::Sustain => {
                            envelope_stage = EnvelopeStage::Release(0.0)
                        }
                        _ => (),
                    };
                }
                for sample in data.iter_mut() {
                    let envelope = envelope_stage.amplitude(adsr.sustain);
                    *sample = (2.0 * PI * FREQ * phase).sin() * envelope;
                    phase = (phase + phase_increment) % 1.0;
                    envelope_stage = match envelope_stage {
                        EnvelopeStage::Attack(mut t) => {
                            t += attack_increment;
                            if t < 1.0 {
                                EnvelopeStage::Attack(t)
                            } else {
                                EnvelopeStage::Decay(0.0)
                            }
                        }
                        EnvelopeStage::Decay(mut t) => {
                            t += decay_increment;
                            if t < 1.0 {
                                EnvelopeStage::Decay(t)
                            } else {
                                EnvelopeStage::Sustain
                            }
                        }
                        EnvelopeStage::Sustain => EnvelopeStage::Sustain,
                        EnvelopeStage::Release(mut t) => {
                            t += release_increment;
                            if t < 1.0 {
                                EnvelopeStage::Release(t)
                            } else {
                                EnvelopeStage::Idle
                            }
                        }
                        EnvelopeStage::Idle => EnvelopeStage::Idle,
                    }
                }
            }
        };
        let _stream = device
            .build_output_stream(
                config.into(),
                write_sin,
                |err| eprintln!("audio stream error: {}", err),
                None,
            )
            .unwrap();
        _stream.play().unwrap();
        Self { _stream, is_on }
    }
}

pub fn execute_command(command: Command, audio: &Audio) {
    match command {
        Command::PlayNote => audio.is_on.store(true, Ordering::Relaxed),
        Command::StopNote => audio.is_on.store(false, Ordering::Relaxed),
        Command::None => (),
    }
}
