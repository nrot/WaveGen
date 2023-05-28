use bitvec::vec::BitVec;
use egui::{Pos2, Ui};
use serde::{Deserialize, Serialize};

use crate::app::windows::WindowResult;

use super::wtype::WaveType;

#[derive(Serialize, Deserialize)]
pub(super) struct StateEdit {
    pub index: usize,
    pub value: BitVec,
    pub pos: Pos2,
    pub tp: WaveType,
}

impl StateEdit {
    pub(super) fn window_edit(&mut self, ui: &mut Ui) -> WindowResult {
        let mut v = self.value[0];
        let mut open = true;
        let mut state = WindowResult::Open;
        egui::Window::new(format!("Edit: {}", self.index))
            .title_bar(true)
            .open(&mut open)
            .collapsible(false)
            .default_pos(self.pos)
            .show(ui.ctx(), |ui| {
                match self.tp{
                    WaveType::Clock(_) => {
                        ui.vertical(|ui|{
                            ui.label("Clock can`t be modified. Change clock by type changing");
                            if ui.button("Ok").clicked(){
                                state = WindowResult::Cancel;
                            }
                        });
                    },
                    WaveType::Wire => {
                        if ui.checkbox(&mut v, "value").changed() {
                            state = WindowResult::Save;
                        };
                    },
                    WaveType::Reg(s) => {

                    },
                }
            });
        if !open {
            state = WindowResult::Cancel;
        }
        state
    }
}


#[cfg(test)]
mod test{
    #[test]
    fn test_f64_max(){
        println!("f64 max: {:e}", f64::MAX);
    }
}