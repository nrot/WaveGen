use annotate_snippets::display_list::DisplayList;
use egui::{Pos2, Ui};
use serde::{Deserialize, Serialize};

use crate::app::windows::WindowResult;

use super::{wtype::WaveType, value::BitValue, WaveDisplay};

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct StateEdit {
    pub index: usize,
    pub init_value: BitValue,
    pub pos: Pos2,
    pub tp: WaveType,
    pub display: WaveDisplay,
    pub current_value: Option<String>
}

impl StateEdit {
    pub(super) fn window_edit(&mut self, ui: &mut Ui) -> WindowResult {
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
                        let mut v = self.init_value.bool();
                        if ui.checkbox(&mut v, "value").changed() {
                            self.init_value.neg_bool();
                            state = WindowResult::Save;
                        };
                    },
                    WaveType::Reg(_) => {
                        if let Some(v) = &mut self.current_value{
                            if ui.text_edit_singleline(v).changed(){
                                if let Err(e) = self.init_value.parse_from(v){
                                    ui.label(DisplayList::from(e).to_string());
                                };
                            }
                        } else {
                            self.current_value = Some(match self.display{
                                WaveDisplay::Binary => self.init_value.to_bin(),
                                WaveDisplay::Hex => self.init_value.to_hex(),
                                WaveDisplay::Decimal(s) => self.init_value.to_dec(s.sgined()),
                                WaveDisplay::Analog(s) => self.init_value.to_dec(s.sgined()),
                            }   );
                        }
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

    #[test]
    fn test_u64_neg(){
        let mut a = 1u64;
        a = !a & 0b1;
        println!("{}", a);
        a = !a & 0b1;
        println!("{}", a);  
    }
}