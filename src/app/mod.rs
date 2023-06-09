use std::{
    io::Write,
    path::PathBuf,
};

use anyhow::anyhow;
use egui::{Ui, Vec2};

mod waves;
mod widgets;
mod windows;

use waves::Wave;
use zip::write::FileOptions;

use crate::{hseparator, PROJECT_FILE_NAME};
use windows::{ProjectExport, ProjectSettings};

use self::windows::ImportData;
pub use waves::WaveType;

#[derive(Default)]
enum AppState {
    #[default]
    Main,
    ProjectSettings(ProjectSettings),
    ProjectExport(ProjectExport),
    ImportData(ImportData),
    Error(anyhow::Error),
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    value: f32,

    waves: Vec<Wave>,

    #[serde(skip)]
    user_input: egui::InputState,

    #[serde(skip)]
    state: AppState,

    project_setting: ProjectSettings,

    #[serde(skip)]
    project_file: Option<PathBuf>,

    #[serde(skip)]
    window_size: Vec2,
}

impl Default for App {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            waves: Vec::new(),
            user_input: egui::InputState::default(),
            state: AppState::Main,
            project_setting: ProjectSettings::default(),
            window_size: Vec2::ZERO,
            project_file: None,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl App {
    fn central_panel(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical()
            .drag_to_scroll(false)
            .show(ui, |ui| {
                // })
                // ui.vertical(|ui|{
                let link_group_id = ui.id().with("link_waves");
                for wave in &mut self.waves {
                    wave.current_size.x = ui.available_width();
                    wave.display(ui, link_group_id, &self.user_input);
                    let s = ui
                        .add(egui::Separator::default().horizontal())
                        .interact(egui::Sense {
                            click: true,
                            drag: true,
                            focusable: true,
                        });
                    if s.dragged() {
                        wave.current_size.y += s.drag_delta().y;
                        s.on_hover_cursor(egui::CursorIcon::Grabbing);
                    } else {
                        s.on_hover_cursor(egui::CursorIcon::Grab);
                    }
                }
                self.waves.retain(|v| !v.deleted());
                if ui.button("Add").clicked() {
                    self.waves.push(Wave::new(
                        format!("Wire {}", self.waves.len()),
                        self.project_setting.max_time,
                        Vec2::new(ui.available_width(), self.window_size.y / 10.0),
                    ));
                }
            });
    }

    fn draw_state(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match self.state {
            AppState::Main => {}
            AppState::ProjectSettings(_) => self.draw_state_settings(ctx, frame),
            AppState::ProjectExport(_) => self.draw_state_export(ctx, frame),
            AppState::Error(_) => self.draw_state_error(ctx, frame),
            AppState::ImportData(_) => self.draw_state_import_data(ctx, frame),
        }
    }

    fn draw_state_export(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let AppState::ProjectExport(settings) = &mut self.state else {
            self.state = AppState::Main;
            return;
        };
        match settings.display(ctx, frame) {
            windows::WindowResult::Open => {}
            windows::WindowResult::Save => {
                match settings.generate_data(&self.waves) {
                    Ok(()) => self.state = AppState::Main,
                    Err(e) => self.state = AppState::Error(e),
                };
            }
            windows::WindowResult::Cancel | windows::WindowResult::Close => {
                self.state = AppState::Main;
            }
            windows::WindowResult::Error(e)=>{
                self.state = AppState::Error(e);
            }
        }
    }

    fn draw_state_settings(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let AppState::ProjectSettings(settings) = &mut self.state else {
            self.state = AppState::Main;
            return;
        };
        match settings.display(ctx, frame) {
            windows::WindowResult::Open => {}
            windows::WindowResult::Save => {
                if settings.max_time != self.project_setting.max_time {
                    self.waves.iter_mut().for_each(|w| {
                        w.set_len(settings.max_time);
                    });
                    self.project_setting.max_time = settings.max_time;
                }
                self.state = AppState::Main;
            }
            windows::WindowResult::Cancel | windows::WindowResult::Close => {
                self.state = AppState::Main;
            }
            windows::WindowResult::Error(e)=>{
                self.state = AppState::Error(e);
            }
        }
    }
    fn draw_state_error(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let AppState::Error(e) = &self.state {
            let mut open = true;
            egui::Window::new("Error").show(ctx, |ui| {
                hseparator!(ui);
                ui.label(egui::RichText::new(e.to_string()).color(egui::Color32::RED));
                hseparator!(ui);
                if ui.button("Ok").clicked() {
                    open = false;
                }
            });
            if !open {
                self.state = AppState::Main;
            }
        }
    }

    fn draw_state_import_data(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame){
        let AppState::ImportData(import) = &mut self.state else {
            self.state = AppState::Main;
            return;
        };
        match import.display(ctx, _frame) {
            windows::WindowResult::Open => {},
            windows::WindowResult::Save => todo!(),
            windows::WindowResult::Cancel | windows::WindowResult::Close => {self.state = AppState::Main},
            windows::WindowResult::Error(e)=>{
                self.state = AppState::Error(e);
            }
        }
    }

    //TODO: Rewrite to ->Result
    fn save_to_file(&mut self)->Result<(), anyhow::Error> {
        if self.project_file.is_none() {
            self.project_file = rfd::FileDialog::new().add_filter("", &["wg"]).save_file();
        }
        if let Some(p) = &self.project_file {
            let f = std::fs::File::create(p)?;
            let mut zip = zip::ZipWriter::new(f);
            zip.start_file(PROJECT_FILE_NAME, FileOptions::default())?;
            let buff = ron::ser::to_string(&self)?;
            zip.write_all(buff.as_bytes())?;
            zip.finish()?;
        }
        Ok(())
    }
    fn open_project(&mut self) {
        let Some(p) = rfd::FileDialog::new().add_filter("", &["wg"]).pick_file() else {
            return;
        };
        let f = match std::fs::File::open(p) {
            Ok(f) => f,
            Err(e) => {
                self.state = AppState::Error(e.into());
                return;
            }
        };
        let mut zip = match zip::ZipArchive::new(f) {
            Ok(z) => z,
            Err(e) => {
                self.state = AppState::Error(anyhow!("This is not wave gen project").context(e));
                return;
            }
        };
        let zf = match zip.by_name(PROJECT_FILE_NAME) {
            Ok(zf) => zf,
            Err(e) => {
                self.state = AppState::Error(anyhow!("This is not wave gen project").context(e));
                return;
            }
        };
        *self = match ron::de::from_reader(zf) {
            Ok(s) => s,
            Err(e) => {
                self.state = AppState::Error(anyhow!("This is not wave gen project").context(e));
                return;
            }
        }
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // let Self { label, value, waves, max_time } = self;
        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::S) {
                if let Err(e) = self.save_to_file(){
                    self.state = AppState::Error(e);
                };
            }
            self.user_input = i.clone();
        });

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        self.window_size = frame.info().window_info.size;

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        self.open_project();
                    }
                    if ui.button("Save").clicked() {
                        if let Err(e) = self.save_to_file(){
                            self.state = AppState::Error(e);
                        };
                    }

                    hseparator!(ui);
                    if let Some(storage) = frame.storage_mut() {
                        if ui.button("Clear").clicked() {
                            *self = Self::default();
                            eframe::set_value(storage, eframe::APP_KEY, self);
                        }
                    }
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
                ui.menu_button("Project", |ui| {
                    if ui.button("Import").clicked(){
                        self.state = AppState::ImportData(ImportData::default());
                    }
                    hseparator!(ui);
                    if ui.button("Generate").clicked() {
                        self.state = AppState::ProjectExport(ProjectExport::default());
                    }
                    if ui.button("Settings").clicked() {
                        self.state = AppState::ProjectSettings(self.project_setting);
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.central_panel(ui);
        });

        self.draw_state(ctx, frame);
    }
}
