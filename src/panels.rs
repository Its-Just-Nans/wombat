//! Side panel

use bladvak::app::BladvakPanel;
use bladvak::eframe::egui;
use bladvak::errors::ErrorManager;
use bladvak::{BladvakApp, File};
use std::path::PathBuf;

use crate::WombatApp;
use crate::windows::hex::hex_viewer_settings;

/// File info
#[derive(Debug, Default)]
pub(crate) struct FileInfo;

/// File info
#[derive(Debug)]
pub(crate) struct FileInfoData {
    /// Kind of file
    pub(crate) kind: file_format::Kind,
    /// Type of file
    pub(crate) file_type: String,
    /// Extension of file format
    pub(crate) extension: String,
    /// format name
    pub(crate) name: String,
}

impl BladvakPanel for FileInfo {
    type App = WombatApp;
    fn name(&self) -> &'static str {
        "File info"
    }
    fn has_ui(&self) -> bool {
        true
    }
    fn has_settings(&self) -> bool {
        true
    }
    fn ui(&self, app: &mut WombatApp, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        let Some(document) = app.documents.get_current_doc_mut() else {
            return;
        };
        ui.label(format!("File: {}", document.filename.display()));
        bladvak::utils::show_size(ui, document.binary_file.len());

        if let Some(fmt) = &document.file_format {
            ui.collapsing("File info", |ui| {
                ui.label(format!("Kind: {:?}", fmt.kind));
                ui.label(format!("Type: {}", fmt.file_type));
                ui.label(format!("Name: {}", fmt.name));
                ui.label(format!("Extension: .{}", fmt.extension));
            });
        } else if ui.button("Get file info").clicked() {
            let _ = document.get_file_format();
        }

        ui.separator();
        hex_viewer_settings(ui, document);
    }

    fn ui_settings(
        &self,
        app: &mut WombatApp,
        ui: &mut egui::Ui,
        error_manager: &mut ErrorManager,
    ) {
        if ui.button("Load default file").clicked() {
            let default_file = WombatApp::load_default_file();
            if let Err(err) = app.handle_file(default_file) {
                error_manager.add_error(err);
            }
        }
        if ui.button("Load ASCII").clicked() {
            let ascii_file = File {
                data: (0..=255).collect(),
                path: PathBuf::from("ascii.bin"),
            };
            if let Err(err) = app.handle_file(ascii_file) {
                error_manager.add_error(err);
            }
        }
        ui.checkbox(
            &mut app.display_settings.display_lsb,
            "Show as Least Significant Bit",
        );
        ui.checkbox(
            &mut app.display_settings.limit_to_base_ascii,
            "Limit to base ASCII",
        );
        ui.checkbox(
            &mut app.display_settings.show_color_picker,
            "Show color picker",
        );
    }
}
