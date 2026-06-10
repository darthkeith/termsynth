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

const FREQ: f32 = 440.0;

pub struct Audio {
    _stream: Stream,
    is_on: Arc<AtomicBool>,
}

pub enum Command {
    PlayNote,
    StopNote,
    None,
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
            let sample_rate = config.sample_rate() as f32;
            let phase_increment = 1.0 / sample_rate;
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                if is_on.load(Ordering::Relaxed) {
                    for sample in data.iter_mut() {
                        *sample = (2.0 * PI * FREQ * phase).sin();
                        phase = (phase + phase_increment) % 1.0;
                    }
                } else {
                    phase = 0.0;
                    for sample in data.iter_mut() {
                        *sample = 0.0;
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
