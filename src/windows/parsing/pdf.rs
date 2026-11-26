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
    pub(crate) fn try_parse(binary_file: Arc<Vec<u8>>) -> Result<Self, String> {
        let Ok(file) = Pdf::new(binary_file) else {
            return Err("Cannot parse pdf".to_string());
        };
        let page_number = file.pages().len();
        Ok(Self {
            page_number,
            info_dict: file.metadata().clone(),
        })
    }

    /// ui
    pub(crate) fn ui(&self, ui: &mut egui::Ui) -> Option<RangeInclusive<usize>> {
        ui.label(format!("Number pages: {}", self.page_number));
        ui.label("Info:");
        ui.label(format!("Creation date: {:?}", self.info_dict.creation_date));
        ui.label(format!(
            "Modification date: {:?}",
            self.info_dict.modification_date
        ));
        ui.label(format!(
            "Title: {:?}",
            print_value(self.info_dict.title.as_deref())
        ));
        ui.label(format!(
            "Author: {:?}",
            print_value(self.info_dict.author.as_deref())
        ));
        ui.label(format!(
            "Subject: {:?}",
            print_value(self.info_dict.subject.as_deref())
        ));
        ui.label(format!(
            "Keywords: {:?}",
            print_value(self.info_dict.keywords.as_deref())
        ));
        ui.label(format!(
            "Creator: {:?}",
            print_value(self.info_dict.creator.as_deref())
        ));
        ui.label(format!(
            "Producer: {:?}",
            print_value(self.info_dict.producer.as_deref())
        ));
        None
    }
}

/// print the value
#[inline]
fn print_value(v: Option<&[u8]>) -> String {
    if let Some(bytes) = v {
        if let Ok(str) = std::str::from_utf8(bytes) {
            str.to_string()
        } else {
            let (encoding, data) = match &bytes[..2] {
                [0xFF, 0xFE] => (true, &bytes[2..]),  // Little Endian
                [0xFE, 0xFF] => (false, &bytes[2..]), // Big Endian
                _ => (true, bytes),
            };
            let utf16: Vec<u16> = data
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| {
                    if encoding {
                        u16::from_le_bytes([c[0], c[1]])
                    } else {
                        u16::from_be_bytes([c[0], c[1]])
                    }
                })
                .collect();

            String::from_utf16_lossy(&utf16)
        }
    } else {
        "None".to_string()
    }
}
