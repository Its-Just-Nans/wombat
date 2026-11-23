//! Image previewer

use std::path::Path;

use bladvak::{
    ErrorManager,
    eframe::egui::{self, Color32},
    utils,
};

/// Image preview
pub(crate) struct ImagePreview {
    /// texture
    pub(crate) texture: egui::TextureHandle,
    /// need reload
    pub(crate) need_reload: bool,
    /// params
    pub(crate) params: RenderParams,
    /// Scene
    pub(crate) scene: (egui::Rect, Color32),
    /// Grid
    grid: Option<bladvak::utils::grid::Grid>,
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
            .field("need_reload", &self.need_reload)
            .field("params", &self.params)
            .finish_non_exhaustive()
    }
}

impl ImagePreview {
    /// Create the preview data
    pub(crate) fn prepare_ui(
        ui: &egui::Ui,
        binary_file: &[u8],
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
            .load_texture("image", color_image, egui::TextureOptions::NEAREST);
        let default_color = if ui.ctx().global_style().visuals.dark_mode {
            Color32::BLACK
        } else {
            Color32::WHITE
        };
        Ok(Self {
            texture,
            need_reload: false,
            params: params_opt.unwrap_or_default(),
            scene: (egui::Rect::ZERO, default_color),
            grid: None,
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
        let [width, height] = self.texture.size();

        ui.label(format!("Image: {width}x{height}"));
        ui.collapsing("Preview settings", |ui| {
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
            ui.horizontal(|ui| {
                ui.label("Background color");
                ui.color_edit_button_srgba(&mut self.scene.1);
            });
            let mut is_some_grid = self.grid.is_some();
            ui.checkbox(&mut is_some_grid, "Show grid");
            self.grid = if is_some_grid {
                Some(utils::grid::Grid::default())
            } else {
                None
            };
        });
        egui::Frame::NONE.fill(self.scene.1).show(ui, |ui| {
            egui::Scene::new()
                .zoom_range(0.0..=f32::INFINITY)
                .show(ui, &mut self.scene.0, |ui| {
                    if let Some(grid) = &self.grid {
                        let painter = ui.painter();
                        let bg_r: egui::Response = ui.response();
                        if bg_r.rect.is_finite() {
                            grid.draw(&bg_r.rect, painter);
                        }
                    }

                    ui.add(egui::Image::new(&self.texture));
                });
        });
    }
}
