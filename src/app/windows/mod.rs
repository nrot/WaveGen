mod project_settings;
mod project_export;

pub use project_settings::ProjectSettings;
pub use project_export::ProjectExport;

#[derive(PartialEq, Eq)]
pub enum WindowResult {
    Open,
    Save,
    Cancel,
    Close
}