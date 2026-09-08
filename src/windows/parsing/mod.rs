//! Parsing

#![cfg(feature = "parsing")]

mod cert;
pub(crate) mod jpg;
mod mp4;
mod pmtiles;
pub(crate) mod png;
mod raw;
mod xml;
mod zip;

use bladvak::eframe::egui::{self};
use bladvak::errors::ErrorManager;
use std::ops::RangeInclusive;
use std::path::Path;

use crate::WombatApp;
use crate::panels::FileInfoData;
use crate::windows::parsing::cert::{CertData, show_certs};
use crate::windows::parsing::jpg::{JpgData, show_jpg_data};
use crate::windows::parsing::mp4::{Mp4Data, ui::show_mp4_ui};
use crate::windows::parsing::png::PngData;
use crate::windows::parsing::raw::StringType;
use crate::windows::parsing::xml::XmlData;
use crate::windows::parsing::zip::ZipData;
use pmtiles::{PmTilesData, show_pmtiles_ui};

/// Histogram data cache
#[derive(Default, Debug)]
enum ParsingCache {
    /// png data cached
    Png(PngData),
    /// jpg data cached
    Jpg(Option<JpgData>),
    /// xml data cached
    Xml(XmlData),
    /// cert data cached
    Cert(Option<CertData>),
    /// mp4 data cached
    Mp4(Option<Mp4Data>),
    /// zip data cached
    Zip(ZipData),
    /// pmtiles
    PmTiles(Option<PmTilesData>),
    /// Message
    Message(String),
    /// Raw String
    RawString(StringType),
    /// Error
    ErrorMessage(String),
    /// no cache
    #[default]
    Empty,
}

impl ParsingCache {
    /// parse to create cache
    fn parse(binary_data: &[u8], file_info: &FileInfoData, filename: &Path) -> Self {
        let filename_ext = filename.extension().unwrap_or_default().to_string_lossy();
        for extension in &[file_info.extension.as_str(), &filename_ext] {
            if let Some(parsing_cache) = Self::parse_from_extension(binary_data, extension) {
                return parsing_cache;
            }
        }
        if let Some(parsed) = StringType::parse(binary_data) {
            return ParsingCache::RawString(parsed);
        }
        ParsingCache::Empty
    }

    /// parse to create cache
    fn parse_from_extension(binary_data: &[u8], extension: &str) -> Option<Self> {
        let parsing_cache = match extension {
            "png" => {
                let parsed = match PngData::parse(binary_data) {
                    Ok(parsed) => parsed,
                    Err(err) => return Some(ParsingCache::ErrorMessage(err)),
                };
                ParsingCache::Png(parsed)
            }
            "xml" | "svg" | "html" => {
                let parsed = XmlData::parse(binary_data);
                ParsingCache::Xml(parsed)
            }
            "crt" => {
                let parsed = CertData::parse(binary_data, false);
                ParsingCache::Cert(parsed)
            }
            "der" => {
                let parsed = CertData::parse(binary_data, true);
                ParsingCache::Cert(parsed)
            }
            "jpg" => {
                let parsed = JpgData::parse(binary_data);
                ParsingCache::Jpg(parsed)
            }
            "mp4" => {
                let parsed = Mp4Data::parse(binary_data);
                ParsingCache::Mp4(parsed)
            }
            "zip" => {
                let parsed = match ZipData::parse(binary_data) {
                    Ok(parsed) => parsed,
                    Err(err) => return Some(ParsingCache::ErrorMessage(err)),
                };
                ParsingCache::Zip(parsed)
            }
            "qoi" => ParsingCache::Message("A QOI (Quite OK Image) image".to_string()),
            "pmtiles" => ParsingCache::PmTiles(PmTilesData::parse(binary_data)),
            _ => {
                return None;
            }
        };
        Some(parsing_cache)
    }
}

/// Histogram data
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct Parsing {
    /// is open
    pub(crate) is_open: bool,

    #[serde(skip)]
    /// cached data
    cache: ParsingCache,
}

impl Parsing {
    /// New import data
    pub(crate) fn new() -> Self {
        Self {
            is_open: false,
            cache: ParsingCache::Empty,
        }
    }

    /// reset data
    pub(crate) fn reset(&mut self) {
        self.cache = ParsingCache::Empty;
    }
}

impl WombatApp {
    /// Show detection
    pub(crate) fn show_parsing_ui(
        &mut self,
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
    ) -> Option<RangeInclusive<usize>> {
        let parsing = &self.documents.get_current_doc()?.windows_data.parsing;
        let current_index = self.documents.get_current_index();
        if parsing.is_open {
            let mut is_open = parsing.is_open;
            let mut ret = None;
            egui::Window::new("Parsing")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    let Some(document) = self.documents.get_current_doc_mut() else {
                        return;
                    };
                    let _ = document.get_file_format();
                    let Some(file_info) = &document.file_format else {
                        return;
                    };
                    ui.label(format!(
                        "Name: {} ({}) - {}",
                        file_info.name, file_info.file_type, file_info.extension
                    ));
                    ui.separator();
                    if matches!(document.windows_data.parsing.cache, ParsingCache::Empty) {
                        let binary_data = &document.binary_file;
                        document.windows_data.parsing.cache =
                            ParsingCache::parse(binary_data, file_info, &document.filename);
                    }
                    let parsing_cache = &document.windows_data.parsing.cache;
                    ret = match parsing_cache {
                        ParsingCache::Png(data) => data.ui(ui),
                        ParsingCache::Jpg(data) => show_jpg_data(ui, data.as_ref()),
                        ParsingCache::Xml(data) => data.ui(ui),
                        ParsingCache::Cert(xml_str) => show_certs(ui, xml_str.as_ref()),
                        ParsingCache::Message(str) => {
                            ui.label(str);
                            None
                        }
                        ParsingCache::Mp4(data) => show_mp4_ui(ui, data.as_ref()),
                        ParsingCache::Zip(_) => self.parsing_ui_zip(ui),
                        ParsingCache::PmTiles(data) => show_pmtiles_ui(ui, data.as_ref()),
                        ParsingCache::RawString(data) => data.ui(ui),
                        ParsingCache::ErrorMessage(err) => {
                            ui.label(err);
                            None
                        }
                        ParsingCache::Empty => {
                            ui.label("No data");
                            None
                        }
                    };
                });
            if let Some(document) = self.documents.get_mut(current_index) {
                document.windows_data.parsing.is_open = is_open;
            }
            return ret;
        }
        None
    }
}
