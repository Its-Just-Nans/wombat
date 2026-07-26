//! Wombat windows

#[cfg(feature = "detection")]
mod detection;
#[cfg(feature = "hashing")]
mod hashing;
mod histogram;
mod metadata;
mod searcher;
#[cfg(feature = "yara")]
mod yara;

pub(crate) mod exporter;
pub(crate) mod importer;

use crate::{WombatApp, panels::FileInfoData, windows::metadata::Metadata};

use bladvak::{ErrorManager, eframe::egui};

use file_format::FileFormat;
use histogram::Histogram;
use searcher::Searcher;

/// File info
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct WindowsData {
    /// Histogram info
    pub(crate) histogram: Histogram,
    /// searcher
    pub(crate) searcher: Searcher,
    /// metadata
    pub(crate) metadata: Metadata,
    /// detection
    #[cfg(feature = "detection")]
    pub(crate) detection: detection::Detection,

    /// hashing
    #[cfg(feature = "hashing")]
    pub(crate) hashing: hashing::Hashing,
    /// yara
    #[cfg(feature = "yara")]
    pub(crate) yara: yara::Yara,
}

impl WindowsData {
    /// Create a new empty window data
    pub(crate) fn new() -> Self {
        Self {
            histogram: Histogram::new(),
            metadata: Metadata::default(),
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
        self.metadata.reset();
        #[cfg(feature = "detection")]
        self.detection.reset();
        self.searcher.reset();
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
        ui.toggle_value(&mut self.searcher.is_open, "Searcher");
        ui.toggle_value(&mut self.metadata.is_open, "Metadata");
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
        let Some(document) = self.documents.get_current_doc_mut() else {
            return;
        };
        document
            .windows_data
            .histogram
            .ui(&document.binary_file, ui, error_manager);
        #[cfg(feature = "hashing")]
        document.windows_data.hashing.ui(
            &document.binary_file,
            &document.selection,
            ui,
            error_manager,
        );
        #[cfg(feature = "yara")]
        document
            .windows_data
            .yara
            .ui(&document.binary_file, ui, error_manager);
        if document.file_format.is_none() {
            let file_fmt = FileFormat::from_bytes(&document.binary_file);
            let data = FileInfoData {
                kind: file_fmt.kind(),
                file_type: file_fmt.media_type().to_string(),
                extension: file_fmt.extension().to_string(),
                name: file_fmt.name().to_string(),
            };
            document.file_format = Some(data);
        }

        if let Some(range) = document.windows_data.searcher.ui(
            &document.binary_file,
            &document.selection,
            ui,
            error_manager,
        ) {
            document.go_to_range(range);
        }
        self.show_metadata_ui(ui, error_manager);
        #[cfg(feature = "detection")]
        if let Some(range) = self.show_detection_ui(ui, error_manager)
            && let Some(document) = self.documents.get_current_doc_mut()
        {
            document.go_to_range(range);
        }
    }
}
