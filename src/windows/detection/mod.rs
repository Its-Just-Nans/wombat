//! Detection

pub(crate) mod exif;
pub(crate) mod gif;
pub(crate) mod pdf;
pub(crate) mod pdf_image;
pub(crate) mod pdf_signature;
pub(crate) mod png;

use bladvak::ErrorManager;
use bladvak::eframe::egui;

use crate::WombatApp;
use crate::format::FileFormat;
use crate::windows::detection::exif::ExifData;
use crate::windows::detection::gif::extract_gif_images;
use crate::windows::detection::pdf::{extract_pdf_all_pages, extract_pdf_page};
use crate::windows::detection::pdf_signature::PdfSignaturesData;
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
    PdfExtractSignature(Option<PdfSignaturesData>),
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
        document.windows_data.detection.exif_data = Some(ExifData::parse(
            &document.binary_file,
            document.file_format.extension(),
        ));

        if document.file_format == FileFormat::PortableNetworkGraphics {
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
        } else if document.file_format == FileFormat::PortableDocumentFormat {
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
        } else if document.file_format == FileFormat::GraphicsInterchangeFormat {
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
                        let (range, doc) = results.ui(ui, error_manager);
                        if let Some(r) = range {
                            go_to_range = Some(r);
                        }
                        if let Some(d) = doc {
                            new_docs.push(d);
                        }
                    } else if ui.button("Extract signatures").clicked() {
                        match PdfSignaturesData::try_parse(document.binary_file.as_slice()) {
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
