//! Wombat windows

mod detection;
mod histogram;
mod importer;

use crate::{WombatApp, panels::FileInfoData};

use bladvak::{ErrorManager, eframe::egui};

use detection::Detection;
use file_format::FileFormat;
use histogram::Histogram;
use importer::Importer;

/// File info
#[derive(Debug)]
pub struct WindowsData {
    /// Histogram info
    pub(crate) histogram: Histogram,
    /// importer
    pub(crate) importer: Importer,
    /// detection
    pub(crate) detection: Detection,
}

impl WindowsData {
    /// Create a new empty window data
    pub(crate) fn new() -> Self {
        Self {
            histogram: Histogram::new(),
            importer: Importer::new(),
            detection: Detection::new(),
        }
    }

    /// reset data
    pub(crate) fn reset(&mut self) {
        self.histogram.reset();
        self.importer.reset();
        self.detection.reset();
    }

    /// Ui top bar
    pub(crate) fn ui_top_bar(&mut self, ui: &mut egui::Ui) {
        if ui.button("Histogram").clicked() {
            self.histogram.is_open = true;
        }
        if ui.button("Import").clicked() {
            self.importer.is_open = true;
        }
        if ui.button("Detection").clicked() {
            self.detection.is_open = true;
        }
    }
}

impl WombatApp {
    /// Display windows
    pub(crate) fn ui_windows(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        use bladvak::{BladvakApp, File};
        use std::path::PathBuf;
        self.windows_data
            .histogram
            .ui(&self.binary_file, ui, error_manager);
        if let Some(data) = self.windows_data.importer.ui(ui, error_manager)
            && let Err(e) = self.handle_file(File {
                data,
                path: PathBuf::from("imported.bin"),
            })
        {
            error_manager.add_error(e);
        }
        if self.file_format.is_none() {
            let file_fmt = FileFormat::from_bytes(&self.binary_file);
            let data = FileInfoData {
                kind: file_fmt.kind(),
                file_type: file_fmt.media_type().to_string(),
                extension: file_fmt.extension().to_string(),
                name: file_fmt.name().to_string(),
            };
            self.file_format = Some(data);
        }
        if let Some(infos) = &self.file_format
            && let Some(range) =
                self.windows_data
                    .detection
                    .ui(&self.binary_file, infos, ui, error_manager)
        {
            self.selection = Some((*range.start(), *range.end()));
        }
    }
}
