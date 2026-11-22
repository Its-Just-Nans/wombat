//! Preview

use std::path::Path;
use std::sync::Arc;

use bladvak::ErrorManager;
use bladvak::eframe::egui;

mod image;
use image::ImagePreview;

mod font;
use font::FontPreview;
mod document;
use document::DocumentPreview;

use crate::format::{FileFormat, FileKind};

/// Preview data
#[derive(Default, PartialEq, Debug)]
enum PreviewData {
    /// image data
    Image(ImagePreview),
    /// document data
    Document(DocumentPreview),
    /// Font data
    Font(FontPreview),
    /// Error
    Error(String),
    /// No data
    #[default]
    None,
}

/// Previewer
#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
pub(crate) struct Previewer {
    /// open
    pub(crate) is_open: bool,
    /// data
    #[serde(skip)]
    data: PreviewData,
}

impl Previewer {
    /// prepare the ui by loading it
    pub(crate) fn prepare_ui(
        &mut self,
        ui: &egui::Ui,
        fonts_definitions: &mut egui::FontDefinitions,
        filename: &Path,
        binary_file: &Arc<Vec<u8>>,
        file_format: FileFormat,
    ) {
        let kind = file_format.kind();
        if self.data == PreviewData::None {
            if kind == FileKind::Image {
                self.data = match ImagePreview::prepare_ui(ui, binary_file, None) {
                    Ok(data) => PreviewData::Image(data),
                    Err(err) => PreviewData::Error(err),
                }
            } else if kind == FileKind::Document {
                self.data = match DocumentPreview::prepare_ui(ui, binary_file, None) {
                    Ok(data) => PreviewData::Document(data),
                    Err(err) => PreviewData::Error(err),
                }
            } else if kind == FileKind::Font {
                self.data = PreviewData::Font(FontPreview::prepare_ui(
                    ui,
                    binary_file,
                    filename,
                    fonts_definitions,
                ));
            } else {
                self.data = PreviewData::Error(format!("Cannot preview '{kind:?}' for the moment"));
            }
        }
        if let PreviewData::Image(img) = &mut self.data
            && img.need_reload
        {
            self.data = match ImagePreview::prepare_ui(ui, binary_file, Some(img.params.clone())) {
                Ok(data) => PreviewData::Image(data),
                Err(err) => PreviewData::Error(err),
            }
        }
        if let PreviewData::Document(doc) = &mut self.data
            && doc.need_reload
        {
            self.data = match DocumentPreview::prepare_ui(ui, binary_file, Some(doc.params.clone()))
            {
                Ok(data) => PreviewData::Document(data),
                Err(err) => PreviewData::Error(err),
            }
        }
    }

    /// inner ui
    pub(crate) fn inner_ui(
        &mut self,
        ui: &mut egui::Ui,
        error_manager: &mut ErrorManager,
        fonts_definitions: &mut egui::FontDefinitions,
        filename: &Path,
        binary_file: &Arc<Vec<u8>>,
        file_format: FileFormat,
    ) {
        match &mut self.data {
            PreviewData::Image(image_preview) => {
                image_preview.ui(ui, binary_file, filename, error_manager);
            }
            PreviewData::Document(doc) => {
                doc.ui(ui, binary_file, filename, error_manager);
            }
            PreviewData::Font(font) => {
                font.ui(ui);
            }
            PreviewData::Error(err) => {
                ui.label(err.as_str());
                ui.button("⟳").clicked().then(|| {
                    self.data = PreviewData::None;
                });
            }
            PreviewData::None => {}
        }
        self.prepare_ui(ui, fonts_definitions, filename, binary_file, file_format);
    }
    /// ui
    pub(crate) fn window_ui(
        &mut self,
        ui: &mut egui::Ui,
        error_manager: &mut ErrorManager,
        fonts_definitions: &mut egui::FontDefinitions,
        filename: &Path,
        binary_file: &Arc<Vec<u8>>,
        file_format: FileFormat,
    ) {
        if self.is_open {
            let mut is_open = self.is_open;
            egui::Window::new("Previewer")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    self.inner_ui(
                        ui,
                        error_manager,
                        fonts_definitions,
                        filename,
                        binary_file,
                        file_format,
                    );
                });
            self.is_open = is_open;
        }
    }

    /// reset
    pub(crate) fn reset(&mut self) {
        self.data = PreviewData::None;
    }
}
