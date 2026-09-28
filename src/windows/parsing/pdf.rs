//! PDF

use std::ops::RangeInclusive;

use bladvak::eframe::egui;
use pdf::object::InfoDict;

/// Pdf data
#[derive(Debug)]
pub(crate) struct PdfData {
    /// page number
    page_number: u32,
    /// info dict
    info_dict: Option<InfoDict>,
}

impl PdfData {
    /// parse the data
    pub(crate) fn parse(binary_file: &[u8]) -> Result<Self, String> {
        let Ok(old_file) = pdf::file::FileOptions::cached().load(binary_file) else {
            return Err("Cannot parse pdf".to_string());
        };
        Ok(Self {
            page_number: old_file.num_pages(),
            info_dict: old_file.trailer.info_dict,
        })
    }

    /// ui
    pub(crate) fn ui(&self, ui: &mut egui::Ui) -> Option<RangeInclusive<usize>> {
        ui.label(format!("Number pages: {}", self.page_number));
        ui.label(format!("Info: {:#?}", self.info_dict));
        None
    }
}
