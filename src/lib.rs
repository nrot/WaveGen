#![warn(clippy::all, rust_2018_idioms)]


pub const PROJECT_FILE_NAME: &str = "project-0.1.ron";

mod app;
pub use app::App;
