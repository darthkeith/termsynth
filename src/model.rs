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

impl Adsr {
    fn new() -> Self {
        Self {
            attack: ATTACK,
            decay: DECAY,
            sustain: SUSTAIN,
            release: RELEASE,
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
