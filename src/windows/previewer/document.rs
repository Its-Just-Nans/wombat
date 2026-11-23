//! Document

use std::{path::Path, sync::Arc};

use bladvak::{ErrorManager, eframe::egui};
use hayro::{
    PixmapSettings, RenderCache, RenderSettings,
    hayro_interpret::InterpreterSettings,
    hayro_syntax::{DecryptionError, LoadPdfError, Pdf},
    vello_cpu::{color::palette::css::WHITE, peniko::ImageAlphaType},
};

/// Document previewer
#[derive(Debug)]
pub(crate) enum DocumentPreview {
    /// Need a reload
    NeedReload((String, HayroRenderParams)),
    /// is ready
    Pdf(PdfPreviewData),
}

/// Pdf previewer data
pub(crate) struct PdfPreviewData {
    /// texture
    pub(crate) texture: egui::TextureHandle,
    /// texture size
    pub(crate) size: f32,
    /// total page
    pub(crate) total_page_len: usize,
    /// params
    pub(crate) params: HayroRenderParams,
}

impl std::fmt::Debug for PdfPreviewData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PdfPreviewData")
            .field("size", &self.size)
            .field("params", &self.params)
            .finish_non_exhaustive()
    }
}

/// Render params
#[derive(Debug, Clone)]
pub(crate) struct HayroRenderParams {
    /// page number
    pub(crate) page_num: usize,
    /// scale
    pub(crate) scale: f32,
    /// password
    pub(crate) password: Option<String>,
    /// need reload
    pub(crate) need_reload: bool,
}

impl Default for HayroRenderParams {
    fn default() -> Self {
        Self {
            page_num: 1,
            scale: 1.0,
            password: None,
            need_reload: false,
        }
    }
}

impl DocumentPreview {
    /// Create the preview data
    pub(crate) fn prepare_ui(
        ui: &egui::Ui,
        binary_file: &Arc<Vec<u8>>,
        params_opt: Option<HayroRenderParams>,
    ) -> Result<Self, String> {
        let pdf_res = if let Some(params) = &params_opt
            && let Some(pass) = &params.password
        {
            Pdf::new_with_password(binary_file.clone(), pass)
        } else {
            Pdf::new(binary_file.clone())
        };
        let pdf = match pdf_res {
            Ok(p) => p,
            Err(err) => match err {
                LoadPdfError::Decryption(err) => match err {
                    DecryptionError::PasswordProtected => {
                        let new_params = params_opt.unwrap_or_else(|| HayroRenderParams {
                            password: Some(String::new()),
                            ..Default::default()
                        });
                        return Ok(Self::NeedReload((
                            "Failed to load PDF: invalid password".to_string(),
                            new_params,
                        )));
                    }
                    err => {
                        return Err(format!("Failed to load PDF: {err:?}"));
                    }
                },
                LoadPdfError::Invalid => {
                    return Err("Failed to load PDF: invalid".to_string());
                }
            },
        };
        let total_page = pdf.pages().len();
        let options = {
            let mut opt = params_opt.unwrap_or_default();
            opt.need_reload = false;
            opt
        };
        let page_number = options.page_num.saturating_sub(1);
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
        Ok(Self::Pdf(PdfPreviewData {
            texture,
            size: width,
            total_page_len: total_page,
            params: options,
        }))
    }

    /// Show the ui
    pub(crate) fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _binary_file: &[u8],
        _filename: &Path,
        _error_manager: &mut ErrorManager,
    ) {
        match self {
            Self::Pdf(doc) => {
                // if ui.button("Open in tarsier").clicked() {
                //     bladvak::utils::open_external(
                //         binary_file.to_vec(),
                //         filename,
                //         error_manager,
                //         "https://tarsier.n4n5.dev",
                //     );
                // }
                let img_max_width = &mut doc.size;
                let [width, height] = doc.texture.size();

                ui.label(format!("Image: {width}x{height}"));
                ui.horizontal(|ui| {
                    ui.label("Current width:");
                    ui.add(egui::DragValue::new(img_max_width).range(0..=width));
                });
                ui.horizontal(|ui| {
                    ui.label("Page number:");
                    let max = doc.total_page_len;
                    let arrow_left_enabled = doc.params.page_num > 1;
                    ui.add_enabled_ui(arrow_left_enabled, |ui| {
                        if ui.button("◀").clicked() {
                            doc.params.page_num = doc.params.page_num.saturating_sub(1);
                            doc.params.need_reload = true;
                        }
                    });

                    if ui
                        .add(egui::DragValue::new(&mut doc.params.page_num).range(1..=max))
                        .changed()
                    {
                        doc.params.need_reload = true;
                    }
                    let arrow_right_enabled = doc.params.page_num < max;
                    ui.add_enabled_ui(arrow_right_enabled, |ui| {
                        if ui.button("▶").clicked() {
                            doc.params.page_num = doc.params.page_num.saturating_add(1);
                            doc.params.need_reload = true;
                        }
                    });
                    ui.label(format!("/ {max}"));
                });
                ui.horizontal(|ui| {
                    ui.label("Scale: ");
                    if ui
                        .add(egui::DragValue::new(&mut doc.params.scale).range(0.5..=4.0))
                        .changed()
                    {
                        doc.params.need_reload = true;
                    }
                });
                ui.add(egui::Image::new(&doc.texture).max_width(*img_max_width));
            }
            Self::NeedReload((err, params)) => {
                params.need_reload = false;
                ui.label(&*err);
                if let Some(pass) = &mut params.password {
                    ui.text_edit_singleline(pass);
                    if ui.button("Load").clicked() {
                        params.need_reload = true;
                    }
                }
            }
        }
    }

    /// Get params
    pub(crate) fn params(&self) -> &HayroRenderParams {
        match self {
            Self::Pdf(doc) => &doc.params,
            Self::NeedReload((_e, params)) => params,
        }
    }
}
