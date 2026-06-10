use cpal::{
    Sample, Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

const FREQ: f32 = 440.0;

pub struct Audio {
    stream: Stream,
}

pub enum Command {
    PlayNote,
    StopNote,
    None,
}

fn write_silence(data: &mut [f32], _: &cpal::OutputCallbackInfo) {
    for sample in data.iter_mut() {
        *sample = Sample::EQUILIBRIUM;
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
        let stream = device
            .build_output_stream(
                config.into(),
                write_silence,
                |err| eprintln!("audio stream error: {}", err),
                None,
            )
            .unwrap();
        stream.play().unwrap();
        Self { stream }
    }
}

pub fn execute_command(command: Command, audio: &Audio) {
    match command {
        Command::PlayNote => (),
        Command::StopNote => (),
        Command::None => (),
    }
}
