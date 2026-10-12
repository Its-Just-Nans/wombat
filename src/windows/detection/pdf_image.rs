//! pdf images

use hayro::hayro_interpret::font::GlyphRun;
use hayro::hayro_interpret::{
    BlendMode, ClipPath, Context, Device, DrawMode, DrawProps, Image, ImageData, ImageDrawProps,
    InterpreterCache, InterpreterSettings, SoftMask, interpret_page,
};
use hayro::kurbo::{Affine, BezPath, Rect};

use image::{DynamicImage, ImageBuffer};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;

use bladvak::{AppError, ErrorManager, log};

use crate::document::Document;

/// image extractor
struct ImageExtractor(Vec<Result<DynamicImage, String>>);

impl ImageExtractor {
    /// new extractor
    fn new() -> Self {
        Self(Vec::new())
    }
}

/// extract pdf images
pub(crate) fn extract_pdf_images(
    input: Arc<Vec<u8>>,
    error_manager: &mut ErrorManager,
) -> Result<Vec<Document>, AppError> {
    let pdf =
        hayro::hayro_syntax::Pdf::new(input).map_err(|e| format!("Failed to load PDF: {e:?}"))?;

    let settings = InterpreterSettings::default();
    let cache = InterpreterCache::new();
    // Pass dummy values for bbox and initial transform, since we don't care about those.
    let mut context = Context::new(
        Affine::IDENTITY,
        Rect::new(0.0, 0.0, 1.0, 1.0),
        &cache,
        pdf.xref(),
        settings,
    );

    let mut new_docs = Vec::new();
    for (idx_page, page) in pdf.pages().iter().enumerate() {
        let mut extractor = ImageExtractor::new();
        interpret_page(page, &mut context, &mut extractor);

        for (idx, res) in extractor.0.into_iter().enumerate() {
            log::info!("Extracting output of page {idx_page}: image {idx}");
            match res {
                Ok(img) => {
                    let mut data = vec![];
                    if let Err(err) =
                        img.write_to(&mut Cursor::new(&mut data), image::ImageFormat::Png)
                    {
                        error_manager.add_error(err.to_string());
                        continue;
                    }
                    let filename = PathBuf::from(format!("page_{idx_page}_image_{idx}.png"));
                    new_docs.push(Document::new(data, filename));
                }
                Err(err) => {
                    error_manager.add_error(format!("Error image {idx}: {err}"));
                }
            }
        }
    }
    Ok(new_docs)
}

/// Implement `Device` for `ImageExtractor`. We can ignore most operations and only
/// need to implement `draw_rgba_image` and `draw_stencil_image`.
impl Device<'_> for ImageExtractor {
    fn draw_path(&mut self, _: &BezPath, _: DrawProps<'_>, _: &DrawMode) {}

    fn push_clip_path(&mut self, _: &ClipPath) {}

    fn push_transparency_group(&mut self, _: f32, _: Option<SoftMask<'_>>, _: BlendMode) {}

    fn draw_glyph_run(&mut self, _: &GlyphRun<'_, '_>, _: DrawProps<'_>, _: &DrawMode) {}

    fn pop_clip(&mut self) {}

    fn pop_transparency_group(&mut self) {}

    fn draw_image(&mut self, image: Image<'_, '_>, _: ImageDrawProps<'_>) {
        match image {
            Image::Stencil(s) => {
                s.with_stencil(
                    |stencil, _paint| {
                        // Stencil images are gray-channel images that should be painted using the color stored in
                        // `paint`. For simplicity, we just store them as gray-channel for now.
                        let image_res = if let Some(buffer) = ImageBuffer::from_raw(
                            stencil.width,
                            stencil.height,
                            stencil.data.clone(),
                        ) {
                            Ok(DynamicImage::ImageLuma8(buffer))
                        } else {
                            Err("Cannot create stencil image".to_string())
                        };
                        self.0.push(image_res);
                    },
                    None,
                );
            }
            Image::Raster(r) => {
                // The alpha and RGB channels are provided separately.
                r.with_rgba(
                    |image, alpha| {
                        let (rgb_data, width, height) = match image {
                            ImageData::Rgb(rgb) => (rgb.data, rgb.width, rgb.height),
                            ImageData::Luma(luma) => {
                                let rgb = luma
                                    .data
                                    .iter()
                                    .flat_map(|g| [*g, *g, *g])
                                    .collect::<Vec<_>>();
                                (rgb, luma.width, luma.height)
                            }
                        };

                        let image_res = if let Some(alpha) = alpha {
                            // This is not complete, as it can in theory happen that the alpha channel has a different
                            // dimension than the RGB channel. We ignore this edge case for this example.
                            if alpha.width == width && alpha.height == height {
                                let interleaved = rgb_data
                                    .chunks(3)
                                    .zip(alpha.data)
                                    .flat_map(|(rgb, a)| [rgb[0], rgb[1], rgb[2], a])
                                    .collect::<Vec<u8>>();

                                if let Some(buffer) =
                                    ImageBuffer::from_raw(width, height, interleaved)
                                {
                                    Ok(DynamicImage::ImageRgba8(buffer))
                                } else {
                                    Err("Cannot create interleaved image".to_string())
                                }
                            } else if let Some(buffer) =
                                ImageBuffer::from_raw(width, height, rgb_data)
                            {
                                Ok(DynamicImage::ImageRgb8(buffer))
                            } else {
                                Err("Cannot create alpha image".to_string())
                            }
                        } else {
                            if let Some(buffer) = ImageBuffer::from_raw(width, height, rgb_data) {
                                Ok(DynamicImage::ImageRgb8(buffer))
                            } else {
                                Err("Cannot create alpha image".to_string())
                            }
                        };

                        self.0.push(image_res);
                    },
                    None,
                );
            }
        }
    }
}
