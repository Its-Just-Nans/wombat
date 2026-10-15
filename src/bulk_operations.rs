//! Bulk operations

use bladvak::{ErrorManager, eframe::egui};

use crate::{WombatApp, windows::DataView};

/// Bulk operations
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct BulkOperations {
    /// open
    pub(crate) is_open: bool,
    /// selected dataview
    toggle_data_view: DataView,
}

impl Default for BulkOperations {
    fn default() -> Self {
        Self {
            is_open: false,
            toggle_data_view: DataView::Previewer,
        }
    }
}

impl WombatApp {
    /// ui bulk
    fn show_ui_bulk(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        let num_doc = self.documents.len();
        ui.label(format!("Number of documents: {num_doc}"));
        ui.horizontal(|ui| {
            ui.label("Open");
            egui::ComboBox::from_id_salt("data_view_combo_box")
                .selected_text(self.bulk_operations.toggle_data_view.title())
                .show_ui(ui, |ui| {
                    for one_view in DataView::all_non_empty() {
                        ui.selectable_value(
                            &mut self.bulk_operations.toggle_data_view,
                            one_view.clone(),
                            one_view.title(),
                        );
                    }
                });
            ui.label("on all files");
            if ui.button("Toggle").clicked() {
                for one_doc in &mut self.documents {
                    one_doc
                        .windows_data
                        .open(&self.bulk_operations.toggle_data_view);
                }
            }
        });
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
