use egui::{Ui, Pos2};

mod waves;

use log::debug;
use waves::Wave;

#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct ProjectSettings{
    max_time: usize
}

impl Default for ProjectSettings{
    fn default() -> Self {
        Self { max_time: 16 }
    }
}

#[derive(Default)]
enum AppState{
    #[default]
    Main,
    ProjectSettings(ProjectSettings)
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

    project_setting: ProjectSettings
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
            project_setting: ProjectSettings::default()
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


impl App{
    fn central_panel(&mut self, ui: &mut Ui){
        ui.vertical(|ui|{
            let link_group_id = ui.id().with("link_waves");
            for wave in &mut self.waves{
                wave.display(ui, link_group_id, &self.user_input);
            }
            self.waves.retain(|v|{
                !v.deleted()
            });
            if ui.button("Add").clicked(){
                self.waves.push(Wave::new("Clock", self.project_setting.max_time));
            }
        });
    }

    fn draw_state(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame){
        match self.state{
            AppState::Main => {},
            AppState::ProjectSettings(_) => self.draw_state_settings(ctx, frame),
        }
    }

    fn draw_state_settings(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame){
        let AppState::ProjectSettings(settings) = &mut self.state else {
            self.state = AppState::Main;
            return;
        };
        let mut open = true;
        egui::Window::new("Settings").show(ctx, |ui|{
            ui.vertical(|ui|{
                ui.horizontal(|ui|{
                    ui.label("Time size");
                    ui.add(egui::DragValue::new(&mut settings.max_time));
                })
            }); 
            ui.horizontal(|ui|{
                if ui.button("Save").clicked(){
                    if settings.max_time != self.project_setting.max_time{
                        self.waves.iter_mut().for_each(|w|{
                            w.set_len(settings.max_time);
                        });
                        self.project_setting.max_time = settings.max_time;
                    }
                    if true{
                        //Заглушка
                    }
                }
                if ui.button("Cancel").clicked(){
                    open = false;
                }
            });
        });
        if !open{
            self.state = AppState::Main;
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
        ctx.input(|i|{
            self.user_input = i.clone();
        });


        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if let Some(storage) = frame.storage_mut(){
                        if ui.button("Clear").clicked(){
                            *self = Self::default();
                            eframe::set_value(storage, eframe::APP_KEY, self);
                        }
                    }
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
                ui.menu_button("Project", |ui|{
                    if ui.button("Settings").clicked(){
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
