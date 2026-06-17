use std::{f32::consts::PI, sync::mpsc};

use cpal::{
    Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::model::{Adsr, DEFAULT_CUTOFF, Waveform};

enum EnvelopeStage {
    Attack(f32),
    Decay(f32),
    Sustain,
    Release(f32),
    Idle,
}

pub enum AudioUpdate {
    NoteOn(f32),
    NoteOff,
    Waveform(Waveform),
    Cutoff(f32),
    Attack(f32),
    Decay(f32),
    Sustain(f32),
    Release(f32),
}

struct AudioProcessor {
    is_on: bool,
    waveform: Waveform,
    alpha: f32,
    envelope_stage: EnvelopeStage,
    phase: f32,
    channels: usize,
    sample_rate: f32,
    sustain: f32,
    phase_increment: f32,
    attack_increment: f32,
    decay_increment: f32,
    release_increment: f32,
    prev_output: f32,
    audio_rx: mpsc::Receiver<AudioUpdate>,
}

pub struct Audio {
    pub audio_tx: mpsc::Sender<AudioUpdate>,
    _stream: Stream,
}

fn cutoff_to_alpha(cutoff: f32, sample_rate: f32) -> f32 {
    1.0 - (-2.0 * PI * cutoff / sample_rate).exp()
}

impl AudioProcessor {
    fn new(
        channels: usize,
        sample_rate: f32,
        audio_rx: mpsc::Receiver<AudioUpdate>,
    ) -> Self {
        let adsr = Adsr::new();
        Self {
            is_on: false,
            waveform: Waveform::Sine,
            alpha: cutoff_to_alpha(DEFAULT_CUTOFF, sample_rate),
            envelope_stage: EnvelopeStage::Idle,
            phase: 0.0,
            channels,
            sample_rate,
            sustain: adsr.sustain,
            phase_increment: 0.0,
            attack_increment: 1.0 / (adsr.attack * sample_rate),
            decay_increment: 1.0 / (adsr.decay * sample_rate),
            release_increment: 1.0 / (adsr.release * sample_rate),
            prev_output: 0.0,
            audio_rx,
        }
    }

    fn receive_updates(&mut self) {
        while let Ok(update) = self.audio_rx.try_recv() {
            match update {
                AudioUpdate::NoteOn(freq) => {
                    self.is_on = true;
                    self.phase_increment = freq / self.sample_rate;
                }
                AudioUpdate::NoteOff => self.is_on = false,
                AudioUpdate::Waveform(waveform) => self.waveform = waveform,
                AudioUpdate::Cutoff(cutoff) => {
                    self.alpha = cutoff_to_alpha(cutoff, self.sample_rate);
                }
                AudioUpdate::Attack(attack) => {
                    self.attack_increment = 1.0 / (attack * self.sample_rate);
                }
                AudioUpdate::Decay(decay) => {
                    self.decay_increment = 1.0 / (decay * self.sample_rate);
                }
                AudioUpdate::Sustain(sustain) => self.sustain = sustain,
                AudioUpdate::Release(release) => {
                    self.release_increment = 1.0 / (release * self.sample_rate);
                }
            }
        }
    }

    fn envelope(&self) -> f32 {
        match self.envelope_stage {
            EnvelopeStage::Attack(t) => t,
            EnvelopeStage::Decay(t) => 1.0 + t * (self.sustain - 1.0),
            EnvelopeStage::Sustain => self.sustain,
            EnvelopeStage::Release(t) => (1.0 - t) * self.sustain,
            EnvelopeStage::Idle => 0.0,
        }
    }

    fn update_envelope_stage(&mut self) {
        if self.is_on {
            match self.envelope_stage {
                EnvelopeStage::Release(_) | EnvelopeStage::Idle => {
                    self.envelope_stage =
                        EnvelopeStage::Attack(self.envelope());
                }
                _ => (),
            };
        } else {
            match self.envelope_stage {
                EnvelopeStage::Attack(_) | EnvelopeStage::Decay(_) => {
                    let t = 1.0 - (self.envelope() / self.sustain);
                    self.envelope_stage = EnvelopeStage::Release(t);
                }
                EnvelopeStage::Sustain => {
                    self.envelope_stage = EnvelopeStage::Release(0.0)
                }
                _ => (),
            };
        }
    }

    fn generate_sample(&self) -> f32 {
        match self.waveform {
            Waveform::Sine => (2.0 * PI * self.phase).sin(),
            Waveform::Square => {
                if self.phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Saw => 1.0 - 2.0 * self.phase,
            Waveform::Triangle => 4.0 * (self.phase - 0.5).abs() - 1.0,
        }
    }

    fn apply_filter(&mut self, input: f32) -> f32 {
        self.prev_output += self.alpha * (input - self.prev_output);
        self.prev_output
    }

    fn increment_envelope(&mut self) {
        self.envelope_stage = match self.envelope_stage {
            EnvelopeStage::Attack(mut t) => {
                t += self.attack_increment;
                if t < 1.0 {
                    EnvelopeStage::Attack(t)
                } else {
                    EnvelopeStage::Decay(0.0)
                }
            }
            EnvelopeStage::Decay(mut t) => {
                t += self.decay_increment;
                if t < 1.0 {
                    EnvelopeStage::Decay(t)
                } else {
                    EnvelopeStage::Sustain
                }
            }
            EnvelopeStage::Sustain => EnvelopeStage::Sustain,
            EnvelopeStage::Release(mut t) => {
                t += self.release_increment;
                if t < 1.0 {
                    EnvelopeStage::Release(t)
                } else {
                    EnvelopeStage::Idle
                }
            }
            EnvelopeStage::Idle => EnvelopeStage::Idle,
        }
    }

    fn process(&mut self, data: &mut [f32]) {
        self.receive_updates();
        self.update_envelope_stage();
        for frame in data.chunks_mut(self.channels) {
            let value =
                self.apply_filter(self.generate_sample()) * self.envelope();
            for sample in frame.iter_mut() {
                *sample = value;
            }
            self.phase = (self.phase + self.phase_increment) % 1.0;
            self.increment_envelope();
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
        let (audio_tx, audio_rx) = mpsc::channel::<AudioUpdate>();
        let mut processor = AudioProcessor::new(
            config.channels() as usize,
            config.sample_rate() as f32,
            audio_rx,
        );
        let _stream = device
            .build_output_stream(
                config.into(),
                move |data, _| processor.process(data),
                |err| eprintln!("audio stream error: {}", err),
                None,
            )
            .unwrap();
        _stream.play().unwrap();
        Self { audio_tx, _stream }
    }
}
