//! Detection

mod cert;
mod png;
mod xml;

use std::ops::RangeInclusive;

use bladvak::eframe::egui::{self};
use bladvak::errors::ErrorManager;

use crate::panels::FileInfoData;
use crate::windows::detection::cert::{CertData, show_certs};
use crate::windows::detection::png::{PngData, show_png_chunks};
use crate::windows::detection::xml::{XmlData, xml_tree_ui};

/// Histogram data cache
#[derive(Debug)]
enum DetectionCache {
    /// png data cached
    Png(Option<PngData>),
    /// xml data cached
    Xml(Option<XmlData>),
    /// cert data cached
    Cert(Option<CertData>),
    /// no cache
    Empty,
}

impl DetectionCache {
    /// Show the ui of cached data
    fn show(&self, ui: &mut egui::Ui, file_info: &FileInfoData) -> Option<RangeInclusive<usize>> {
        match self {
            DetectionCache::Png(data) => show_png_chunks(ui, data.as_ref()),
            DetectionCache::Xml(xml_str) => xml_tree_ui(ui, xml_str.as_ref()),
            DetectionCache::Cert(xml_str) => show_certs(ui, xml_str.as_ref()),
            DetectionCache::Empty => {
                ui.label(format!("Kind: {:?}", file_info.kind));
                ui.label("No data");
                None
            }
        }
    }

    /// parse to create cache
    fn parse(binary_data: &[u8], file_info: &FileInfoData) -> DetectionCache {
        match file_info.extension.as_str() {
            "png" => {
                let parsed = PngData::parse(binary_data);
                DetectionCache::Png(parsed)
            }
            "xml" => {
                let parsed = XmlData::parse(binary_data);
                DetectionCache::Xml(Some(parsed))
            }
            "crt" => {
                let parsed = CertData::parse(binary_data);
                DetectionCache::Cert(parsed)
            }
            _ => DetectionCache::Empty,
        }
    }
}

/// Histogram data
#[derive(Debug)]
pub(crate) struct Detection {
    /// is open
    pub(crate) is_open: bool,

    /// cached data
    cache: DetectionCache,
}

impl Detection {
    /// New import data
    pub(crate) fn new() -> Self {
        Self {
            is_open: false,
            cache: DetectionCache::Empty,
        }
    }

    /// reset data
    pub(crate) fn reset(&mut self) {
        self.cache = DetectionCache::Empty;
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
                    if matches!(self.cache, DetectionCache::Empty) {
                        self.cache = DetectionCache::parse(binary_data, file_info);
                    }
                    ret = self.cache.show(ui, file_info);
                });
            self.is_open = is_open;
            return ret;
        }
        None
    }
}
