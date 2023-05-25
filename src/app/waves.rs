use bitvec::prelude::*;
use egui::{
    plot::{Line, PlotPoint, PlotPoints, Polygon, AxisBools},
    InputState, Pos2, Ui,
};
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Clock {
    period: usize,
    duty: usize,
    phase: usize,
}

#[derive(Serialize, Deserialize)]
enum WaveType {
    Clock(Clock),
    Wire,
    Reg(usize),
}

#[derive(Serialize, Deserialize)]
enum WaveDisplay {
    Binary,
    Hex,
    SignedDecimal,
    UnsignedDecimal,
    Analog(Box<WaveDisplay>), // ?
}

#[derive(Serialize, Deserialize)]
struct StateEdit {
    index: usize,
    value: BitVec,
    pos: Pos2,
}

#[derive(Serialize, Deserialize)]
enum WaveState {
    Show,
    Edit(StateEdit),
}

#[derive(Serialize, Deserialize)]
pub struct Wave {
    state: WaveState,
    tp: WaveType,
    display: WaveDisplay,
    name: String,
    data: Vec<BitVec>,
    max_value: f64,
    min_value: f64,
    deleted: bool,
}

impl Wave {
    pub fn new<T: Into<String>>(name: T, size: usize) -> Self {
        let mut v = Vec::with_capacity(size);
        v.resize(size, bitvec![0; 1]);
        debug!("New data size: {}", v.len());
        Self {
            state: WaveState::Show,
            tp: WaveType::Wire,
            display: WaveDisplay::Binary,
            name: name.into(),
            data: v,
            max_value: 0.0,
            min_value: 0.0,
            deleted: false,
        }
    }

    pub fn display(&mut self, ui: &mut Ui, link_group_id: egui::Id, user_input: &InputState) {
        ui.horizontal(|ui| {
            let name = self.name.clone();
            ui.vertical(|ui| {
                ui.text_edit_singleline(&mut self.name)
                    .context_menu(|ui| self.name_menu(ui));
                if ui.button("Delete").clicked() {
                    self.deleted = true;
                }
            });
            let mut diff = 0.0;
            if let Some(ts) = ui.ctx().style().text_styles.get(&egui::TextStyle::Body){
                diff = ts.size as f64;
            };

            egui::plot::Plot::new(name)
                .link_axis(link_group_id, true, false)
                .link_cursor(link_group_id, true, true)
                .allow_drag(AxisBools::new(true, false))
                .allow_scroll(false)
                .allow_zoom(AxisBools::new(true, false))
                .auto_bounds_y()
                .show(ui, |plot_ui| {
                    let mut max = f64::NEG_INFINITY;
                    let mut min = f64::INFINITY;
                    let plot: PlotPoints = self
                        .data
                        .iter()
                        .enumerate()
                        .flat_map(|(i, v)| {
                            let t = v[0] as u8 as f64;
                            if t > max {
                                max = t;
                            }
                            if t < min {
                                min = t;
                            }
                            [[i as f64, t], [(i + 1) as f64, t]]
                        })
                        .collect();
                    // max += (max - min).abs() * 0.1;
                    // min -= (max - min).abs() * 0.1;
                    let transfom = plot_ui.transform();
                    diff *= transfom.bounds().height() / transfom.frame().height() as f64;
                    diff *= 1.05;
                    max += diff;
                    min -= diff;

                    
 
                    let line = Line::new(plot);
                    plot_ui.line(line);
                    if let Some(p) = plot_ui.pointer_coordinate() {
                        if p.x >= 0.0 && p.x <= self.data.len() as f64 {
                            let polygon = Polygon::new(PlotPoints::Owned(vec![
                                //TODO: переделать в выделение максимума/минимума
                                PlotPoint::new(p.x.floor(), max),
                                PlotPoint::new(p.x.ceil(), max),
                                PlotPoint::new(p.x.ceil(), min),
                                PlotPoint::new(p.x.floor(), min),
                            ]));
                            plot_ui.polygon(polygon.name(""));
                            if plot_ui.plot_secondary_clicked() {
                                debug!(
                                    "Clicked by plot: {}; Data size: {}",
                                    p.x.floor() as usize,
                                    self.data.len()
                                );
                                if let Some(v) = self.data.get(p.x.floor() as usize) {
                                    debug!("Mouse position: {:?}", user_input.pointer.hover_pos());
                                    self.state = WaveState::Edit(StateEdit {
                                        index: p.x.floor() as usize,
                                        value: v.clone(),
                                        pos: user_input
                                            .pointer
                                            .hover_pos()
                                            .unwrap_or(Pos2 { x: 0.0, y: 0.0 }),
                                    });
                                }
                            }
                        }
                    }
                });
            if let WaveState::Edit(edit) = &mut self.state {
                let mut v = edit.value[0];
                let mut open = true;
                let mut open_2 = true;
                egui::Window::new(format!("Edit: {}", edit.index))
                    .title_bar(true)
                    .open(&mut open)
                    .collapsible(false)
                    .default_pos(edit.pos)
                    .show(ui.ctx(), |ui| {
                        if ui.checkbox(&mut v, "value").changed() {
                            self.data[edit.index].set(0, v);
                            open_2 = false;
                        };
                    });
                if !open || !open_2 {
                    self.state = WaveState::Show;
                }
            }
        });
    }

    fn name_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Change Type", |ui| {
            ui.button("Wire");
        });
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn set_len(&mut self, len: usize){
        if self.data.len() < len{
            self.data.resize(len, bitvec!(0; 1));
        } else {
            self.data.truncate(len);
        }
    }
}
