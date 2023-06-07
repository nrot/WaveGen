use std::{collections::HashMap, path::PathBuf};

use anyhow::anyhow;
use egui::Ui;
use log::warn;
use vcd::Parser;

use crate::app::{waves::{Wave, BitValue}, WaveType};

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
}

impl Default for ImportData {
    fn default() -> Self {
        Self {
            input_tp_file: InputType::CSV,
            file_path: None,
            new_waves: Vec::new(),
        }
    }
}

impl ImportData {
    pub fn display(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> WindowResult {
        let mut state = WindowResult::Open;
        let mut open = true;
        egui::Window::new("Import data")
            .open(&mut open)
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
                                self.import_vcd(&f);
                                self.file_path = Some(f);
                            }
                        }
                    });
                });
            });
        if !open {
            state = WindowResult::Close;
        }
        state
    }

    fn import_vcd(&mut self, p: &PathBuf) -> Result<(), anyhow::Error> {
        let f = std::fs::File::open(p)?;
        let mut parser = vcd::Parser::new(f);
        let header = parser.parse_header()?;
        let mut scope = String::new();
        let mut vars = HashMap::new();
        let mut waves: HashMap<vcd::IdCode, Wave> = HashMap::new();
        let mut stack = Vec::new();
        stack.push(header.items);
        while let Some(items) = stack.pop() {
            for i in items {
                match i {
                    vcd::ScopeItem::Scope(s) => {
                        stack.push(s.children);
                    }
                    vcd::ScopeItem::Var(v) => {
                        let Some(nt) = vcd_type(&v) else {
                            continue;
                        };
                        let mut w =Wave::new(v.reference.clone(), 16, egui::Vec2::ZERO); 
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
                }
            }
        }
        let mut current_time = 0usize;
        let (time_div, _) = header.timescale.unwrap_or((1, vcd::TimescaleUnit::S));
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
                        w.extend_by_last((t / time_div as u64) as usize);
                    }
                    current_time = t as usize;
                }
                vcd::Command::ChangeScalar(id, v) => {
                    warn!("Not implemetned yet: {id}->{v}");
                },
                vcd::Command::ChangeVector(id, v) => {
                    if v.len() > BitValue::BITS{
                        warn!("To big value to implement: {id}");
                        continue;
                    }
                },
                vcd::Command::ChangeReal(id, v) => todo!(),
                vcd::Command::ChangeString(id, _) => {
                    warn!("String unsuported: {id}");
                },
                _ => warn!("Unknow vcd command"),
            }
        }

        Ok(())
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
