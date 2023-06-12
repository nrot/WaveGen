use std::{collections::HashMap, path::PathBuf};

use anyhow::anyhow;
use egui::{Ui, Vec2};
use log::{debug, warn};

use crate::{
    app::{
        waves::{BitValue, Wave},
        WaveType,
    },
    hseparator,
};

use super::WindowResult;

#[derive(PartialEq, Eq, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
enum InputType {
    CSV,
    VCD,
}

impl InputType {
    fn into_iter() -> [Self; 2] {
        [InputType::CSV, InputType::VCD]
    }
}

impl ToString for InputType {
    fn to_string(&self) -> String {
        match self {
            InputType::CSV => "CSV",
            InputType::VCD => "VCD",
        }
        .into()
    }
}

pub struct ImportData {
    input_tp_file: InputType,
    file_path: Option<PathBuf>,
    new_waves: Vec<Wave>,
    unknow_value: bool,
    high_impedance: bool,
}

impl Default for ImportData {
    fn default() -> Self {
        Self {
            input_tp_file: InputType::CSV,
            file_path: None,
            new_waves: Vec::new(),
            unknow_value: false,
            high_impedance: false,
        }
    }
}

impl ImportData {
    pub fn display(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> WindowResult {
        let mut state = WindowResult::Open;
        let mut open = true;
        let ms = _frame.info().window_info.monitor_size;
        egui::Window::new("Import data")
            .open(&mut open)
            .resizable(true)
            .resize(|r|{
                if let Some(ms) = ms{
                    r.max_size(ms)
                } else {
                    r.auto_sized()
                }
            })
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    egui::ComboBox::new("input_type_file", "Input type file").show_ui(ui, |ui| {
                        InputType::into_iter().into_iter().for_each(|v| {
                            ui.selectable_value(&mut self.input_tp_file, v, v.to_string());
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "File: {}",
                            self.file_path
                                .as_ref()
                                .unwrap_or(&PathBuf::new())
                                .as_path()
                                .to_string_lossy()
                        ));
                        if ui.button("Choose file").clicked() {
                            let folder = rfd::FileDialog::new().pick_file();
                            if let Some(f) = folder {
                                match self.import_vcd(&f) {
                                    Ok(nw) => self.new_waves = nw,
                                    Err(e) => {
                                        warn!("Error: {}", e);
                                        state = WindowResult::Error(e);
                                    }
                                }
                                self.file_path = Some(f);
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        match self.input_tp_file {
                            InputType::CSV => {
                                ui.label("Not implemented now");
                            }
                            InputType::VCD => {
                                self.params_vcd(ui);
                            }
                        };
                    });
                    self.display_new_values(ui);
                });
            });
        if !open {
            state = WindowResult::Close;
        }
        state
    }

    fn params_vcd(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Unknow Value replace to:");
                ui.checkbox(&mut self.unknow_value, "");
            });
            ui.horizontal(|ui| {
                ui.label("High Impedance replace to:");
                ui.checkbox(&mut self.high_impedance, "");
            });
        });
    }

    fn display_new_values(&mut self, ui: &mut Ui) {
        let link_group_id = ui.id().with("link_waves");
        // egui::CollapsingHeader::new("Imported plots").show(ui, |ui|{
            // egui::ScrollArea::new([true, false]).show(ui, |ui| {
                // ui.vertical(|ui| {
                    for w in &mut self.new_waves {
                        w.current_size.x = ui.available_width();
                        w.display(ui, link_group_id, &egui::InputState::default());
                        hseparator!(ui);
                    }
                // });
            // });
        // });
    }

    fn import_vcd(&mut self, p: &PathBuf) -> Result<Vec<Wave>, anyhow::Error> {
        let f = std::fs::File::open(p)?;
        let mut parser = vcd::Parser::new(f);
        let header = parser.parse_header().map_err(|e| {
            warn!("VCD parse header: {:#}", e);
            anyhow!(e)
        })?;
        let mut vars = HashMap::new();
        let mut waves: HashMap<vcd::IdCode, Wave> = HashMap::new();
        let mut stack = Vec::new();
        stack.push((String::new(), header.items));
        while let Some((scope, items)) = stack.pop() {
            for i in items {
                match i {
                    vcd::ScopeItem::Scope(s) => {
                        stack.push((s.identifier ,s.children));
                    }
                    vcd::ScopeItem::Var(v) => {
                        let Some(nt) = vcd_type(&v) else {
                            continue;
                        };
                        let mut w = Wave::new(format!("{}.{}", scope, v.reference.clone()), 16, egui::Vec2::ZERO);
                        w.set_type(nt);
                        waves.insert(v.code, w);
                        vars.insert(
                            v.code,
                            VcdVar {
                                scope: scope.clone(),
                                var: v,
                            },
                        );
                    }
                    vcd::ScopeItem::Comment(_) => {}
                }
            }
        }
        let mut current_time = 0usize;
        let (time_div, s) = header.timescale.unwrap_or((1, vcd::TimescaleUnit::S));
        let time_div = s.divisor() / time_div as u64;
        debug!("Time div: {}, {}", time_div, s);
        for item in parser {
            let item = item?;
            match item {
                vcd::Command::Comment(_)
                | vcd::Command::Date(_)
                | vcd::Command::Version(_)
                | vcd::Command::Timescale(_, _)
                | vcd::Command::ScopeDef(_, _)
                | vcd::Command::Upscope
                | vcd::Command::VarDef(_, _, _, _, _)
                | vcd::Command::Enddefinitions
                | vcd::Command::Begin(_)
                | vcd::Command::End(_) => {}
                vcd::Command::Timestamp(t) => {
                    for w in waves.values_mut() {
                        w.extend_by_last(t as usize / time_div as usize);
                    }
                    current_time = t as usize;
                }
                vcd::Command::ChangeScalar(id, v) => {
                    warn!("Not implemetned yet: {id}->{v}");
                }
                vcd::Command::ChangeVector(id, v) => {
                    if v.len() > BitValue::BITS {
                        warn!("To big value to implement: {id}");
                        continue;
                    }
                    if let Some(value) = waves.get(&id) {
                        let mut b = BitValue::new(value.reg_size());
                        let s = format!(
                            "0b{}",
                            v.iter()
                                .map(|v| {
                                    match v {
                                        vcd::Value::V0 => '0',
                                        vcd::Value::V1 => '1',
                                        vcd::Value::X => {
                                            if self.unknow_value {
                                                '1'
                                            } else {
                                                '0'
                                            }
                                        }
                                        vcd::Value::Z => {
                                            if self.high_impedance {
                                                '1'
                                            } else {
                                                '0'
                                            }
                                        }
                                    }
                                })
                                .collect::<String>()
                        );
                        b.parse_from(&s).map_err(|v| {
                            warn!("Error value: {}", s);
                            anyhow!("Error change vector: {}", v)
                        })?;
                        waves.get_mut(&id).unwrap().set_last_value(b);
                    }
                }
                vcd::Command::ChangeReal(id, v) => {
                    warn!("Not implemented Real value: {id}->{v:.2}");
                }
                vcd::Command::ChangeString(id, _) => {
                    warn!("String unsuported: {id}");
                }
                _ => warn!("Unknow vcd command"),
            }
        }
        Ok(waves.into_values().collect())
    }
}

struct VcdVar {
    scope: String,
    var: vcd::Var,
}

#[rustfmt::skip]
fn vcd_type(v: &vcd::Var)->Option<WaveType>{
    match v.var_type{
        vcd::VarType::Integer => {
            Some(WaveType::Reg(v.size as usize))
        },
        vcd::VarType::Real => {
            Some(WaveType::Reg(v.size as usize))
        },
        vcd::VarType::Reg => {
            Some(WaveType::Reg(v.size as usize))
        },
        vcd::VarType::Parameter => {warn!("Unsuported type: Parameter");None},
        vcd::VarType::Event   => {warn!("Unsuported type: Event");None},
        vcd::VarType::Supply0 => {warn!("Unsuported type: Supply0");None},
        vcd::VarType::Supply1 => {warn!("Unsuported type: Supply1");None},
        vcd::VarType::Time    => {warn!("Unsuported type: Time");None},
        vcd::VarType::Tri     => {warn!("Unsuported type: Tri");None},
        vcd::VarType::TriAnd  => {warn!("Unsuported type: TriAnd");None},
        vcd::VarType::TriOr   => {warn!("Unsuported type: TriOr");None},
        vcd::VarType::TriReg  => {warn!("Unsuported type: TriReg");None},
        vcd::VarType::Tri0    => {warn!("Unsuported type: Tri0");None},
        vcd::VarType::Tri1    => {warn!("Unsuported type: Tri1");None},
        vcd::VarType::WAnd    => {warn!("Unsuported type: WAnd");None},
        vcd::VarType::Wire    => {warn!("Unsuported type: Wire");None},
        vcd::VarType::WOr     => {warn!("Unsuported type: WOr");None},
        vcd::VarType::String =>  {warn!("Unsuported type: String");None}
        t => {
            warn!("Unsuported type: {}", t.to_string()); None
        }
    }
}
