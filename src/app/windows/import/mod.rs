use std::{collections::HashMap, path::PathBuf};

use egui::Ui;
use vcd::Parser;

use crate::app::waves::Wave;

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
        let mut scope = vcd::Scope::default();
        let mut vars = HashMap::new();
        let mut stack = Vec::new();
        stack.push(header.items);
        while let Some(items) = stack.pop() {
            for i in items {
                match i {
                    vcd::ScopeItem::Scope(s) => {
                        stack.push(s.children);
                    }
                    vcd::ScopeItem::Var(v) => {
                        vars.insert(v.code, v);
                    }
                }
            }
        }
        

        Ok(())
    }
}

struct VcdVar {
    scope: vcd::Scope,
    var: vcd::Var,
}
