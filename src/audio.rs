use std::{
    f32::consts::PI,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};

use cpal::{
    Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::model::{Adsr, DEFAULT_CUTOFF, Waveform};

const FREQ: f32 = 440.0;

enum EnvelopeStage {
    Attack(f32),
    Decay(f32),
    Sustain,
    Release(f32),
    Idle,
}

pub enum ParamUpdate {
    Waveform(Waveform),
    Cutoff(f32),
    Attack(f32),
    Decay(f32),
    Sustain(f32),
    Release(f32),
}

struct AudioProcessor {
    waveform: Waveform,
    alpha: f32,
    adsr: Adsr,
    envelope_stage: EnvelopeStage,
    phase: f32,
    channels: usize,
    sample_rate: f32,
    phase_increment: f32,
    attack_increment: f32,
    decay_increment: f32,
    release_increment: f32,
    prev_output: f32,
    is_on: Arc<AtomicBool>,
    param_rx: mpsc::Receiver<ParamUpdate>,
}

pub struct Audio {
    pub is_on: Arc<AtomicBool>,
    pub param_tx: mpsc::Sender<ParamUpdate>,
    _stream: Stream,
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

fn cutoff_to_alpha(cutoff: f32, sample_rate: f32) -> f32 {
    1.0 - (-2.0 * PI * cutoff / sample_rate).exp()
}

impl AudioProcessor {
    fn new(
        channels: usize,
        sample_rate: f32,
        is_on: Arc<AtomicBool>,
        param_rx: mpsc::Receiver<ParamUpdate>,
    ) -> Self {
        let alpha = cutoff_to_alpha(DEFAULT_CUTOFF, sample_rate);
        let adsr = Adsr::new();
        Self {
            waveform: Waveform::Sine,
            alpha,
            adsr,
            envelope_stage: EnvelopeStage::Idle,
            phase: 0.0,
            channels,
            sample_rate,
            phase_increment: FREQ / sample_rate,
            attack_increment: 1.0 / (adsr.attack * sample_rate),
            decay_increment: 1.0 / (adsr.decay * sample_rate),
            release_increment: 1.0 / (adsr.release * sample_rate),
            prev_output: 0.0,
            is_on,
            param_rx,
        }
    }

    fn receive_param_updates(&mut self) {
        while let Ok(update) = self.param_rx.try_recv() {
            match update {
                ParamUpdate::Waveform(waveform) => self.waveform = waveform,
                ParamUpdate::Cutoff(cutoff) => {
                    self.alpha = cutoff_to_alpha(cutoff, self.sample_rate);
                }
                ParamUpdate::Attack(attack) => {
                    self.attack_increment = 1.0 / (attack * self.sample_rate);
                    self.adsr.attack = attack;
                }
                ParamUpdate::Decay(decay) => {
                    self.decay_increment = 1.0 / (decay * self.sample_rate);
                    self.adsr.decay = decay;
                }
                ParamUpdate::Sustain(sustain) => self.adsr.sustain = sustain,
                ParamUpdate::Release(release) => {
                    self.release_increment = 1.0 / (release * self.sample_rate);
                    self.adsr.release = release;
                }
            }
        }
    }

    fn update_envelope_on_signal(&mut self) {
        if self.is_on.load(Ordering::Relaxed) {
            match self.envelope_stage {
                EnvelopeStage::Release(_) | EnvelopeStage::Idle => {
                    let t = self.envelope_stage.amplitude(self.adsr.sustain);
                    self.envelope_stage = EnvelopeStage::Attack(t);
                }
                _ => (),
            };
        } else {
            match self.envelope_stage {
                EnvelopeStage::Attack(_) | EnvelopeStage::Decay(_) => {
                    let amplitude =
                        self.envelope_stage.amplitude(self.adsr.sustain);
                    let t = 1.0 - (amplitude / self.adsr.sustain);
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
        self.receive_param_updates();
        self.update_envelope_on_signal();
        for frame in data.chunks_mut(self.channels) {
            let envelope = self.envelope_stage.amplitude(self.adsr.sustain);
            let value = self.apply_filter(self.generate_sample()) * envelope;
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
        let is_on = Arc::new(AtomicBool::new(false));
        let (param_tx, param_rx) = mpsc::channel::<ParamUpdate>();
        let mut processor = AudioProcessor::new(
            config.channels() as usize,
            config.sample_rate() as f32,
            is_on.clone(),
            param_rx,
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
        Self {
            is_on,
            param_tx,
            _stream,
        }
    }
}
