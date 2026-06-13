const DEFAULT_CUTOFF: f32 = 8000.0;
const DEFAULT_ATTACK: f32 = 0.01;
const DEFAULT_DECAY: f32 = 0.1;
const DEFAULT_SUSTAIN: f32 = 0.7;
const DEFAULT_RELEASE: f32 = 0.3;
const CUTOFF_MIN: f32 = 20.0;
const CUTOFF_MAX: f32 = 20000.0;
const ENV_TIME_MIN: f32 = 0.0001;
const ENV_TIME_MAX: f32 = 10.0;

#[derive(Clone, Copy)]
pub enum Waveform {
    Sine,
    Square,
    Saw,
    Triangle,
}

#[derive(Clone, Copy)]
pub struct Adsr {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

#[derive(PartialEq)]
pub enum Param {
    Cutoff,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Model {
    pub is_on: bool,
    pub waveform: Waveform,
    pub cutoff: f32,
    pub adsr: Adsr,
    pub selected: Param,
}

impl Waveform {
    pub fn name(&self) -> &'static str {
        match self {
            Waveform::Sine => "Sine",
            Waveform::Square => "Square",
            Waveform::Saw => "Saw",
            Waveform::Triangle => "Triangle",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Waveform::Sine => Waveform::Square,
            Waveform::Square => Waveform::Saw,
            Waveform::Saw => Waveform::Triangle,
            Waveform::Triangle => Waveform::Sine,
        }
    }
}

fn adjust(val: f32, delta: f32) -> f32 {
    (val + delta).clamp(0.0, 1.0)
}

fn exp_adjust(val: f32, delta: f32, min: f32, max: f32) -> f32 {
    let step = delta.abs();
    let log_val = (val.log10() / step).round() * step;
    10f32.powf(log_val + delta).clamp(min, max)
}

fn exp_adjust_cutoff(val: f32, delta: f32) -> f32 {
    exp_adjust(val, delta, CUTOFF_MIN, CUTOFF_MAX)
}

fn exp_adjust_env_time(val: f32, delta: f32) -> f32 {
    exp_adjust(val, delta, ENV_TIME_MIN, ENV_TIME_MAX)
}

impl Adsr {
    pub fn new() -> Self {
        Self {
            attack: DEFAULT_ATTACK,
            decay: DEFAULT_DECAY,
            sustain: DEFAULT_SUSTAIN,
            release: DEFAULT_RELEASE,
        }
    }
}

impl Model {
    pub fn new() -> Self {
        Self {
            is_on: false,
            waveform: Waveform::Sine,
            cutoff: DEFAULT_CUTOFF,
            adsr: Adsr::new(),
            selected: Param::Attack,
        }
    }

    pub fn increment(&mut self) {
        match self.selected {
            Param::Cutoff => self.cutoff = exp_adjust_cutoff(self.cutoff, 0.01),
            Param::Attack => {
                self.adsr.attack = exp_adjust_env_time(self.adsr.attack, 0.1)
            }
            Param::Decay => {
                self.adsr.decay = exp_adjust_env_time(self.adsr.decay, 0.1)
            }
            Param::Sustain => {
                self.adsr.sustain = adjust(self.adsr.sustain, 0.01)
            }
            Param::Release => {
                self.adsr.release = exp_adjust_env_time(self.adsr.release, 0.1)
            }
        }
    }

    pub fn decrement(&mut self) {
        match self.selected {
            Param::Cutoff => {
                self.cutoff = exp_adjust_cutoff(self.cutoff, -0.01)
            }
            Param::Attack => {
                self.adsr.attack = exp_adjust_env_time(self.adsr.attack, -0.1)
            }
            Param::Decay => {
                self.adsr.decay = exp_adjust_env_time(self.adsr.decay, -0.1)
            }
            Param::Sustain => {
                self.adsr.sustain = adjust(self.adsr.sustain, -0.01)
            }
            Param::Release => {
                self.adsr.release = exp_adjust_env_time(self.adsr.release, -0.1)
            }
        }
    }
}
