//! Preview

use std::sync::Arc;

use bladvak::eframe::egui;
use bladvak::{ErrorManager, image};
use file_format::Kind;

mod font;
use font::FontPreview;

/// Preview data
#[derive(Default, PartialEq)]
enum PreviewData {
    /// egui texture
    /// image max width
    Image(Result<(egui::TextureHandle, f32), String>),
    /// Font data
    Font(font::FontPreview),
    /// Error
    Error(String),
    /// No data
    #[default]
    None,
}

impl std::fmt::Debug for PreviewData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreviewData").finish_non_exhaustive()
    }
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
    /// preapre the ui by loading it
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
                #[allow(clippy::cast_precision_loss)]
                if let Ok(img) = image::load_from_memory(binary_file) {
                    let img = img.to_rgba8();
                    let size = [img.width() as usize, img.height() as usize];

                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &img);

                    let texture =
                        ui.ctx()
                            .load_texture("image", color_image, egui::TextureOptions::LINEAR);
                    let width = texture.size()[0] as f32;
                    let available_width = ui.available_width();
                    let width = if available_width < width && available_width > 0.0 {
                        available_width.floor()
                    } else {
                        width
                    };
                    self.data = PreviewData::Image(Ok((texture, width)));
                } else {
                    self.data =
                        PreviewData::Image(Err("Failed to load image from memory".to_string()));
                }
            } else if kind == Kind::Font {
                let font_name = filename.to_string();
                fonts_definitions.font_data.insert(
                    font_name.clone(),
                    std::sync::Arc::new(egui::FontData::from_owned((*binary_file).to_vec())),
                );
                let font_family = egui::FontFamily::Name(font_name.clone().into());
                // Register the custom family with the preview font as its only font.
                fonts_definitions
                    .families
                    .insert(font_family.clone(), vec![font_name.clone()]);

                ui.ctx().set_fonts(fonts_definitions.clone());
                let font_id = egui::FontId::new(12.0, font_family.clone());
                self.data = PreviewData::Font(FontPreview::new(font_id));
            } else {
                self.data = PreviewData::Error(format!("Cannot preview '{kind:?}' for the moment"));
            }
        }
    }
    /// ui
    pub(crate) fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
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
                    PreviewData::Image(image_res) => {
                        if kind == Kind::Image {
                            match image_res {
                                Ok(image_data) => {
                                    let texture = &image_data.0;
                                    let img_max_width = &mut image_data.1;
                                    let img_max = texture.size()[0];
                                    ui.horizontal(|ui| {
                                        ui.label("Max width: ");
                                        ui.add(
                                            egui::DragValue::new(img_max_width).range(0..=img_max),
                                        );
                                    });
                                    ui.add(egui::Image::new(texture).max_width(*img_max_width));
                                }
                                Err(e) => {
                                    ui.label(e.as_str());
                                }
                            }
                        } else {
                            ui.label("Nothing to show");
                        }
                    }
                    PreviewData::Font(font) => font.ui(ui),
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
