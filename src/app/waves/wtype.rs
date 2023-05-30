use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub(super) struct Clock {
    pub period: usize,
    pub duty: usize,
    pub phase: usize,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            period: 2,
            duty: 1,
            phase: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub(super) enum WaveType {
    Clock(Clock),
    Wire,
    Reg(usize),
}
