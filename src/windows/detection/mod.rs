//! Detection

#![cfg(feature = "detection")]

mod cert;
mod jpg;
mod mp4;
mod png;
mod raw;
mod xml;
mod zip;

use bladvak::eframe::egui::{self};
use bladvak::errors::ErrorManager;
use std::ops::RangeInclusive;

use crate::WombatApp;
use crate::panels::FileInfoData;
use crate::windows::detection::cert::{CertData, show_certs};
use crate::windows::detection::jpg::{JpgData, show_jpg_data};
use crate::windows::detection::mp4::{Mp4Data, ui::show_mp4_ui};
use crate::windows::detection::png::{PngData, show_png_chunks};
use crate::windows::detection::raw::{StringType, show_raw_string_data};
use crate::windows::detection::xml::{XmlData, xml_tree_ui};
use crate::windows::detection::zip::ZipData;

/// Histogram data cache
#[derive(Default, Debug)]
enum DetectionCache {
    /// png data cached
    Png(Option<PngData>),
    /// jpg data cached
    Jpg(Option<JpgData>),
    /// xml data cached
    Xml(Option<XmlData>),
    /// cert data cached
    Cert(Option<CertData>),
    /// mp4 data cached
    Mp4(Option<Mp4Data>),
    /// zip data cached
    Zip(Option<ZipData>),
    /// Message
    Message(String),
    /// Raw String
    RawString(Option<StringType>),
    /// no cache
    #[default]
    Empty,
}

impl DetectionCache {
    /// parse to create cache
    fn parse(binary_data: &[u8], file_info: &FileInfoData) -> DetectionCache {
        match file_info.extension.as_str() {
            "png" => {
                let parsed = PngData::parse(binary_data);
                DetectionCache::Png(parsed)
            }
            "xml" | "svg" | "html" => {
                let parsed = XmlData::parse(binary_data);
                DetectionCache::Xml(Some(parsed))
            }
            "crt" => {
                let parsed = CertData::parse(binary_data, false);
                DetectionCache::Cert(parsed)
            }
            "der" => {
                let parsed = CertData::parse(binary_data, true);
                DetectionCache::Cert(parsed)
            }
            "jpg" => {
                let parsed = JpgData::parse(binary_data);
                DetectionCache::Jpg(parsed)
            }
            "mp4" => {
                let parsed = Mp4Data::parse(binary_data);
                DetectionCache::Mp4(parsed)
            }
            "zip" => {
                let parsed = ZipData::parse(binary_data);
                DetectionCache::Zip(parsed)
            }
            "bin" => {
                let parsed = StringType::parse(binary_data);
                DetectionCache::RawString(parsed)
            }
            "qoi" => DetectionCache::Message("A QOI (Quite OK Image) image".to_string()),
            _ => DetectionCache::Empty,
        }
    }
}

/// Histogram data
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct Detection {
    /// is open
    pub(crate) is_open: bool,

    #[serde(skip)]
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
}

impl WombatApp {
    /// Show detection
    pub(crate) fn show_detection_ui(
        &mut self,
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
    ) -> Option<RangeInclusive<usize>> {
        let detection = &self.documents.get_current_doc()?.windows_data.detection;
        if detection.is_open {
            let mut is_open = detection.is_open;
            let mut ret = None;
            egui::Window::new("Detection")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    let Some(document) = self.documents.get_current_doc_mut() else {
                        return;
                    };
                    let detection = &mut document.windows_data.detection;
                    let binary_data = &document.binary_file;
                    let Some(file_info) = &document.file_format else {
                        return;
                    };
                    ui.label(format!(
                        "Name: {} ({}) - {}",
                        file_info.name, file_info.file_type, file_info.extension
                    ));
                    ui.separator();
                    if matches!(detection.cache, DetectionCache::Empty) {
                        detection.cache = DetectionCache::parse(binary_data, file_info);
                    }
                    let detection_cache = &document.windows_data.detection.cache;
                    ret = match detection_cache {
                        DetectionCache::Png(data) => show_png_chunks(ui, data.as_ref()),
                        DetectionCache::Jpg(data) => show_jpg_data(ui, data.as_ref()),
                        DetectionCache::Xml(xml_str) => xml_tree_ui(ui, xml_str.as_ref()),
                        DetectionCache::Cert(xml_str) => show_certs(ui, xml_str.as_ref()),
                        DetectionCache::Message(str) => {
                            ui.label(str);
                            None
                        }
                        DetectionCache::Mp4(data) => show_mp4_ui(ui, data.as_ref()),
                        DetectionCache::Zip(_) => self.detection_ui_zip(ui),
                        DetectionCache::RawString(data) => show_raw_string_data(ui, data.as_ref()),
                        DetectionCache::Empty => {
                            ui.label("No data");
                            None
                        }
                    };
                });
            if let Some(document) = self.documents.get_current_doc_mut() {
                document.windows_data.detection.is_open = is_open;
            }
            return ret;
        }
        None
    }
}
