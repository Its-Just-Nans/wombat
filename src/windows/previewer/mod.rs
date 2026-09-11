//! Preview

use std::sync::Arc;

use bladvak::ErrorManager;
use bladvak::eframe::egui;
use file_format::Kind;

mod image;
use image::ImagePreview;

mod font;
use font::FontPreview;

/// Preview data
#[derive(Default, PartialEq, Debug)]
enum PreviewData {
    /// image data
    Image(ImagePreview),
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
        filename: &str,
        binary_file: &Arc<Vec<u8>>,
        kind: Kind,
    ) {
        if self.data == PreviewData::None {
            if kind == Kind::Image {
                self.data = match ImagePreview::prepare_ui(ui, binary_file) {
                    Ok(data) => PreviewData::Image(data),
                    Err(err) => PreviewData::Error(err),
                }
            } else if kind == Kind::Font {
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
    }

    /// ui
    pub(crate) fn ui(
        &mut self,
        ui: &mut egui::Ui,
        error_manager: &mut ErrorManager,
        fonts_definitions: &mut egui::FontDefinitions,
        filename: &str,
        binary_file: &Arc<Vec<u8>>,
        kind: Kind,
    ) {
        if self.is_open {
            let mut is_open = self.is_open;
            egui::Window::new("Previewer")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| match &mut self.data {
                    PreviewData::Image(image_preview) => {
                        image_preview.ui(ui, binary_file, error_manager);
                    }
                    PreviewData::Font(font) => {
                        font.ui(ui);
                    }
                    PreviewData::Error(err) => {
                        ui.label(err.as_str());
                    }
                    PreviewData::None => {}
                });
            self.is_open = is_open;
        }
        self.prepare_ui(ui, fonts_definitions, filename, binary_file, kind);
    }

    /// reset
    pub(crate) fn reset(&mut self) {
        self.data = PreviewData::None;
    }
}
