//! Detection

pub(crate) mod exif;
pub(crate) mod gif;
pub(crate) mod pdf;
pub(crate) mod pdf_image;
pub(crate) mod pdf_signature;
pub(crate) mod png;

use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

use bladvak::ErrorManager;
use bladvak::eframe::egui::{self, Color32};
use cms::der::{DateTime, Encode};
use time::OffsetDateTime;

use crate::WombatApp;
use crate::document::Document;
use crate::windows::detection::exif::ExifData;
use crate::windows::detection::gif::extract_gif_images;
use crate::windows::detection::pdf::{extract_pdf_all_pages, extract_pdf_page};
use crate::windows::detection::pdf_signature::{ByteRangeResult, extract_pdf_signatures};
use crate::windows::detection::png::do_acropalypse;
use crate::windows::parsing::ParsingCache;
use crate::windows::parsing::png::{INVALID_PNG_CHUNK, PngData, PngDataViewer};

/// Possible actions
#[derive(Debug)]
pub enum Action {
    /// Acropalypse issue
    Acropalypse(u32, u32, bool),
    /// Extract images
    PdfExtractImages,
    /// Extract page
    PdfExtractPage(usize),
    /// Extract Signature
    PdfExtractSignature(Option<Vec<ByteRangeResult>>),
    /// Extract gif
    ExtractGif,
}

/// Detection
#[derive(serde::Serialize, serde::Deserialize, Default, Debug)]
pub(crate) struct Detection {
    /// Is open
    pub(crate) is_open: bool,
    /// exif
    exif_data: Option<Result<ExifData, String>>,
    /// Actions
    #[serde(skip)]
    actions: Vec<Action>,
}

impl Detection {
    /// title
    pub(crate) fn title() -> &'static str {
        "Forensics"
    }

    /// reset
    pub(crate) fn reset(&mut self) {
        self.exif_data = None;
        self.actions = Vec::new();
    }
}

impl WombatApp {
    /// detection prepare ui
    pub(crate) fn detection_prepare_ui(&mut self, error_manager: &mut ErrorManager) {
        let Some(document) = self.documents.get_current_doc_mut() else {
            return;
        };
        let format = document.get_file_format().clone();
        document.windows_data.detection.exif_data =
            Some(ExifData::parse(&document.binary_file, &format.extension));

        if format.extension == "png" {
            if let ParsingCache::Empty = document.windows_data.parsing.cache {
                match PngData::parse(&document.binary_file) {
                    Ok(data) => {
                        document.windows_data.parsing.cache =
                            ParsingCache::Png((PngDataViewer::default(), data));
                    }
                    Err(err) => {
                        error_manager.add_error(format!("Failed to parse the PNG: {err}"));
                    }
                }
            }
            match &document.windows_data.parsing.cache {
                ParsingCache::Png(png_data) => {
                    if png_data
                        .1
                        .chunks
                        .iter()
                        .find(|chunk| chunk.chunk_type == INVALID_PNG_CHUNK)
                        .is_some()
                    {
                        document
                            .windows_data
                            .detection
                            .actions
                            .push(Action::Acropalypse(1920, 1080, true));
                    }
                }
                _ => {
                    error_manager.add_error("Document is a png but the data is not a PNG");
                }
            }
        } else if format.extension == "pdf" {
            document
                .windows_data
                .detection
                .actions
                .push(Action::PdfExtractImages);
            document
                .windows_data
                .detection
                .actions
                .push(Action::PdfExtractPage(0));
            document
                .windows_data
                .detection
                .actions
                .push(Action::PdfExtractSignature(None));
        } else if format.extension == "gif" {
            document
                .windows_data
                .detection
                .actions
                .push(Action::ExtractGif);
        }
    }
    /// show inner ui
    #[allow(clippy::too_many_lines)]
    pub(crate) fn show_detection_inner_ui(
        &mut self,
        ui: &mut egui::Ui,
        error_manager: &mut ErrorManager,
        current_idx: usize,
    ) {
        // exif
        let Some(document) = self.documents.get_mut(current_idx) else {
            return;
        };
        if document.windows_data.detection.exif_data.is_none() {
            self.detection_prepare_ui(error_manager);
        }
        self.show_detection_exif(ui, error_manager);

        // actions
        let Some(document) = self.documents.get_mut(current_idx) else {
            return;
        };
        let mut new_docs = Vec::new();
        let mut go_to_range = None;
        for one_action in &mut document.windows_data.detection.actions {
            match one_action {
                Action::Acropalypse(width, height, reconstruct_only) => {
                    ui.horizontal(|ui| {
                        ui.label("Width");
                        ui.add(egui::DragValue::new(width));
                        ui.label("Height");
                        ui.add(egui::DragValue::new(height));
                        ui.checkbox(reconstruct_only, "Reconstruct only");
                        if ui.button("Run acropalyse").clicked() {
                            match &document.windows_data.parsing.cache {
                                ParsingCache::Png(png_data) => {
                                    match do_acropalypse(
                                        &document.binary_file,
                                        &png_data.1,
                                        (*width, *height),
                                        *reconstruct_only,
                                    ) {
                                        Ok(doc) => {
                                            new_docs.push(doc);
                                        }
                                        Err(err) => {
                                            error_manager.add_error(err);
                                        }
                                    }
                                }
                                _ => {
                                    error_manager
                                        .add_error("Document is a png but the data is not a PNG");
                                }
                            }
                        }
                    });
                }
                Action::ExtractGif => {
                    if ui.button("Extract gif").clicked() {
                        match extract_gif_images(
                            document.binary_file.as_ref().as_slice(),
                            error_manager,
                        ) {
                            Ok(docs) => {
                                for one_doc in docs {
                                    new_docs.push(one_doc);
                                }
                            }
                            Err(err) => {
                                error_manager.add_error(err);
                            }
                        }
                    }
                }
                Action::PdfExtractImages => {
                    if ui.button("extract images").clicked() {
                        match pdf_image::extract_pdf_images(
                            document.binary_file.clone(),
                            error_manager,
                        ) {
                            Ok(docs) => {
                                for one_doc in docs {
                                    new_docs.push(one_doc);
                                }
                            }
                            Err(err) => {
                                error_manager.add_error(err);
                            }
                        }
                    }
                }
                Action::PdfExtractSignature(res) => {
                    if let Some(results) = &res {
                        if results.is_empty() {
                            ui.label("No ByteRange found");
                        }
                        for one_byte_range_res in results {
                            let head_text = format!("Bytes range {:?}", one_byte_range_res.range);
                            ui.collapsing(head_text, |ui| {
                                if ui.button("Show").clicked() {
                                    let start = one_byte_range_res.range[0].saturating_add(
                                        one_byte_range_res.range[1].saturating_add(1),
                                    );
                                    let end = one_byte_range_res.range[2].saturating_sub(2);
                                    go_to_range = Some(start..=end);
                                }
                                if one_byte_range_res.results.is_empty() {
                                    ui.label("No signature results");
                                }
                                for (idx, one_res) in one_byte_range_res.results.iter().enumerate()
                                {
                                    ui.collapsing(format!("Certificate {idx}"), |ui| {
                                        ui.label(format!(
                                            "Subject: {}",
                                            one_res.certificate.tbs_certificate().subject()
                                        ));
                                        ui.label(format!(
                                            "Issuer: {}",
                                            one_res.certificate.tbs_certificate().issuer()
                                        ));
                                        let validity =
                                            one_res.certificate.tbs_certificate().validity();
                                        let now = OffsetDateTime::now_utc();
                                        #[allow(clippy::cast_precision_loss)]
                                        #[allow(clippy::cast_possible_truncation)]
                                        #[allow(clippy::cast_sign_loss)]
                                        let system_time = if now.unix_timestamp_nanos() >= 0 {
                                            UNIX_EPOCH
                                                + Duration::from_nanos(
                                                    now.unix_timestamp_nanos() as u64
                                                )
                                        } else {
                                            UNIX_EPOCH
                                                - Duration::from_nanos(
                                                    (-now.unix_timestamp_nanos()) as u64,
                                                )
                                        };
                                        let Ok(current_datetime) =
                                            DateTime::from_system_time(system_time)
                                        else {
                                            ui.label("Error cannot get current time");
                                            return;
                                        };
                                        ui.horizontal(|ui| {
                                            ui.label(format!(
                                                "Not before: {} -",
                                                validity.not_before
                                            ));
                                            let is_valid = validity.not_before.to_date_time()
                                                < current_datetime;
                                            ui.colored_label(
                                                if is_valid {
                                                    Color32::GREEN
                                                } else {
                                                    Color32::RED
                                                },
                                                if is_valid { "true" } else { "false" },
                                            );
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label(format!(
                                                "Not after: {} -",
                                                validity.not_after
                                            ));
                                            let is_valid = validity.not_after.to_date_time()
                                                > current_datetime;
                                            ui.colored_label(
                                                if is_valid {
                                                    Color32::GREEN
                                                } else {
                                                    Color32::RED
                                                },
                                                if is_valid { "true" } else { "false" },
                                            );
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Is signature valid:");
                                            ui.colored_label(
                                                if one_res.is_valid {
                                                    Color32::GREEN
                                                } else {
                                                    Color32::RED
                                                },
                                                if one_res.is_valid { "true" } else { "false" },
                                            );
                                        });
                                        if ui.button("Open in a new document").clicked() {
                                            match one_res.certificate.to_der() {
                                                Ok(der) => {
                                                    let filename = PathBuf::from("certificate.pem");
                                                    new_docs.push(Document::new(der, filename));
                                                }
                                                Err(err) => {
                                                    error_manager.add_error(err.to_string());
                                                }
                                            }
                                        }
                                    });
                                }
                            });
                        }
                    } else if ui.button("Extract signatures").clicked() {
                        match extract_pdf_signatures(document.binary_file.as_slice()) {
                            Ok(parsed) => {
                                *res = Some(parsed);
                            }

                            Err(err) => {
                                error_manager.add_error(err);
                            }
                        }
                    }
                }
                Action::PdfExtractPage(page_num) => {
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(page_num));
                        if ui.button("Extract page").clicked() {
                            match extract_pdf_page(document.binary_file.clone(), *page_num) {
                                Ok(new_doc) => {
                                    new_docs.push(new_doc);
                                }
                                Err(err) => {
                                    error_manager.add_error(err);
                                }
                            }
                        }
                    });
                    if ui.button("Extract all pages").clicked() {
                        match extract_pdf_all_pages(document.binary_file.clone(), error_manager) {
                            Ok(docs) => {
                                for doc in docs {
                                    new_docs.push(doc);
                                }
                            }
                            Err(err) => {
                                error_manager.add_error(err);
                            }
                        }
                    }
                }
            }
        }
        if let Some(range) = go_to_range {
            document.go_to_range(range);
        }
        for one_doc in new_docs {
            self.documents.push(one_doc);
        }
    }

    /// show detection ui
    pub(crate) fn show_detection_ui(
        &mut self,
        ui: &mut egui::Ui,
        error_manager: &mut ErrorManager,
    ) {
        let current_idx = self.documents.get_current_index();
        let Some(document) = self.documents.get_mut(current_idx) else {
            return;
        };
        let is_open = document.windows_data.detection.is_open;
        if is_open {
            let mut is_open = is_open;
            egui::Window::new(Detection::title())
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    self.show_detection_inner_ui(ui, error_manager, current_idx);
                });
            if let Some(document) = self.documents.get_mut(current_idx) {
                document.windows_data.detection.is_open = is_open;
            }
        }
    }
}
