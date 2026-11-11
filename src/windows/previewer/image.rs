//! Image previewer

use std::{path::Path, sync::Arc};

use bladvak::{ErrorManager, eframe::egui, image};

/// Image preview
#[derive(PartialEq)]
pub(crate) struct ImagePreview {
    /// texture
    pub(crate) texture: egui::TextureHandle,
    /// texture size
    pub(crate) size: f32,
    /// need reload
    pub(crate) need_reload: bool,
    /// params
    pub(crate) params: RenderParams,
}

/// Render params
#[derive(PartialEq, Debug, Default, Clone)]
pub(crate) struct RenderParams {
    /// transparency
    pub(crate) transparency: Option<u8>,
}

impl std::fmt::Debug for ImagePreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImagePreview")
            .field("size", &self.size)
            .field("params", &self.params)
            .finish_non_exhaustive()
    }
}

impl ImagePreview {
    /// Create the preview data
    pub(crate) fn prepare_ui(
        ui: &egui::Ui,
        binary_file: &Arc<Vec<u8>>,
        params_opt: Option<RenderParams>,
    ) -> Result<Self, String> {
        let img = match image::load_from_memory(binary_file) {
            Ok(img) => img,
            Err(err) => return Err(format!("Failed to load image from memory: {err}")),
        };
        let mut img = img.to_rgba8();
        let size = [img.width() as usize, img.height() as usize];

        if let Some(params) = &params_opt
            && let Some(trans) = params.transparency
        {
            img.pixels_mut().for_each(|p| {
                if p.0[3] == 0 {
                    p.0[3] = trans;
                }
            });
        }

        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &img);

        let texture = ui
            .ctx()
            .load_texture("image", color_image, egui::TextureOptions::LINEAR);
        #[allow(clippy::cast_precision_loss)]
        let width = texture.size()[0] as f32;
        let available_width = ui.available_width();
        let width = if available_width < width && available_width > 0.0 {
            available_width.floor()
        } else {
            width
        };
        Ok(Self {
            texture,
            size: width,
            need_reload: false,
            params: params_opt.unwrap_or_default(),
        })
    }

    /// Show the ui
    pub(crate) fn ui(
        &mut self,
        ui: &mut egui::Ui,
        binary_file: &[u8],
        filename: &Path,
        error_manager: &mut ErrorManager,
    ) {
        if ui.button("Open in tarsier").clicked() {
            bladvak::utils::open_external(
                binary_file.to_vec(),
                filename,
                error_manager,
                "https://tarsier.n4n5.dev",
            );
        }
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
        let [width, height] = self.texture.size();

        ui.label(format!("Image: {width}x{height}"));
        ui.horizontal(|ui| {
            ui.label("Current width:");
            ui.add(egui::DragValue::new(img_max_width).range(0..=width));
        });
        ui.horizontal(|ui| {
            ui.label("Force transparency:");
            if let Some(t) = &mut self.params.transparency {
                if ui.add(egui::DragValue::new(t).range(0..=255)).changed() {
                    self.need_reload = true;
                }
            } else {
                let mut v = 0;
                if ui
                    .add(egui::DragValue::new(&mut v).range(0..=255))
                    .changed()
                {
                    self.need_reload = true;
                    self.params.transparency = Some(v);
                }
            }
        });
        ui.add(egui::Image::new(&self.texture).max_width(*img_max_width));
    }
}
