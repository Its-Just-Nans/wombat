//! GIF

use std::io::{BufReader, Cursor};
use std::path::PathBuf;

use bladvak::{AppError, ErrorManager, log};
use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, ImageFormat};

use crate::document::Document;

/// Extract fig images
pub(crate) fn extract_gif_images(
    input: &[u8],
    error_manager: &mut ErrorManager,
) -> Result<Vec<Document>, AppError> {
    let buffer = BufReader::new(Cursor::new(input));
    let decoder = match GifDecoder::new(buffer) {
        Ok(d) => d,
        Err(err) => {
            return Err(err.to_string().into());
        }
    };
    log::info!("Gif is decoded");
    let mut docs = Vec::new();
    let decoder_iter = decoder.into_frames();
    for (idx, res_frame) in decoder_iter.enumerate() {
        match res_frame {
            Ok(frame) => {
                log::info!("New frame {idx}");
                let mut png = Vec::new();
                if let Err(err) = frame
                    .into_buffer()
                    .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
                {
                    error_manager.add_error(format!("Cannot create image {idx}: {err}"));
                }
                docs.push(Document::new(png, PathBuf::from(format!("image_{idx}"))));
            }
            Err(err) => {
                error_manager.add_error(err.to_string());
            }
        }
    }
    Ok(docs)
}
