//! PDF

use std::{ops::RangeInclusive, sync::Arc};

use bladvak::eframe::egui;
use hayro::hayro_syntax::{Pdf, metadata::Metadata};

/// Pdf data
#[derive(Debug)]
pub(crate) struct PdfData {
    /// page number
    page_number: usize,
    /// info dict
    info_dict: Metadata,
}

impl PdfData {
    /// parse the data
    pub(crate) fn parse(binary_file: Arc<Vec<u8>>) -> Result<Self, String> {
        let Ok(file) = Pdf::new(binary_file) else {
            return Err("Cannot parse pdf".to_string());
        };
        let page_number = file.pages().iter().count();
        Ok(Self {
            page_number,
            info_dict: file.metadata().clone(),
        })
    }

    /// ui
    pub(crate) fn ui(&self, ui: &mut egui::Ui) -> Option<RangeInclusive<usize>> {
        ui.label(format!("Number pages: {}", self.page_number));
        ui.label(format!("Info: {:#?}", self.info_dict));
        None
    }
}
