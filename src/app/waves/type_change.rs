use egui::Ui;
use serde::{Serialize, Deserialize};
use tracing_subscriber::field::display;

use crate::app::windows::WindowResult;

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct TypeChange{

}


impl TypeChange {
    pub(crate) fn display(&mut self, ui: &mut Ui)->WindowResult{
        
        WindowResult::Cancel
    }
}