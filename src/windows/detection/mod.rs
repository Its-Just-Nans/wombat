//! Detection

mod png;
mod xml;

use std::ops::RangeInclusive;

use bladvak::eframe::egui::{self, TextBuffer};
use bladvak::errors::ErrorManager;

use crate::panels::FileInfoData;
use crate::windows::detection::png::show_png_chunks;
use crate::windows::detection::xml::xml_tree_ui;

/// Histogram data
#[derive(Debug)]
pub(crate) struct Detection {
    /// is open
    pub(crate) is_open: bool,
}

impl Detection {
    /// New import data
    pub(crate) fn new() -> Self {
        Self { is_open: false }
    }

    /// Show the detection ui
    pub(crate) fn ui(
        &mut self,
        binary_data: &[u8],
        file_info: &FileInfoData,
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
    ) -> Option<RangeInclusive<usize>> {
        if self.is_open {
            let mut is_open = self.is_open;
            let mut ret = None;
            egui::Window::new("Detection")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    ui.label(format!(
                        "Name: {} ({}) - {}",
                        file_info.name, file_info.file_type, file_info.extension
                    ));
                    ui.separator();
                    match file_info.extension.as_str() {
                        "png" => {
                            ret = show_png_chunks(ui, binary_data);
                        }
                        "xml" => {
                            let xml_str = String::from_utf8_lossy(binary_data);
                            xml_tree_ui(ui, xml_str.as_str());
                        }
                        _ => {
                            ui.label(format!("Kind: {:?}", file_info.kind));
                        }
                    }
                });
            self.is_open = is_open;
            return ret;
        }
        None
    }
}
