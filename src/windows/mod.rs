//! Wombat windows
mod detection;
#[cfg(feature = "hashing")]
mod hashing;
mod histogram;
#[cfg(feature = "parsing")]
mod parsing;
mod previewer;
mod searcher;
#[cfg(feature = "yara")]
mod yara;

pub(crate) mod exporter;
pub(crate) mod hex;
pub(crate) mod importer;

use std::slice::Iter;

use crate::{WombatApp, windows::hex::HexViewer};

use bladvak::{ErrorManager, eframe::egui, utils::document::DocumentTrait};

use detection::Detection;
use histogram::Histogram;
use previewer::Previewer;
use searcher::Searcher;

/// File info
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct WindowsData {
    /// Hex viewer
    pub(crate) hex: HexViewer,
    /// Histogram info
    pub(crate) histogram: Histogram,
    /// searcher
    pub(crate) searcher: Searcher,
    /// detection
    pub(crate) detection: Detection,
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

/// a data view
#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Clone)]
pub(crate) enum DataView {
    /// Empty
    Empty,
    /// hex
    Hex,
    /// histo
    Histogram,
    /// search
    Searcher,
    /// forensics
    Forensics,
    /// previewer
    Previewer,
    /// parsing
    #[cfg(feature = "parsing")]
    Parsing,
    /// hashing
    #[cfg(feature = "hashing")]
    Hashing,
    /// yara
    #[cfg(feature = "yara")]
    Yara,
}

impl DataView {
    /// title
    pub(crate) fn title(&self) -> &str {
        match self {
            Self::Empty => "Empty",
            Self::Hex => "Hex",
            Self::Histogram => "Histogram",
            Self::Searcher => "Searcher",
            Self::Forensics => "Forensics",
            Self::Previewer => "Previewer",
            #[cfg(feature = "parsing")]
            Self::Parsing => "Parsing",
            #[cfg(feature = "hashing")]
            Self::Hashing => "Hashing",
            #[cfg(feature = "yara")]
            Self::Yara => "Yara",
        }
    }

    /// all data views
    pub(crate) fn all() -> Iter<'static, DataView> {
        [
            DataView::Hex,
            DataView::Histogram,
            DataView::Searcher,
            DataView::Forensics,
            DataView::Previewer,
            DataView::Parsing,
            DataView::Hashing,
            DataView::Yara,
        ]
        .iter()
    }
}

/// show toggle value if not current
fn show_toggle_value(
    ui: &mut egui::Ui,
    current_value: &DataView,
    is_open: &mut bool,
    selector: &DataView,
) {
    if current_value != selector {
        ui.toggle_value(is_open, selector.title());
    }
}

impl WindowsData {
    /// Create a new empty window data
    pub(crate) fn new() -> Self {
        Self {
            hex: HexViewer::default(),
            histogram: Histogram::new(),
            detection: Detection::default(),
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
        self.hex.reset();
        self.histogram.reset();
        self.previewer.reset();
        self.detection.reset();
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
    pub(crate) fn ui_top_bar(&mut self, ui: &mut egui::Ui, current_value: &DataView) {
        show_toggle_value(ui, current_value, &mut self.hex.is_open, &DataView::Hex);
        show_toggle_value(
            ui,
            current_value,
            &mut self.histogram.is_open,
            &DataView::Histogram,
        );
        show_toggle_value(
            ui,
            current_value,
            &mut self.searcher.is_open,
            &DataView::Searcher,
        );
        show_toggle_value(
            ui,
            current_value,
            &mut self.detection.is_open,
            &DataView::Forensics,
        );
        #[cfg(feature = "parsing")]
        show_toggle_value(
            ui,
            current_value,
            &mut self.parsing.is_open,
            &DataView::Parsing,
        );
        show_toggle_value(
            ui,
            current_value,
            &mut self.previewer.is_open,
            &DataView::Previewer,
        );
        #[cfg(feature = "hashing")]
        show_toggle_value(
            ui,
            current_value,
            &mut self.hashing.is_open,
            &DataView::Hashing,
        );
        #[cfg(feature = "yara")]
        show_toggle_value(ui, current_value, &mut self.yara.is_open, &DataView::Yara);
    }
}

impl WombatApp {
    /// Show a view
    pub(crate) fn show_view_window(
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
                self.show_hex_window_ui(ui, error_manager);
            }
            DataView::Histogram => {
                document
                    .windows_data
                    .histogram
                    .window_ui(&document.binary_file, ui, error_manager);
            }
            DataView::Searcher => {
                if let Some(range) = document.windows_data.searcher.window_ui(
                    &document.binary_file,
                    &document.selection,
                    ui,
                    error_manager,
                ) {
                    document.go_to_range(range);
                }
            }
            DataView::Forensics => {
                self.show_detection_ui(ui, error_manager);
            }
            DataView::Previewer => {
                let kind = document.get_file_format().kind;
                document.windows_data.previewer.window_ui(
                    ui,
                    error_manager,
                    &mut self.fonts_definitions,
                    &format!("{}", document.name()),
                    &document.binary_file,
                    kind,
                );
            }
            DataView::Parsing => {
                if let Some(range) = self.show_parsing_window_ui(ui, error_manager)
                    && let Some(document) = self.documents.get_current_doc_mut()
                {
                    document.go_to_range(range);
                }
            }
            DataView::Hashing => {
                #[cfg(feature = "hashing")]
                document.windows_data.hashing.window_ui(
                    &document.binary_file,
                    &document.selection,
                    ui,
                    error_manager,
                );
            }
            DataView::Yara => {
                document
                    .windows_data
                    .yara
                    .window_ui(&document.binary_file, ui, error_manager);
            }
        }
    }

    /// Display windows
    pub(crate) fn ui_windows(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        let current_idx = self.documents.get_current_index();
        for view_type in DataView::all() {
            if &self.display_settings.data_view != view_type {
                self.show_view_window(ui, current_idx, error_manager, view_type);
            }
        }
    }
}
