//! Bulk operations

use bladvak::{ErrorManager, eframe::egui};

use crate::WombatApp;

/// Bulk operations
#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub(crate) struct BulkOperations {
    /// open
    pub(crate) is_open: bool,
}

impl WombatApp {
    /// ui bulk
    fn show_ui_bulk(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        let num_doc = self.documents.len();
        ui.label(format!("Number of documents: {num_doc}"));
        if ui.button("Open previewer on files").clicked() {
            for one_doc in &mut self.documents {
                one_doc.windows_data.previewer.is_open = true;
            }
        }
    }

    /// Show the importer ui
    pub(crate) fn ui_bulk(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        if self.bulk_operations.is_open {
            let mut is_open = self.bulk_operations.is_open;
            egui::Window::new("Bulk operations")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    self.show_ui_bulk(ui, error_manager);
                });
            self.bulk_operations.is_open = is_open;
        }
    }
}
