use egui::Ui;
use serde::{Deserialize, Serialize};
use log::error;

use crate::{app::windows::WindowResult, hseparator};

use super::{wtype::{Clock, WaveType}, value::BitValue};

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct TypeChange {
    pub current_tp: WaveType,
    pub new_tp: WaveType,
    pub max_size: usize,
}

impl TypeChange {
    pub(crate) fn display(&mut self, ui: &mut Ui) -> WindowResult {
        let mut open = true;
        let mut state = WindowResult::Open;
        egui::Window::new("Edit type")
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| match self.new_tp {
                    WaveType::Clock(_) => self.display_clock(ui),
                    WaveType::Wire => {
                        error!("Wire don`t have ");
                        state = WindowResult::Cancel
                    }
                    WaveType::Reg(_) => self.display_reg(ui),
                });
                hseparator!(ui);
                if ui.button("Save").clicked(){
                    state = WindowResult::Save;
                }
            });
        if !open {
            state = WindowResult::Cancel;
        }
        state
    }
    fn display_clock(&mut self, ui: &mut Ui) {
        if let WaveType::Clock(c) = &mut self.new_tp {
            ui.horizontal(|ui| {
                ui.label("Period");
                ui.add(egui::DragValue::new(&mut c.period).clamp_range(0..=self.max_size));
            });
            ui.horizontal(|ui| {
                ui.label("Period");
                ui.add(egui::DragValue::new(&mut c.duty).clamp_range(0..=c.period));
            });
            ui.horizontal(|ui| {
                ui.label("Phase");
                ui.add(egui::DragValue::new(&mut c.phase).clamp_range(0..=c.period));
            });
        };
    }

    fn display_reg(&mut self, ui: &mut Ui){
        if let WaveType::Reg(r)=&mut self.new_tp{
            ui.horizontal(|ui|{
                ui.label("Reg size");
                ui.add(egui::DragValue::new(r).clamp_range(1..=BitValue::BITS));
                ui.label(egui::RichText::new(format!("value must be from 1 to {}", BitValue::BITS)).small());
            });
        };
    }
}
