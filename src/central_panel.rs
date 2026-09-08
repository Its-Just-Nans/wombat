//! Central panel

use bladvak::eframe::egui;
use bladvak::errors::ErrorManager;

use crate::WombatApp;

impl WombatApp {
    /// Show the central panel
    pub(crate) fn app_central_panel(
        &mut self,
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
    ) {
        if self.documents.get_current_doc_mut().is_none() {
            bladvak::utils::central_ui(ui, |ui| {
                ui.heading(concat!("Welcome to ", env!("CARGO_PKG_NAME")));
                ui.label("No document opened");
            });
            return;
        }
        self.show_hex(ui);
    }
}
