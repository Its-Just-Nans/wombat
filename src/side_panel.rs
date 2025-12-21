//! Side panel
use bladvak::eframe::egui;
use bladvak::errors::ErrorManager;

use crate::WombatApp;

impl WombatApp {
    /// Side panel
    pub(crate) fn app_side_panel(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        if !self.sidebar_as_window {
            self.file_info(ui);
        }
    }

    /// Image info
    pub(crate) fn file_info(&mut self, ui: &mut egui::Ui) {
        ui.heading("File");
        ui.label("Binary length");
        ui.add(egui::Slider::new(&mut self.bytes_per_line, 1..=64));

        if let Some((select1, select2)) = self.selection.as_mut() {
            ui.label("Selection");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(select1).range(0..=*select2));
                ui.add(egui::DragValue::new(select2).range(*select1..=self.binary_file.len()));
            });
        } else {
            ui.label("No selection");
        }
    }
}
