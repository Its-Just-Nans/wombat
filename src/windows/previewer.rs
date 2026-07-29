//! Preview

use std::sync::Arc;

use bladvak::eframe::egui;
use bladvak::{ErrorManager, image};
use file_format::Kind;

/// Previewer
#[derive(serde::Deserialize, serde::Serialize, Default)]
pub(crate) struct Previewer {
    /// open
    pub(crate) is_open: bool,
    /// egui texture
    #[serde(skip)]
    texture: Option<Result<egui::TextureHandle, String>>,
    /// image max width
    img_max_width: f32,
}

impl std::fmt::Debug for Previewer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Previewer")
            .field("is_open", &self.is_open)
            .finish_non_exhaustive()
    }
}

impl Previewer {
    /// ui
    pub(crate) fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
        binary_file: &Arc<Vec<u8>>,
        kind: Kind,
    ) {
        if self.texture.is_none() {
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
                    self.img_max_width = width;
                    self.texture = Some(Ok(texture));
                } else {
                    self.texture = Some(Err("Failed to load image from memory".to_string()));
                }
            } else {
                self.texture = Some(Err(format!("Cannot preview '{kind:?}' for the moment")));
            }
        }
        if self.is_open {
            let mut is_open = self.is_open;
            egui::Window::new("Previewer")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    if let Some(res_texture) = &self.texture {
                        match res_texture {
                            Ok(texture) => {
                                let img_max = texture.size()[0];
                                ui.horizontal(|ui| {
                                    ui.label("Max width: ");
                                    ui.add(
                                        egui::DragValue::new(&mut self.img_max_width)
                                            .range(0..=img_max),
                                    );
                                });
                                ui.add(egui::Image::new(texture).max_width(self.img_max_width));
                            }
                            Err(e) => {
                                ui.label(e);
                            }
                        }
                    } else {
                        ui.label("Nothing to show");
                    }
                });
            self.is_open = is_open;
        }
    }

    /// reset
    pub(crate) fn reset(&mut self) {
        self.texture = None;
    }
}
