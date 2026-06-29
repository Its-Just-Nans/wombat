//! Wombat windows

#[cfg(feature = "detection")]
mod detection;
mod exporter;
#[cfg(feature = "hashing")]
mod hashing;
mod histogram;
mod importer;
mod searcher;
#[cfg(feature = "yara")]
mod yara;

use crate::{WombatApp, panels::FileInfoData, windows::exporter::Exporter};

use bladvak::{ErrorManager, eframe::egui};

use file_format::FileFormat;
use histogram::Histogram;
use importer::Importer;
use searcher::Searcher;

/// File info
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct WindowsData {
    /// Histogram info
    pub(crate) histogram: Histogram,
    /// importer
    pub(crate) importer: Importer,
    /// searcher
    pub(crate) searcher: Searcher,

    /// detection
    #[cfg(feature = "detection")]
    pub(crate) detection: detection::Detection,

    /// hashing
    #[cfg(feature = "hashing")]
    pub(crate) hashing: hashing::Hashing,
    /// exporter
    pub(crate) exporter: Exporter,
    /// yara
    #[cfg(feature = "yara")]
    pub(crate) yara: yara::Yara,
}

impl WindowsData {
    /// Create a new empty window data
    pub(crate) fn new() -> Self {
        Self {
            histogram: Histogram::new(),
            importer: Importer::new(),
            exporter: Exporter::new(),
            #[cfg(feature = "detection")]
            detection: detection::Detection::new(),
            searcher: Searcher::new(),
            #[cfg(feature = "hashing")]
            hashing: hashing::Hashing::new(),
            #[cfg(feature = "yara")]
            yara: yara::Yara::new(),
        }
    }

    /// reset data
    pub(crate) fn reset(&mut self) {
        self.histogram.reset();
        self.importer.reset();
        #[cfg(feature = "detection")]
        self.detection.reset();
        self.searcher.reset();
        self.exporter.reset();
        #[cfg(feature = "hashing")]
        self.hashing.reset();
        #[cfg(feature = "yara")]
        self.yara.reset();
    }

    /// Mark selection stale
    pub(crate) fn selection_stale(&mut self) {
        #[cfg(feature = "hashing")]
        self.hashing.selection_stale();
    }

    /// Ui top bar
    pub(crate) fn ui_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.toggle_value(&mut self.histogram.is_open, "Histogram");
        ui.toggle_value(&mut self.importer.is_open, "Import");
        ui.toggle_value(&mut self.exporter.is_open, "Exporter");
        ui.toggle_value(&mut self.searcher.is_open, "Searcher");
        #[cfg(feature = "detection")]
        ui.toggle_value(&mut self.detection.is_open, "Detection");
        #[cfg(feature = "hashing")]
        ui.toggle_value(&mut self.hashing.is_open, hashing::Hashing::window_title());
        #[cfg(feature = "yara")]
        ui.toggle_value(&mut self.yara.is_open, yara::Yara::window_title());
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
        self.windows_data.exporter.ui(
            &self.binary_file,
            &self.filename,
            &self.selection,
            ui,
            error_manager,
        );
        #[cfg(feature = "hashing")]
        self.windows_data
            .hashing
            .ui(&self.binary_file, &self.selection, ui, error_manager);
        #[cfg(feature = "yara")]
        self.windows_data
            .yara
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
        if let Some(range) =
            self.windows_data
                .searcher
                .ui(&self.binary_file, &self.selection, ui, error_manager)
        {
            self.go_to_range(range);
        }
        #[cfg(feature = "detection")]
        if let Some(infos) = &self.file_format
            && let Some(range) =
                self.windows_data
                    .detection
                    .ui(&self.binary_file, infos, ui, error_manager)
        {
            self.go_to_range(range);
        }
    }
}
