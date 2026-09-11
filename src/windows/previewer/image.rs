//! Image previewer

use std::sync::Arc;

use bladvak::{ErrorManager, eframe::egui, image};

/// Image preview
#[derive(PartialEq)]
pub(crate) struct ImagePreview {
    /// texture
    pub(crate) texture: egui::TextureHandle,
    /// texture size
    pub(crate) size: f32,
}

impl std::fmt::Debug for ImagePreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImagePreview")
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}

impl ImagePreview {
    /// Create the preview data
    pub(crate) fn prepare_ui(ui: &egui::Ui, binary_file: &Arc<Vec<u8>>) -> Result<Self, String> {
        #[allow(clippy::cast_precision_loss)]
        if let Ok(img) = image::load_from_memory(binary_file) {
            let img = img.to_rgba8();
            let size = [img.width() as usize, img.height() as usize];

            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &img);

            let texture = ui
                .ctx()
                .load_texture("image", color_image, egui::TextureOptions::LINEAR);
            let width = texture.size()[0] as f32;
            let available_width = ui.available_width();
            let width = if available_width < width && available_width > 0.0 {
                available_width.floor()
            } else {
                width
            };
            return Ok(Self {
                texture,
                size: width,
            });
        }
        Err("Failed to load image from memory".to_string())
    }

    /// Show the ui
    pub(crate) fn ui(
        &mut self,
        ui: &mut egui::Ui,
        binary_file: &[u8],
        error_manager: &mut ErrorManager,
    ) {
        if ui.button("Copy to clipboard").clicked() {
            match image::load_from_memory(binary_file) {
                Ok(img) => {
                    if let Err(err) = bladvak::utils::set_image_in_clipboard(
                        ui.ctx(),
                        img.width() as usize,
                        img.height() as usize,
                        img.to_rgba8().as_flat_samples().as_slice(),
                    ) {
                        error_manager.add_error(err);
                    }
                }
                Err(err) => {
                    error_manager.add_error(err.to_string());
                }
            }
        }
        let img_max_width = &mut self.size;
        let img_max = self.texture.size()[0];
        ui.horizontal(|ui| {
            ui.label("Max width: ");
            ui.add(egui::DragValue::new(img_max_width).range(0..=img_max));
        });
        ui.add(egui::Image::new(&self.texture).max_width(*img_max_width));
    }
}
