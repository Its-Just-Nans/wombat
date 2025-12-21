//! Top panel
use bladvak::eframe::egui::{self};
use bladvak::errors::ErrorManager;

use crate::WombatApp;

impl WombatApp {
    /// Show the file menu
    pub(crate) fn app_menu_file(&mut self, _ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {}

    /// Show the top panel
    pub(crate) fn app_top_panel(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        ui.separator();
        ui.label(format!("File: {}", self.filename.display()));
    }
}
