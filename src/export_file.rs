//! Export file

use std::fmt::Display;

use crate::WombatApp;
use crate::windows::exporter::{ExportType, format_export};
use bladvak::ErrorManager;
use bladvak::eframe::egui;

/// Export to file
#[derive(Debug)]
pub(crate) enum FileExportType {
    /// Raw
    Raw,
    /// Text
    Text(ExportType),
}

impl Display for FileExportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Raw => write!(f, "Raw"),
            Self::Text(exp) => write!(f, "{exp}"),
        }
    }
}

impl FileExportType {
    /// Extension
    fn extension(&self) -> &str {
        match self {
            Self::Raw => "bin",
            Self::Text(_) => "txt",
        }
    }
}

impl WombatApp {
    /// Export file ui
    pub(crate) fn export_file_ui(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        ui.menu_button("Save", |ui| {
            if ui.button(FileExportType::Raw.to_string()).clicked() {
                ui.close();
                self.export_file(FileExportType::Raw, error_manager);
            }
            if ui
                .button(FileExportType::Text(ExportType::Binary).to_string())
                .clicked()
            {
                ui.close();
                self.export_file(FileExportType::Text(ExportType::Binary), error_manager);
            }
            if ui
                .button(FileExportType::Text(ExportType::Hex).to_string())
                .clicked()
            {
                ui.close();
                self.export_file(FileExportType::Text(ExportType::Hex), error_manager);
            }
            if ui
                .button(FileExportType::Text(ExportType::Octal).to_string())
                .clicked()
            {
                ui.close();
                self.export_file(FileExportType::Text(ExportType::Octal), error_manager);
            }
            if ui
                .button(FileExportType::Text(ExportType::Decimal).to_string())
                .clicked()
            {
                ui.close();
                self.export_file(FileExportType::Text(ExportType::Decimal), error_manager);
            }
        });
    }

    /// Export the file
    pub(crate) fn export_file(
        &mut self,
        file_export_type: FileExportType,
        error_manager: &mut ErrorManager,
    ) {
        let Some(document) = self.documents.get_current_doc_mut() else {
            error_manager.add_error("No document to save");
            return;
        };
        let extension = file_export_type.extension();
        let current_save_path =
            if document.filename.extension().and_then(|e| e.to_str()) == Some(extension) {
                document.filename.clone()
            } else {
                document.filename.with_extension(extension)
            };
        let save_path = bladvak::utils::get_save_path(Some(&current_save_path));
        match save_path {
            Ok(save_p) => {
                if let Some(path_to_save) = save_p {
                    document.filename.clone_from(&path_to_save);
                    match file_export_type {
                        FileExportType::Raw => {
                            if let Err(err) =
                                bladvak::utils::save_file(&document.binary_file, &path_to_save)
                            {
                                error_manager.add_error(err);
                            }
                        }
                        FileExportType::Text(exp) => {
                            let export_str = format_export(&document.binary_file, &exp, true, " ");
                            if let Err(err) =
                                bladvak::utils::save_file(export_str.as_bytes(), &path_to_save)
                            {
                                error_manager.add_error(err);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error_manager.add_error(e);
            }
        }
    }
}
