//! Detection

pub(crate) mod exif;
pub(crate) mod gif;
pub(crate) mod pdf;
pub(crate) mod pdf_signature;
// pub(crate) mod pdf_image;

use std::path::PathBuf;
use std::time::SystemTime;

use bladvak::ErrorManager;
use bladvak::eframe::egui::{self, Color32};
use der::{DateTime, Encode};

use crate::WombatApp;
use crate::document::Document;
use crate::panels::FileInfoData;
use crate::windows::detection::exif::ExifData;
use crate::windows::detection::gif::extract_gif_images;
use crate::windows::detection::pdf::{extract_pdf_all_pages, extract_pdf_page};
use crate::windows::detection::pdf_signature::{ByteRangeResult, extract_pdf_signatures};
// use crate::windows::detection::pdf_image::pdf_image_to_png;
use crate::windows::parsing::png::PngData;

/// Possible actions
#[derive(Debug)]
pub enum Action {
    /// Acropalypse issue
    Acropalypse,
    /// Extract images
    PdfExtractImages,
    /// Extract page
    PdfExtractPage(u32),
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

    /// prepare ui
    pub(crate) fn prepare_ui(&mut self, binary_file: &[u8], format: &FileInfoData) {
        self.exif_data = Some(ExifData::parse(binary_file, &format.extension));
        if format.extension == "png" {
            let parsed_data = PngData::parse(binary_file);
            match parsed_data {
                Ok(png_data) => {
                    if png_data
                        .chunks
                        .iter()
                        .find(|chunk| chunk.chunk_type == "Invalid")
                        .is_some()
                    {
                        self.actions.push(Action::Acropalypse);
                    }
                }
                Err(_err) => {}
            }
        } else if format.extension == "pdf" {
            self.actions.push(Action::PdfExtractImages);
            self.actions.push(Action::PdfExtractPage(0));
            self.actions.push(Action::PdfExtractSignature(None));
        } else if format.extension == "gif" {
            self.actions.push(Action::ExtractGif);
        }
    }
}

impl WombatApp {
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
        let file_info_data = document.get_file_format().clone();
        let detection = &mut document.windows_data.detection;
        if detection.exif_data.is_none() {
            detection.prepare_ui(&document.binary_file, &file_info_data);
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
                Action::Acropalypse => {
                    ui.label("Possible to acropalypse");
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
                    // if ui.button("extract images").clicked()
                    //     && let Err(err) = self.extract_pdf_images(current_idx)
                    // {
                    //     error_manager.add_error(err);
                    // }
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
                                        let Ok(current_datetime) =
                                            DateTime::from_system_time(SystemTime::now())
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
                    ui.add(egui::DragValue::new(page_num));
                    if ui.button("Extract page").clicked() {
                        match extract_pdf_page(document.binary_file.as_slice(), *page_num) {
                            Ok(new_doc) => {
                                new_docs.push(new_doc);
                            }
                            Err(err) => {
                                error_manager.add_error(err);
                            }
                        }
                    }
                    if ui.button("Extract all pages").clicked() {
                        match extract_pdf_all_pages(document.binary_file.as_slice(), error_manager)
                        {
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
