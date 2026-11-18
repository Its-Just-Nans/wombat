//! Document

use std::{path::Path, sync::Arc};

use bladvak::{ErrorManager, eframe::egui};
use hayro::{
    PixmapSettings, RenderCache, RenderSettings,
    hayro_interpret::InterpreterSettings,
    hayro_syntax::Pdf,
    vello_cpu::{color::palette::css::WHITE, peniko::ImageAlphaType},
};

/// Document previewer setting
#[derive(PartialEq)]
pub(crate) struct DocumentPreview {
    /// texture
    pub(crate) texture: egui::TextureHandle,
    /// texture size
    pub(crate) size: f32,
    /// need reload
    pub(crate) need_reload: bool,
    /// total page
    pub(crate) total_page_len: usize,
    /// params
    pub(crate) params: RenderParams,
}

impl std::fmt::Debug for DocumentPreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImagePreview")
            .field("size", &self.size)
            .field("need_reload", &self.need_reload)
            // .field("params", &self.params)
            .finish_non_exhaustive()
    }
}

/// Render params
#[derive(PartialEq, Debug, Clone)]
pub(crate) struct RenderParams {
    /// page number
    pub(crate) page_num: usize,
    /// scale
    pub(crate) scale: f32,
}

impl Default for RenderParams {
    fn default() -> Self {
        Self {
            page_num: 0,
            scale: 1.0,
        }
    }
}

impl DocumentPreview {
    /// Create the preview data
    pub(crate) fn prepare_ui(
        ui: &egui::Ui,
        binary_file: &Arc<Vec<u8>>,
        params_opt: Option<RenderParams>,
    ) -> Result<Self, String> {
        let pdf = Pdf::new(binary_file.clone()).map_err(|_e| "Failed to load PDF")?;
        let total_page = pdf.pages().len();
        let options = params_opt.unwrap_or_default();
        let page_number = options.page_num;
        let page = pdf
            .pages()
            .get(page_number)
            .ok_or_else(|| format!("Cannot get page {page_number}"))?;
        let scale = options.scale;
        let img = hayro::render(
            page,
            &RenderCache::default(),
            &InterpreterSettings::default(),
            &RenderSettings::default(),
            &PixmapSettings {
                x_scale: scale,
                y_scale: scale,
                bg_color: WHITE,
            },
        );
        let size = [img.width() as usize, img.height() as usize];
        let data = img.take_rgba8(ImageAlphaType::Alpha);

        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &data);

        let texture = ui.ctx().load_texture(
            "document_preview",
            color_image,
            egui::TextureOptions::LINEAR,
        );
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
            total_page_len: total_page,
            need_reload: false,
            params: options,
        })
    }

    /// Show the ui
    pub(crate) fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _binary_file: &[u8],
        _filename: &Path,
        _error_manager: &mut ErrorManager,
    ) {
        // if ui.button("Open in tarsier").clicked() {
        //     bladvak::utils::open_external(
        //         binary_file.to_vec(),
        //         filename,
        //         error_manager,
        //         "https://tarsier.n4n5.dev",
        //     );
        // }
        let img_max_width = &mut self.size;
        let [width, height] = self.texture.size();

        ui.label(format!("Image: {width}x{height}"));
        ui.horizontal(|ui| {
            ui.label("Current width:");
            ui.add(egui::DragValue::new(img_max_width).range(0..=width));
        });
        ui.horizontal(|ui| {
            ui.label("Page number:");
            let max = self.total_page_len.saturating_sub(1);
            let arrow_left_enabled = self.params.page_num > 0;
            ui.add_enabled_ui(arrow_left_enabled, |ui| {
                if ui.button("◀").clicked() {
                    self.params.page_num = self.params.page_num.saturating_sub(1);
                    self.need_reload = true;
                }
            });

            if ui
                .add(egui::DragValue::new(&mut self.params.page_num).range(0..=max))
                .changed()
            {
                self.need_reload = true;
            }
            let arrow_right_enabled = self.params.page_num < max;
            ui.add_enabled_ui(arrow_right_enabled, |ui| {
                if ui.button("▶").clicked() {
                    self.params.page_num = self.params.page_num.saturating_add(1);
                    self.need_reload = true;
                }
            });
            ui.label(format!("/ {max}"));
        });
        ui.horizontal(|ui| {
            ui.label("Scale: ");
            if ui
                .add(egui::DragValue::new(&mut self.params.scale).range(0.5..=4.0))
                .changed()
            {
                self.need_reload = true;
            }
        });
        ui.add(egui::Image::new(&self.texture).max_width(*img_max_width));
    }
}
