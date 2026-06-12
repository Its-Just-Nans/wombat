//! Searcher

use bladvak::eframe::egui;
use bladvak::eframe::egui::Color32;
use bladvak::errors::ErrorManager;
use std::ops::RangeInclusive;

use crate::selection::Selection;
use crate::windows::importer::parse_hex_string;

/// search type
#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) enum SearchType {
    /// hex
    Hex,
    /// decimal
    Text,
}

/// Searcher data
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct Searcher {
    /// is open
    pub(crate) is_open: bool,

    /// Search input
    pub(crate) search: (String, Option<Vec<u8>>),

    /// search type
    pub(crate) search_type: SearchType,

    /// Last search idx
    #[serde(skip)]
    last_search_idx: Option<usize>,

    #[serde(skip)]
    /// error
    search_error: Option<String>,
}

impl Searcher {
    /// New import data
    pub(crate) fn new() -> Self {
        Self {
            is_open: false,
            search: (String::new(), None),
            search_type: SearchType::Text,
            last_search_idx: None,
            search_error: None,
        }
    }

    /// reset data
    pub(crate) fn reset(&mut self) {
        self.search.0.clear();
        if let Some(needle_mut) = self.search.1.as_mut() {
            needle_mut.clear();
        }
        self.search_error = None;
    }

    /// Ui inside the windows of the searcher
    pub(crate) fn windows_ui(
        &mut self,
        binary_file: &[u8],
        selection: &Selection,
        ui: &mut egui::Ui,
    ) -> Option<RangeInclusive<usize>> {
        ui.horizontal(|ui| {
            ui.label("Search as selection to:");
            ui.selectable_value(&mut self.search_type, SearchType::Hex, "Hex");
            ui.selectable_value(&mut self.search_type, SearchType::Text, "Text");
        });

        if binary_file.is_empty() {
            ui.label("File is empty - cannot search");
        } else {
            ui.text_edit_singleline(&mut self.search.0);
        }
        let current_idx = if let Some(range) = selection.range {
            range.0 + 1
        } else {
            0
        };
        self.search.1 = match self.search_type {
            SearchType::Text => Some(self.search.0.as_bytes().to_vec()),
            SearchType::Hex => parse_hex_string(&self.search.0).ok(),
        };
        if self.search.0.is_empty() {
            ui.label("Search is empty");
        } else if let Some(needle) = &self.search.1
            && !needle.is_empty()
        {
            let needle_pretty = needle
                .iter()
                .map(|one_u8| format!("0x{one_u8:02X}"))
                .collect::<Vec<String>>()
                .join(",");
            ui.label(format!("Needle is : [{needle_pretty}]"));
            let mut should_return = false;
            if ui.button("Search next").clicked() {
                let idx_result = binary_file[current_idx..]
                    .windows(needle.len())
                    .position(|window| window == needle);
                if let Some(start_idx) = idx_result {
                    let start_idx = start_idx + current_idx;
                    self.last_search_idx = Some(start_idx);
                    should_return = true
                } else {
                    self.last_search_idx = None;
                }
            }
            if let Some(last_start_idx) = self.last_search_idx {
                ui.label(format!("Found at {last_start_idx}"));
                if should_return {
                    let end_idx = last_start_idx + needle.len().checked_sub(1).unwrap_or(1);
                    return Some(last_start_idx..=end_idx);
                }
                return None;
            }
            ui.label("Nothing");
            return None;
        } else {
            ui.colored_label(Color32::RED, "Cannot parse needle");
        }
        None
    }

    /// Show the search ui
    pub(crate) fn ui(
        &mut self,
        binary_file: &[u8],
        selection: &Selection,
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
    ) -> Option<RangeInclusive<usize>> {
        let mut range = None;
        if self.is_open {
            let mut is_open = self.is_open;
            egui::Window::new("Searcher")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    range = self.windows_ui(binary_file, selection, ui);
                });
            self.is_open = is_open;
        }
        range
    }
}
