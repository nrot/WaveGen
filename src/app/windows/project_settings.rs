use super::WindowResult;

#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ProjectSettings {
    pub max_time: usize,
}

impl ProjectSettings {
    pub fn display(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> WindowResult {
        let mut state = WindowResult::Open;
        egui::Window::new("Settings").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Time size");
                    ui.add(egui::DragValue::new(&mut self.max_time));
                })
            });
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    state = WindowResult::Save;
                }
                if ui.button("Cancel").clicked() {
                    state = WindowResult::Cancel;
                }
            });
        });
        state
    }
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self { max_time: 16 }
    }
}
