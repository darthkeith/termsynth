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

use crate::model::{Adsr, Waveform};

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
    adsr: Adsr,
    envelope_stage: EnvelopeStage,
    phase: f32,
    sample_rate: f32,
    phase_increment: f32,
    attack_increment: f32,
    decay_increment: f32,
    release_increment: f32,
    is_on: Arc<AtomicBool>,
    waveform_rx: mpsc::Receiver<Waveform>,
    adsr_rx: mpsc::Receiver<Adsr>,
}

pub struct Audio {
    _stream: Stream,
    is_on: Arc<AtomicBool>,
    waveform_tx: mpsc::Sender<Waveform>,
    adsr_tx: mpsc::Sender<Adsr>,
}

pub enum Command {
    PlayNote,
    StopNote,
    SetWaveform(Waveform),
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

impl AudioProcessor {
    fn new(
        sample_rate: f32,
        is_on: Arc<AtomicBool>,
        waveform_rx: mpsc::Receiver<Waveform>,
        adsr_rx: mpsc::Receiver<Adsr>,
    ) -> Self {
        let adsr = Adsr::new();
        Self {
            waveform: Waveform::Sine,
            adsr,
            envelope_stage: EnvelopeStage::Idle,
            phase: 0.0,
            sample_rate,
            phase_increment: FREQ / sample_rate,
            attack_increment: 1.0 / (adsr.attack * sample_rate),
            decay_increment: 1.0 / (adsr.decay * sample_rate),
            release_increment: 1.0 / (adsr.release * sample_rate),
            is_on,
            waveform_rx,
            adsr_rx,
        }
    }

    fn process(&mut self, data: &mut [f32]) {
        if let Ok(new_waveform) = self.waveform_rx.try_recv() {
            self.waveform = new_waveform;
        }
        if let Ok(new_adsr) = self.adsr_rx.try_recv() {
            self.adsr = new_adsr;
            self.attack_increment = 1.0 / (self.adsr.attack * self.sample_rate);
            self.decay_increment = 1.0 / (self.adsr.decay * self.sample_rate);
            self.release_increment =
                1.0 / (self.adsr.release * self.sample_rate);
        }
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
        for sample in data.iter_mut() {
            let envelope = self.envelope_stage.amplitude(self.adsr.sustain);
            *sample = match self.waveform {
                Waveform::Sine => (2.0 * PI * self.phase).sin() * envelope,
                Waveform::Square => {
                    if self.phase < 0.5 {
                        envelope
                    } else {
                        -envelope
                    }
                }
                Waveform::Saw => (1.0 - 2.0 * self.phase) * envelope,
                Waveform::Triangle => {
                    (4.0 * (self.phase - 0.5).abs() - 1.0) * envelope
                }
            };
            self.phase = (self.phase + self.phase_increment) % 1.0;
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
        let (adsr_tx, adsr_rx) = mpsc::channel::<Adsr>();
        let mut processor = AudioProcessor::new(
            config.sample_rate() as f32,
            is_on.clone(),
            waveform_rx,
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
            adsr_tx,
        }
    }
}

pub fn execute_command(command: Command, audio: &Audio) {
    match command {
        Command::PlayNote => audio.is_on.store(true, Ordering::Relaxed),
        Command::StopNote => audio.is_on.store(false, Ordering::Relaxed),
        Command::SetWaveform(waveform) => {
            audio.waveform_tx.send(waveform).unwrap()
        }
        Command::SetAdsr(adsr) => audio.adsr_tx.send(adsr).unwrap(),
        Command::None => (),
    }
}
