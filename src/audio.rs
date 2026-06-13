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

pub enum EnvelopeStage {
    Attack(f32),
    Decay(f32),
    Sustain,
    Release(f32),
    Idle,
}

pub struct AudioProcessor {
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
    waveform_rx: mpsc::Receiver<Waveform>,
    cutoff_rx: mpsc::Receiver<f32>,
    adsr_rx: mpsc::Receiver<Adsr>,
}

pub struct Audio {
    _stream: Stream,
    is_on: Arc<AtomicBool>,
    waveform_tx: mpsc::Sender<Waveform>,
    cutoff_tx: mpsc::Sender<f32>,
    adsr_tx: mpsc::Sender<Adsr>,
}

pub enum Command {
    PlayNote,
    StopNote,
    SetWaveform(Waveform),
    SetCutoff(f32),
    SetAdsr(Adsr),
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

fn cutoff_to_alpha(cutoff: f32, sample_rate: f32) -> f32 {
    1.0 - (-2.0 * PI * cutoff / sample_rate).exp()
}

impl AudioProcessor {
    fn new(
        channels: usize,
        sample_rate: f32,
        is_on: Arc<AtomicBool>,
        waveform_rx: mpsc::Receiver<Waveform>,
        cutoff_rx: mpsc::Receiver<f32>,
        adsr_rx: mpsc::Receiver<Adsr>,
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
            waveform_rx,
            cutoff_rx,
            adsr_rx,
        }
    }

    fn receive_param_updates(&mut self) {
        if let Ok(new_waveform) = self.waveform_rx.try_recv() {
            self.waveform = new_waveform;
        }
        if let Ok(cutoff) = self.cutoff_rx.try_recv() {
            self.alpha = cutoff_to_alpha(cutoff, self.sample_rate);
        }
        if let Ok(new_adsr) = self.adsr_rx.try_recv() {
            self.adsr = new_adsr;
            self.attack_increment = 1.0 / (self.adsr.attack * self.sample_rate);
            self.decay_increment = 1.0 / (self.adsr.decay * self.sample_rate);
            self.release_increment =
                1.0 / (self.adsr.release * self.sample_rate);
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
        let (waveform_tx, waveform_rx) = mpsc::channel::<Waveform>();
        let (cutoff_tx, cutoff_rx) = mpsc::channel::<f32>();
        let (adsr_tx, adsr_rx) = mpsc::channel::<Adsr>();
        let mut processor = AudioProcessor::new(
            config.channels() as usize,
            config.sample_rate() as f32,
            is_on.clone(),
            waveform_rx,
            cutoff_rx,
            adsr_rx,
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
            _stream,
            is_on,
            waveform_tx,
            cutoff_tx,
            adsr_tx,
        }
    }
}

pub fn execute_command(command: Command, audio: &Audio) {
    match command {
        Command::PlayNote => audio.is_on.store(true, Ordering::Relaxed),
        Command::StopNote => audio.is_on.store(false, Ordering::Relaxed),
        Command::SetCutoff(cutoff) => audio.cutoff_tx.send(cutoff).unwrap(),
        Command::SetWaveform(waveform) => {
            audio.waveform_tx.send(waveform).unwrap()
        }
        Command::SetAdsr(adsr) => audio.adsr_tx.send(adsr).unwrap(),
        Command::None => (),
    }
}
