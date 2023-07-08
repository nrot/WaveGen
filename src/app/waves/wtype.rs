use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Clock {
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
pub enum WaveType {
    Clock(Clock),
    Wire,
    Reg(usize),
}


impl Display for WaveType{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WaveType::Clock(c) => write!(f, "Clock: period->{}, duty->{}, phase->{}", c.period, c.duty, c.phase),
            WaveType::Wire => write!(f, "Wire"),
            WaveType::Reg(r) => write!(f, "Register: {}", r),
        }
    }
}