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
    /// Show the modal
    pub(crate) fn show_bulk_modal(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        ui.label("Bulk operations");
        if ui.button("Open previewer on files").clicked() {
            for one_doc in &mut self.documents {
                one_doc.windows_data.previewer.is_open = true;
            }
        }
    }
}
