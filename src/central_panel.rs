//! Central panel

use bladvak::eframe::egui;
use bladvak::errors::ErrorManager;

use crate::WombatApp;
use crate::windows::DataView;

impl WombatApp {
    /// Show the central panel
    pub(crate) fn app_central_panel(
        &mut self,
        ui: &mut egui::Ui,
        error_manager: &mut ErrorManager,
    ) {
        if self.documents.get_current_doc_mut().is_none() {
            bladvak::utils::central_ui(ui, |ui| {
                ui.heading(concat!("Welcome to ", env!("CARGO_PKG_NAME")));
                ui.label("No document opened");
            });
            return;
        }
        let current_idx = self.documents.get_current_index();
        let current_data_view = self.display_settings.data_view.clone();
        self.show_view(ui, current_idx, error_manager, &current_data_view);
    }

    /// show the data view
    pub(crate) fn show_view(
        &mut self,
        ui: &mut egui::Ui,
        current_idx: usize,
        error_manager: &mut ErrorManager,
        view_type: &DataView,
    ) {
        let Some(document) = self.documents.get_mut(current_idx) else {
            return;
        };
        match view_type {
            DataView::Empty => {}
            DataView::Hex => {
                self.show_hex_inner_ui(ui, error_manager, current_idx);
            }
            DataView::Histogram => {
                document
                    .windows_data
                    .histogram
                    .inner_ui(&document.binary_file, ui);
            }
            DataView::Searcher => {
                if let Some(range) = document.windows_data.searcher.inner_ui(
                    &document.binary_file,
                    &document.selection,
                    ui,
                ) {
                    document.go_to_range(range);
                }
            }
            DataView::Forensics => {
                self.show_detection_inner_ui(ui, error_manager, current_idx);
            }
            DataView::Previewer => {
                let format =
                    file_format::FileFormat::from_extension(&document.file_format.extension)[0];
                document.windows_data.previewer.inner_ui(
                    ui,
                    error_manager,
                    &mut self.fonts_definitions,
                    &document.filename,
                    &document.binary_file,
                    format.kind(),
                );
            }
            DataView::Parsing => {
                if let Some(range) = self.show_parsing_inner_ui(ui, current_idx)
                    && let Some(document) = self.documents.get_current_doc_mut()
                {
                    document.go_to_range(range);
                }
            }
            DataView::Hashing => {
                #[cfg(feature = "hashing")]
                document.windows_data.hashing.inner_ui(
                    &document.binary_file,
                    &document.selection,
                    ui,
                );
            }
            DataView::Yara => {
                document
                    .windows_data
                    .yara
                    .inner_ui(&document.binary_file, ui);
            }
        }
    }
}
