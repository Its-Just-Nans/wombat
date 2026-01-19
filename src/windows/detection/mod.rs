//! Detection

mod png;
mod xml;

use std::ops::RangeInclusive;

use bladvak::eframe::egui::{self};
use bladvak::errors::ErrorManager;

use crate::panels::FileInfoData;
use crate::windows::detection::png::{PngData, show_png_chunks};
use crate::windows::detection::xml::{XmlData, xml_tree_ui};

/// Histogram data cache
#[derive(Debug)]
enum DetectionCache {
    /// png data cached
    Png(PngData),

    /// xml data cached
    Xml(XmlData),
    /// no cache
    None,
}

impl DetectionCache {
    /// Show the ui of cached data
    fn show(&self, ui: &mut egui::Ui, file_info: &FileInfoData) -> Option<RangeInclusive<usize>> {
        match self {
            DetectionCache::Png(data) => show_png_chunks(ui, data),
            DetectionCache::Xml(xml_str) => {
                xml_tree_ui(ui, xml_str);
                None
            }
            DetectionCache::None => {
                ui.label(format!("Kind: {:?}", file_info.kind));
                None
            }
        }
    }

    /// parse to create cache
    fn parse(binary_data: &[u8], file_info: &FileInfoData) -> DetectionCache {
        match file_info.extension.as_str() {
            "png" => {
                if let Some(data) = PngData::parse(binary_data) {
                    DetectionCache::Png(data)
                } else {
                    DetectionCache::None
                }
            }
            "xml" => {
                let data = XmlData::parse(binary_data);
                DetectionCache::Xml(data)
            }
            _ => DetectionCache::None,
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
            cache: DetectionCache::None,
        }
    }

    /// reset data
    pub(crate) fn reset(&mut self) {
        self.cache = DetectionCache::None;
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
                    if matches!(self.cache, DetectionCache::None) {
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
