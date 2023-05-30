mod state_edit;
mod type_change;
mod value;
mod wtype;

use std::{io::Write, path::PathBuf};

use egui::{
    plot::{AxisBools, Line, PlotPoint, PlotPoints, Polygon},
    InputState, Pos2, Ui, Vec2,
};
use log::debug;
use serde::{Deserialize, Serialize};

use crate::hseparator;

use self::{state_edit::StateEdit, type_change::TypeChange, value::BitValue, wtype::WaveType};
use super::windows::WindowResult;

#[derive(Serialize, Deserialize, Clone, Copy)]
enum WaveSign {
    Unsigned,
    Signed,
}

impl WaveSign {
    pub fn signed(&self) -> bool {
        match self {
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

impl WaveDisplay {
    pub fn signed(&self) -> bool {
        match self {
            WaveDisplay::Binary => false,
            WaveDisplay::Hex => false,
            WaveDisplay::Decimal(s) => s.signed(),
            WaveDisplay::Analog(s) => s.signed(),
        }
    }
}

#[derive(Serialize, Deserialize)]
enum WaveState {
    Show,
    Edit(StateEdit),
    TypeChange(TypeChange),
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
    selected_data: Vec<usize>,
    max_value: f64,
    min_value: f64,
    deleted: bool,
    pub current_size: Vec2,
}

impl Wave {
    pub fn new<T: Into<String>>(name: T, size: usize, ui_size: Vec2) -> Self {
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
            selected_data: Vec::new(),
            max_value: 0.0,
            min_value: 0.0,
            deleted: false,
            current_size: ui_size,
        }
    }

    pub fn display(&mut self, ui: &mut Ui, link_group_id: egui::Id, user_input: &InputState) {
        ui.horizontal(|ui| {
            ui.allocate_ui(self.current_size, |ui| {
                let name = self.name.clone();
                ui.vertical(|ui| {
                    ui.text_edit_singleline(&mut self.name)
                        .context_menu(|ui| self.name_menu(ui));
                });
                let mut diff = 0.0;
                if let Some(ts) = ui.ctx().style().text_styles.get(&egui::TextStyle::Body) {
                    diff = ts.size as f64;
                };

                let plot_response = egui::plot::Plot::new(name)
                    .link_axis(link_group_id, true, false)
                    .link_cursor(link_group_id, true, true)
                    .allow_drag(AxisBools::new(true, false))
                    .allow_scroll(false)
                    .allow_zoom(AxisBools::new(true, false))
                    .auto_bounds_y()
                    .show(ui, |plot_ui| {
                        let mut max = self.max_value;
                        let mut min = self.min_value;
                        let idata = self.data.iter().enumerate();
                        let plot: PlotPoints = match self.display {
                            WaveDisplay::Binary | WaveDisplay::Hex => idata
                                .flat_map(|(i, v)| {
                                    let t = v.to_f64(false);
                                    [[i as f64, t], [(i + 1) as f64, t]]
                                })
                                .collect(),
                            WaveDisplay::Decimal(s) => idata
                                .flat_map(|(i, v)| {
                                    let t = v.to_f64(s.signed());
                                    [[i as f64, t], [(i + 1) as f64, t]]
                                })
                                .collect(),
                            WaveDisplay::Analog(s) => idata
                                .map(|(i, v)| {
                                    let t = v.to_f64(s.signed());
                                    [i as f64, t]
                                })
                                .collect(),
                        };
                        //  self
                        //     .data
                        //     .iter()
                        //     .enumerate()
                        //     .flat_map(|(i, v)| {
                        //         match self.display{
                        //             WaveDisplay::Binary | WaveDisplay::Hex  => {
                        //                 let t = v.to_f64(false);
                        //                 [[i as f64, t], [(i + 1) as f64, t]]
                        //             },
                        //             WaveDisplay::Decimal(s) => {
                        //                 let t = v.to_f64(s.signed());
                        //                 [[i as f64, t], [(i + 1) as f64, t]]
                        //             },
                        //             WaveDisplay::Analog(s) => {
                        //                 let t = v.to_f64(s.signed());
                        //                 [[i as f64, t],]
                        //             },
                        //         }

                        //     })
                        //     .collect();
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
                                ]))
                                .color(egui::Color32::from_rgba_unmultiplied(30, 30, 150, 125));
                                plot_ui.polygon(polygon.name(""));
                                if plot_ui.plot_secondary_clicked() {
                                    debug!(
                                        "Clicked by plot: {}; Data size: {}",
                                        p.x.floor() as usize,
                                        self.data.len()
                                    );
                                    if let Some(v) = self.data.get(p.x.floor() as usize) {
                                        debug!(
                                            "Mouse position: {:?}",
                                            user_input.pointer.hover_pos()
                                        );
                                        self.state = WaveState::Edit(StateEdit {
                                            index: p.x.floor() as usize,
                                            init_value: v.clone(),
                                            pos: user_input
                                                .pointer
                                                .hover_pos()
                                                .unwrap_or(Pos2 { x: 0.0, y: 0.0 }),
                                            tp: self.tp,
                                            display: self.display,
                                            current_value: None,
                                            error: None,
                                        });
                                    }
                                }
                            }
                        }
                        if let WaveState::Edit(e) = &self.state {
                            let polygon = Polygon::new(PlotPoints::Owned(vec![
                                //TODO: переделать в выделение максимума/минимума
                                PlotPoint::new(e.index as f64, max),
                                PlotPoint::new((e.index + 1) as f64, max),
                                PlotPoint::new((e.index + 1) as f64, min),
                                PlotPoint::new(e.index as f64, min),
                            ]))
                            .color(egui::Color32::from_rgba_unmultiplied(150, 30, 30, 125));
                            plot_ui.polygon(polygon.name(""));
                        }
                    });
                if plot_response.response.double_clicked() {
                    self.refresh_min_max();
                }
                self.display_window_edit(ui);
                self.display_type_change(ui);
            });
        });
    }

    fn display_window_edit(&mut self, ui: &mut Ui) {
        if let WaveState::Edit(edit) = &mut self.state {
            match edit.window_edit(ui) {
                WindowResult::Open => {}
                WindowResult::Save => {
                    self.data[edit.index] = edit.init_value.clone();
                    let vf = self.data[edit.index].to_f64(self.display.signed());
                    if vf > self.max_value {
                        self.max_value = vf;
                    }
                    if vf < self.min_value {
                        self.min_value = vf;
                    }
                    self.state = WaveState::Show;
                }
                WindowResult::Cancel | WindowResult::Close => {
                    self.state = WaveState::Show;
                }
            };
        }
    }

    fn recalculate_clock(&mut self) {
        if let WaveType::Clock(c) = self.tp {
            self.data.iter_mut().enumerate().for_each(|(i, v)| {
                let i = i + c.phase;
                v.set_size(1).unwrap();
                v.set_zero();
                v.set_bool((i % c.period) < c.duty);
            });
        }
    }

    fn display_type_change(&mut self, ui: &mut Ui) {
        if let WaveState::TypeChange(params) = &mut self.state {
            match params.display(ui) {
                WindowResult::Open => {}
                WindowResult::Save => {
                    match params.new_tp {
                        WaveType::Clock(c) => {
                            self.tp = WaveType::Clock(c);
                            self.recalculate_clock();
                            self.max_value = 1.0;
                            self.min_value = 0.0;
                            self.display = WaveDisplay::Binary;
                        }
                        WaveType::Wire => {}
                        WaveType::Reg(r) => {
                            self.data.iter_mut().for_each(|v| {
                                v.set_size(r).unwrap();
                            });
                            self.tp = WaveType::Reg(r);
                            self.display = WaveDisplay::Hex;
                        }
                    }
                    self.state = WaveState::Show;
                }
                WindowResult::Cancel | WindowResult::Close => {
                    self.state = WaveState::Show;
                }
            }
        }
    }

    fn name_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Change Type", |ui| {
            if ui.button("Wire").clicked() {
                self.data.iter_mut().for_each(|v| {
                    v.set_size(1).unwrap();
                    self.display = WaveDisplay::Binary;
                    self.tp = WaveType::Wire;
                });
                return;
            };
            if ui.button("Clock").clicked() {
                self.state = WaveState::TypeChange(TypeChange {
                    current_tp: self.tp,
                    new_tp: if let WaveType::Clock(_) = self.tp {
                        self.tp
                    } else {
                        WaveType::Clock(wtype::Clock::new())
                    },
                    max_size: self.data.len(),
                });
                return;
            }
            if ui.button("Reg").clicked() {
                self.state = WaveState::TypeChange(TypeChange {
                    current_tp: self.tp,
                    new_tp: WaveType::Reg(1),
                    max_size: self.data.len(),
                });
            }
        });
        ui.menu_button("Change display", |ui| {
            if ui.button("Binary").clicked() {
                self.display = WaveDisplay::Binary;
                return;
            }
            if ui.button("Hex").clicked() {
                self.display = WaveDisplay::Hex;
                return;
            }
            ui.menu_button("Decimal", |ui| {
                if ui.button("Unsigned").clicked() {
                    self.display = WaveDisplay::Decimal(WaveSign::Unsigned);
                }
                if ui.button("Signed").clicked() {
                    self.display = WaveDisplay::Decimal(WaveSign::Signed);
                }
            });
            ui.menu_button("Analog", |ui| {
                if ui.button("Unsigned").clicked() {
                    self.display = WaveDisplay::Analog(WaveSign::Unsigned);
                }
                if ui.button("Signed").clicked() {
                    self.display = WaveDisplay::Analog(WaveSign::Signed);
                }
            });
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
        self.recalculate_clock();
        self.refresh_min_max();
    }

    fn refresh_min_max(&mut self) {
        self.max_value = f64::NEG_INFINITY;
        self.min_value = f64::INFINITY;
        self.data.iter().for_each(|v| {
            let fv = v.to_f64(self.display.signed());
            if fv > self.max_value {
                self.max_value = fv;
            }
            if fv < self.min_value {
                self.min_value = fv;
            }
        });
    }

    pub fn export_type(&self) -> String {
        match self.tp {
            WaveType::Clock(_) => "wire".into(),
            WaveType::Wire => "wire".into(),
            WaveType::Reg(s) => format!("reg [{}:0]", s),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone().replace(' ', "_")
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[allow(unused)]
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
