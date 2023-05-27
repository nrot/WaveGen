use std::{
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use handlebars::to_json;
use log::{debug, error};

use crate::{app::waves::Wave, hseparator, App};

use super::WindowResult;

const TEMPLATE_NAME: &str = "test.sv";

#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ProjectExport {
    generate_sv: bool,
    generate_mem: bool,
    export_folder: PathBuf,
}

#[derive(serde::Serialize)]
struct ExportVariable {
    tp: String,
    name: String,
    name_data: String,
    index: String,
    name_file: String,
    memory_size: usize,
}

#[derive(serde::Serialize)]
struct ExportData {
    variables: Vec<ExportVariable>,
    end_time: usize,
}

impl ProjectExport {
    pub fn display(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> WindowResult {
        let mut state = WindowResult::Open;
        let mut open = true;
        egui::Window::new("Export").open(&mut open).show(ctx, |ui| {
            hseparator!(ui);
            if ui.radio(self.generate_sv, "Generate sv file").changed() {
                self.generate_sv = !self.generate_sv;
            };
            if ui.radio(self.generate_mem, "Generate mem file").changed() {
                self.generate_mem = !self.generate_mem;
            };
            ui.label(format!(
                "File: {}",
                self.export_folder.as_path().to_string_lossy()
            ));
            if ui.button("Choose folder").clicked() {
                let folder = rfd::FileDialog::new()
                    .set_directory(self.export_folder.as_path().to_string_lossy().as_ref())
                    .pick_folder();
                if let Some(f) = folder {
                    self.export_folder = f;
                }
            }
            if ui.button("Generate").clicked() {
                state = WindowResult::Save;
            }
        });
        if !open {
            state = WindowResult::Close;
        }
        state
    }
    pub fn generate_data(&mut self, waves: &Vec<Wave>) {
        let Some(w) = waves.first() else {
            error!("Nothing to generate add one signal");
            return;
        };
        let mut hand = handlebars::Handlebars::new();
        hand.register_template_file(TEMPLATE_NAME, "templates/test.hbs")
            .unwrap();
        let mut fout = std::fs::File::create(self.export_folder.join("test.sv")).unwrap();

        let mut data = ExportData {
            end_time: w.len() * 2,
            variables: Vec::with_capacity(waves.len()),
        };

        for wave in waves {
            data.variables.push(ExportVariable {
                tp: wave.export_type(),
                name: wave.name(),
                name_data: wave.name() + "_data",
                index: wave.name() + "_index",
                name_file: wave.name() + "_file.memb",
                memory_size: wave.len(),
            })
        }

        hand.render_to_write(TEMPLATE_NAME, &to_json(data), fout)
            .unwrap();
        for wave in waves{
            let path = self.export_folder.join(wave.name() + "_file.memb");
            wave.generate_memb(path).unwrap();
        }

        debug!("Generate files");
    }
}

impl Default for ProjectExport {
    fn default() -> Self {
        Self {
            generate_sv: true,
            generate_mem: true,
            export_folder: Path::new("./test").to_path_buf(),
        }
    }
}