use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use handlebars::to_json;
use log::{debug, error};

use crate::{app::waves::Wave, hseparator};

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
            ui.checkbox(&mut self.generate_sv, "Generate sv file");
            ui.checkbox(&mut self.generate_mem, "Generate mem file");
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

    pub fn generate_data(&mut self, waves: &Vec<Wave>) -> Result<()> {
        self.generate_sv_file(waves)?;
        self.generate_mem_files(waves)?;

        debug!("Generate files");
        Ok(())
    }

    fn generate_sv_file(&mut self, waves: &Vec<Wave>) -> Result<()> {
        if self.generate_sv {
            let Some(w) = waves.first() else {
                error!("Nothing to generate add one signal");
                return Err(anyhow!("Nothing to generate. Add atleast one signal"));
            };
            let mut hand = handlebars::Handlebars::new();
            hand.register_template_file(TEMPLATE_NAME, "templates/test.hbs")?;
            let fout = std::fs::File::create(self.export_folder.join("test.sv"))?;

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

            hand.render_to_write(TEMPLATE_NAME, &to_json(data), fout)?;
        }
        Ok(())
    }

    fn generate_mem_files(&mut self, waves: &Vec<Wave>) -> Result<(), std::io::Error> {
        if self.generate_mem {
            for wave in waves {
                let path = self.export_folder.join(wave.name() + "_file.memb");
                wave.generate_memb(path)?;
            }
        }
        Ok(())
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
