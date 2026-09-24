//! Detection

pub(crate) mod exif;
pub(crate) mod pdf;
// pub(crate) mod pdf_image;

use bladvak::ErrorManager;
use bladvak::eframe::egui;

use crate::WombatApp;
use crate::panels::FileInfoData;
use crate::windows::detection::exif::ExifData;
// use crate::windows::detection::pdf_image::pdf_image_to_png;
use crate::windows::parsing::png::PngData;

/// Possible actions
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Action {
    /// Acropalypse issue
    Acropalypse,
    /// Extract images
    PdfExtractImages,
    /// Extract page
    PdfExtractPage(u32),
}

/// Detection
#[derive(serde::Serialize, serde::Deserialize, Default, Debug)]
pub(crate) struct Detection {
    /// Is open
    pub(crate) is_open: bool,
    /// exif
    exif_data: Option<Result<ExifData, String>>,
    /// Actions
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
        }
        if format.extension == "pdf" {
            self.actions.push(Action::PdfExtractImages);
            self.actions.push(Action::PdfExtractPage(0));
        }
    }
}

impl WombatApp {
    /// show inner ui
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
        let mut actions = document.windows_data.detection.actions.clone();
        for one_action in &mut actions {
            match one_action {
                Action::Acropalypse => {
                    ui.label("Possible to acropalypse");
                }
                Action::PdfExtractImages => {
                    // if ui.button("extract images").clicked()
                    //     && let Err(err) = self.extract_pdf_images(current_idx)
                    // {
                    //     error_manager.add_error(err);
                    // }
                }
                Action::PdfExtractPage(page_num) => {
                    ui.add(egui::DragValue::new(page_num));
                    if ui.button("Extract page").clicked()
                        && let Err(err) = self.extract_pdf_page(current_idx, *page_num)
                    {
                        error_manager.add_error(err);
                    }
                    if ui.button("Extract all pages").clicked()
                        && let Err(err) = self.extract_pdf_all_pages(current_idx, error_manager)
                    {
                        error_manager.add_error(err);
                    }
                }
            }
        }
        let Some(document) = self.documents.get_mut(current_idx) else {
            return;
        };
        document.windows_data.detection.actions = actions;
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
