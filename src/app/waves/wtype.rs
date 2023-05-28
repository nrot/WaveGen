use egui::Ui;
use serde::{Serialize, Deserialize};
use tracing_subscriber::field::display;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub(super) struct Clock {
    pub period: usize,
    pub duty: usize,
    pub phase: usize,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub(super) enum WaveType {
    Clock(Clock),
    Wire,
    Reg(usize),
}