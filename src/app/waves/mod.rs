mod state_edit;
mod value;
mod wtype;

use std::{io::Write, path::PathBuf};

use egui::{
    plot::{AxisBools, Line, PlotPoint, PlotPoints, Polygon},
    InputState, Pos2, Ui,
};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::hseparator;

use self::{state_edit::StateEdit, value::BitValue, wtype::WaveType};


#[derive(Serialize, Deserialize, Clone, Copy)]
enum WaveSign{
    Unsigned,
    Signed
}

impl WaveSign{
    pub fn sgined(&self)->bool{
        match self{
            WaveSign::Unsigned => false,
            WaveSign::Signed => true,
        }
    }
}


#[derive(Serialize, Deserialize, Clone, Copy)]
enum WaveDisplay {
    Binary,
    Hex,
    Decimal(WaveSign),
    Analog(WaveSign), 
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
    data: Vec<BitValue>,
    plot_data: Vec<f64>,
    viwed_data: Vec<String>,
    max_value: f64,
    min_value: f64,
    deleted: bool,
}

impl Wave {
    pub fn new<T: Into<String>>(name: T, size: usize) -> Self {
        let mut data = Vec::with_capacity(size);
        data.resize(size, BitValue::new(1));
        let mut plot_data = Vec::with_capacity(size);
        plot_data.resize(size, 0.0);
        let mut viwed_data = Vec::with_capacity(size);
        viwed_data.resize(size, "0".into());
        debug!("New data size: {}", data.len());
        Self {
            state: WaveState::Show,
            tp: WaveType::Wire,
            display: WaveDisplay::Binary,
            name: name.into(),
            data,
            plot_data,
            viwed_data,
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
            });
            let mut diff = 0.0;
            if let Some(ts) = ui.ctx().style().text_styles.get(&egui::TextStyle::Body) {
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
                                        init_value: v.clone(),
                                        pos: user_input
                                            .pointer
                                            .hover_pos()
                                            .unwrap_or(Pos2 { x: 0.0, y: 0.0 }),
                                        tp: self.tp,
                                        display: self.display,
                                        current_value: None
                                    });
                                }
                            }
                        }
                    }
                });
            if let WaveState::Edit(edit) = &mut self.state {
                match edit.window_edit(ui) {
                    super::windows::WindowResult::Open => {}
                    super::windows::WindowResult::Save => {
                        self.data[edit.index] = edit.init_value.clone();
                        self.state = WaveState::Show;
                    }
                    super::windows::WindowResult::Cancel | super::windows::WindowResult::Close => {
                        self.state = WaveState::Show;
                    }
                };
            }
        });
    }

    fn name_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Change Type", |ui| {
            ui.button("Wire");
        });
        hseparator!(ui);
        if ui.button("Delete").clicked() {
            self.deleted = true;
        }
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn set_len(&mut self, len: usize) {
        if self.data.len() < len {
            self.data.resize(len, BitValue::new(1));
        } else {
            self.data.truncate(len);
        }
    }

    pub fn export_type(&self) -> String {
        match self.tp {
            WaveType::Clock(_) => "wire".into(),
            WaveType::Wire => "wire".into(),
            WaveType::Reg(s) => format!("reg [{}:0]", s),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    fn reg_size(&self) -> usize {
        match self.tp {
            WaveType::Clock(_) => 1,
            WaveType::Wire => 1,
            WaveType::Reg(s) => s,
        }
    }

    pub fn generate_memb(&self, path: PathBuf) -> Result<(), std::io::Error> {
        let mut fl = std::fs::File::create(path)?;
        for v in &self.data {
            fl.write_all(v.to_bin().as_bytes()).unwrap();
            fl.write_all(b"\n").unwrap();
        }
        fl.sync_data()?;
        Ok(())
    }
}
