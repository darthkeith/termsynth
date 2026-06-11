use crate::audio::{ATTACK, DECAY, RELEASE, SUSTAIN};

pub struct Adsr {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

#[derive(PartialEq)]
pub enum Param {
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Model {
    pub is_on: bool,
    pub adsr: Adsr,
    pub selected: Param,
}

fn adjust(val: f32, delta: f32) -> f32 {
    (val + delta).clamp(0.0, 1.0)
}

fn exp_adjust(val: f32, delta: f32) -> f32 {
    let step = delta.abs();
    let log_val = (val.log10() / step).round() * step;
    10f32.powf(log_val + delta).clamp(0.0001, 10.0)
}

impl Adsr {
    fn new() -> Self {
        Self {
            attack: ATTACK,
            decay: DECAY,
            sustain: SUSTAIN,
            release: RELEASE,
        }
    }

    pub fn increment(&mut self, selected: &Param) {
        match selected {
            Param::Attack => self.attack = exp_adjust(self.attack, 0.1),
            Param::Decay => self.decay = exp_adjust(self.decay, 0.1),
            Param::Sustain => self.sustain = adjust(self.sustain, 0.01),
            Param::Release => self.release = exp_adjust(self.release, 0.1),
        }
    }

    pub fn decrement(&mut self, selected: &Param) {
        match selected {
            Param::Attack => self.attack = exp_adjust(self.attack, -0.1),
            Param::Decay => self.decay = exp_adjust(self.decay, -0.1),
            Param::Sustain => self.sustain = adjust(self.sustain, -0.01),
            Param::Release => self.release = exp_adjust(self.release, -0.1),
        }
    }
}

impl Model {
    pub fn new() -> Self {
        Self {
            is_on: false,
            adsr: Adsr::new(),
            selected: Param::Attack,
        }
    }
}
