//! Wombat windows

#[cfg(feature = "hashing")]
mod hashing;
mod histogram;
mod metadata;
#[cfg(feature = "parsing")]
mod parsing;
mod previewer;
mod searcher;
#[cfg(feature = "yara")]
mod yara;

pub(crate) mod exporter;
pub(crate) mod importer;

use crate::WombatApp;

use bladvak::{ErrorManager, eframe::egui};

use histogram::Histogram;
use metadata::Metadata;
use previewer::Previewer;
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
    /// previewer
    pub(crate) previewer: Previewer,
    /// parsing
    #[cfg(feature = "parsing")]
    pub(crate) parsing: parsing::Parsing,

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
            previewer: Previewer::default(),
            #[cfg(feature = "parsing")]
            parsing: parsing::Parsing::new(),
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
        self.previewer.reset();
        self.metadata.reset();
        #[cfg(feature = "parsing")]
        self.parsing.reset();
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
        ui.toggle_value(&mut self.previewer.is_open, "Previewer");
        #[cfg(feature = "parsing")]
        ui.toggle_value(&mut self.parsing.is_open, "Parsing");
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
        if let Some(range) = document.windows_data.searcher.ui(
            &document.binary_file,
            &document.selection,
            ui,
            error_manager,
        ) {
            document.go_to_range(range);
        }
        let kind = document.get_file_format().kind;
        document
            .windows_data
            .previewer
            .ui(ui, error_manager, &document.binary_file, kind);
        self.show_metadata_ui(ui, error_manager);
        #[cfg(feature = "parsing")]
        if let Some(range) = self.show_parsing_ui(ui, error_manager)
            && let Some(document) = self.documents.get_current_doc_mut()
        {
            document.go_to_range(range);
        }
    }
}
