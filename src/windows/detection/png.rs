//! PNG forensics functions

use std::{
    io::{Cursor, Read, Write},
    path::PathBuf,
};

use bladvak::{AppError, log};
use flate2::{Compression, read::DeflateDecoder, write::ZlibEncoder};
use image::{DynamicImage, ImageFormat, RgbImage, RgbaImage};

use crate::{
    document::Document,
    windows::parsing::png::{INVALID_PNG_CHUNK, PngData, crc32},
};

/// run acropalyse
#[allow(clippy::too_many_lines)]
pub(crate) fn do_acropalypse(
    binary_file: &[u8],
    png_data: &PngData,
    (orig_width, orig_height): (u32, u32),
    reconstruct_only: bool,
) -> Result<Document, AppError> {
    let Some(ihdr_chunk) = png_data.chunks.first() else {
        return Err("No IHDR chunk".into());
    };
    let ihdr_start = ihdr_chunk.start + 8 + 8;
    let Some(ihdr_data) = binary_file.get(ihdr_start..(ihdr_start + 5)) else {
        return Err("No bit depth in IHDR chunk".into());
    };
    let bit_depth = ihdr_data[0];
    let color_type = ihdr_data[1];
    let compression = ihdr_data[2];
    let filter = ihdr_data[3];
    let interlace = ihdr_data[4];
    log::info!("bit_depth: {bit_depth}");
    log::info!("color_type: {color_type}");
    log::info!("compression: {compression}");
    log::info!("filter: {filter}");
    log::info!("interlace: {interlace}");

    let Some(invalid_chunk) = png_data
        .chunks
        .iter()
        .find(|c| c.chunk_type == INVALID_PNG_CHUNK)
    else {
        return Err("No invalid chunk in the PNG".into());
    };

    // extract chunks
    let Some(raw_chunk) = binary_file.get(invalid_chunk.start..) else {
        return Err("No raw chunk in the PNG".into());
    };
    let idat_signature = b"IDAT";
    let mut idat_indices = if raw_chunk.is_empty() {
        Vec::new()
    } else {
        raw_chunk
            .windows(idat_signature.len())
            .enumerate()
            .filter_map(|(i, window)| (window == idat_signature).then_some(i))
            .collect()
    };
    if idat_indices.len() > 1 && idat_indices[0] > 4 {
        idat_indices.insert(0, 0);
    }
    let mut chunks = Vec::new();
    for (idx, start_chunk) in idat_indices.iter().enumerate() {
        let start_len = start_chunk.saturating_sub(4);
        let len = match raw_chunk.get(start_len..*start_chunk) {
            Some(v) if v.len() == 4 => {
                let Ok(bytes): Result<[u8; 4], _> = v.try_into() else {
                    return Err("Cannot get length of sub chunk".into());
                };
                u32::from_be_bytes(bytes)
            }
            _ => {
                let size = idat_indices[idx + 1] - start_chunk - idat_signature.len();
                let Ok(len) = u32::try_from(size) else {
                    return Err("Failed calculate the IDAT len to a u32".into());
                };
                len
            }
        };
        let Some(chunk_type) = raw_chunk.get(*start_chunk..(*start_chunk + 4)) else {
            return Err("Cannot get type of chunk of sub chunk".into());
        };

        // data
        let data_start = start_chunk + idat_signature.len();
        let Some(data_extracted) = raw_chunk.get(data_start..(data_start + len as usize)) else {
            return Err("Cannot get data of sub chunk".into());
        };
        let mut data = Vec::new();
        if chunk_type != idat_signature {
            data.extend_from_slice(chunk_type);
        }
        data.extend_from_slice(data_extracted);
        let len = if chunk_type == idat_signature {
            len
        } else {
            let Ok(len) = u32::try_from(data.len()) else {
                return Err("Failed convert the new IDAT len to a u32".into());
            };
            len
        };

        // crc
        let crc_start = data_start + (len as usize);
        let Some(crc) = raw_chunk.get(crc_start..(crc_start + 4)) else {
            return Err("Cannot get crc of sub chunk".into());
        };
        chunks.push((len, chunk_type, data, crc));
    }
    for c in &chunks {
        log::info!("{} {:x?} {:x?} {}", c.0, c.1, c.3, c.2.len());
    }
    let full_idat: Vec<u8> = chunks.iter().flat_map(|c| &c.2).copied().collect();

    let data_idat = &full_idat[..(full_idat.len() - 4)]; // remove adler32 checksum
    log::info!("Extracted {} bytes of idat", data_idat.len());

    // Convert input bytes into a bitstream (LSB first).
    let mut bitstream = Vec::with_capacity(data_idat.len() * 8 + 7);

    for &byte in data_idat {
        for bit in 0..8 {
            bitstream.push((byte >> bit) & 1);
        }
    }

    bitstream.extend(std::iter::repeat_n(0, 7));

    // Generate byte streams for all 8 possible bit offsets.
    let mut byte_offsets: Vec<Vec<u8>> = Vec::with_capacity(8);

    for i in 0..8 {
        let mut shifted_bytestream = Vec::new();

        let mut j = i;
        while j + 7 < bitstream.len() {
            let mut val = 0u8;

            for k in 0..8 {
                val |= bitstream[j + k] << k;
            }

            shifted_bytestream.push(val);
            j += 8;
        }

        byte_offsets.push(shifted_bytestream);
    }

    // Create a new non-final stored DEFLATE block.
    let mut prefix = Vec::with_capacity(0x8005);

    prefix.push(0x00);
    prefix.extend_from_slice(&0x8000u16.to_le_bytes());
    prefix.extend_from_slice(&(0x8000u16 ^ 0xFFFF).to_le_bytes());
    prefix.extend(std::iter::repeat_n(0u8, 0x8000));

    // Search for a viable DEFLATE parse.
    let mut decompressed_result: Option<Vec<u8>> = None;

    for i in 0..data_idat.len() {
        let truncated = &byte_offsets[i % 8][i / 8..];

        // Check for the beginning of a non-final dynamic Huffman block.
        if truncated[0] & 7 != 0b100 {
            continue;
        }

        let mut input = prefix.clone();
        input.extend_from_slice(truncated);

        let cursor = Cursor::new(input);
        let mut decoder = DeflateDecoder::new(cursor);

        let mut decompressed = Vec::new();

        if decoder.read_to_end(&mut decompressed).is_err() {
            continue;
        }

        let Ok(consumed) = usize::try_from(decoder.total_in()) else {
            return Err("Failed convert the decoder total_in into usize".into());
        };
        let cursor = decoder.into_inner();

        let unused_data = &cursor.get_ref()[consumed..];

        if unused_data.is_empty() || unused_data == [0] {
            log::info!("Found viable parse at bit offset {i}!");

            // Remove the leading padding.
            decompressed.drain(..0x8000);

            decompressed_result = Some(decompressed);
            break;
        }
    }

    let Some(decompressed) = decompressed_result else {
        return Err("Failed to find viable parse".into());
    };

    let channels = if color_type == 6 { 4 } else { 3 };
    let pixel: &[u8] = if color_type == 6 {
        &[0xFF, 0x00, 0xFF, 0xFF]
    } else {
        &[0xFF, 0x00, 0xFF]
    };
    let capacity = (1 + orig_width * channels) * orig_height;
    let mut reconstructed_idat = Vec::with_capacity(capacity as usize);

    for _ in 0..orig_height {
        reconstructed_idat.push(0x00);
        reconstructed_idat.extend(pixel.repeat(orig_width as usize));
    }
    let start = reconstructed_idat.len() - decompressed.len();

    reconstructed_idat[start..].copy_from_slice(&decompressed);

    let stream = {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::none());
        encoder.write_all(&reconstructed_idat)?;
        encoder.finish()?
    };

    let mut idat_chunk = Vec::new();
    let Ok(idat_len) = u32::try_from(stream.len()) else {
        return Err("Failed convert the IDAT len to a u32".into());
    };
    idat_chunk.extend_from_slice(&idat_len.to_be_bytes());
    idat_chunk.extend_from_slice(b"IDAT");
    idat_chunk.extend_from_slice(&stream);
    let crc = crc32(&idat_chunk[4..]);
    idat_chunk.extend_from_slice(&crc.to_be_bytes());

    let signature = b"IHDR";
    let mut new_ihdr = Vec::with_capacity(25);
    new_ihdr.write_all(&13_u32.to_be_bytes())?;
    new_ihdr.write_all(signature)?;
    new_ihdr.write_all(&orig_width.to_be_bytes())?;
    new_ihdr.write_all(&orig_height.to_be_bytes())?;
    new_ihdr.write_all(&[
        bit_depth, color_type, // color type
        0_u8,       // compression method
        filter,     // filter method
        interlace,  // interlace method
    ])?;
    let crc = crc32(&new_ihdr[4..]);
    new_ihdr.write_all(&crc.to_be_bytes())?;

    let mut data = Vec::new();
    data.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    data.extend_from_slice(&new_ihdr);
    data.extend_from_slice(&idat_chunk);
    let iend_chunk = {
        let signature = b"IEND";
        let mut chunk = Vec::new();

        chunk.extend_from_slice(&0u32.to_be_bytes());
        chunk.extend_from_slice(&signature[..]);

        chunk.extend_from_slice(&crc32(&signature[..]).to_be_bytes());
        chunk
    };
    data.extend_from_slice(&iend_chunk);

    if reconstruct_only {
        return Ok(Document::new(data, PathBuf::from("extracted.png")));
    }
    let data = parse_idat(
        &reconstructed_idat,
        orig_width as usize,
        orig_height as usize,
        bit_depth,
        color_type,
        interlace,
        false,
    )?;

    let image = match color_type {
        2 => {
            let Some(img) = RgbImage::from_raw(orig_width, orig_height, data) else {
                return Err("Cannot convert RGB from_raw".into());
            };

            DynamicImage::ImageRgb8(img)
        }
        6 => {
            // we force some transparent pixels
            let data = data
                .chunks_exact(4)
                .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 0xFF])
                .collect();
            let Some(img) = RgbaImage::from_raw(orig_width, orig_height, data) else {
                return Err("Cannot convert RGBA from_raw".into());
            };

            DynamicImage::ImageRgba8(img)
        }
        _ => return Err("Unsupported color type".into()),
    };
    let mut buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, ImageFormat::Png)
        .map_err(|e| format!("Error writing the image {e}"))?;
    let data = buffer.into_inner();
    Ok(Document::new(data, PathBuf::from("rebuild_image.png")))
}

/// Paeth predictor function used in PNG filtering.
#[allow(clippy::cast_lossless)]
fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i32 + b as i32 - c as i32;

    let pa = (p - a as i32).abs();
    let pb = (p - b as i32).abs();
    let pc = (p - c as i32).abs();

    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

/// Unfilter the scanlines of a PNG image.
#[allow(clippy::cast_possible_truncation)]
fn unfilter_scanlines(
    data: &[u8],
    width: usize,
    height: usize,
    bpp: usize,
    raise_error: bool,
) -> Result<Vec<u8>, String> {
    let scanline_length = width * bpp;
    let mut result = Vec::with_capacity(scanline_length * height);
    let mut prev_line = vec![0u8; scanline_length];
    let mut offset = 0;

    for _ in 0..height {
        if offset >= data.len() {
            return Err("Unexpected end of PNG data".to_string());
        }

        let filter_type = data[offset];
        offset += 1;

        if offset + scanline_length > data.len() {
            return Err("Incomplete PNG scanline".to_string());
        }

        let mut scanline = data[offset..offset + scanline_length].to_vec();

        offset += scanline_length;

        for i in 0..scanline_length {
            let left = if i >= bpp { scanline[i - bpp] } else { 0 };

            let up = prev_line[i];

            let upper_left = if i >= bpp { prev_line[i - bpp] } else { 0 };

            let predictor = match filter_type {
                0 => 0,                                                   // None
                1 => left,                                                // Sub
                2 => up,                                                  // Up
                3 => u16::midpoint(u16::from(left), u16::from(up)) as u8, // Average
                4 => paeth_predictor(left, up, upper_left),               // Paeth
                _ => {
                    if raise_error {
                        return Err(format!("Unknown filter type: {filter_type}"));
                    }
                    0
                }
            };

            scanline[i] = scanline[i].wrapping_add(predictor);
        }

        result.extend_from_slice(&scanline);
        prev_line = scanline;
    }

    Ok(result)
}

/// Deinterlace Adam7 interlaced PNG data.
fn deinterlace_adam7(
    data: &[u8],
    width: usize,
    height: usize,
    bpp: usize,
    raise_error: bool,
) -> Result<Vec<u8>, String> {
    // Adam7 pattern: (x_start, y_start, x_step, y_step)
    let passes = [
        (0, 0, 8, 8),
        (4, 0, 8, 8),
        (0, 4, 4, 8),
        (2, 0, 4, 4),
        (0, 2, 2, 4),
        (1, 0, 2, 2),
        (0, 1, 1, 2),
    ];

    let mut img = vec![0u8; width * height * bpp];
    let mut offset = 0;

    for (x_start, y_start, x_step, y_step) in passes {
        let pass_width = width.saturating_sub(x_start).div_ceil(x_step);

        let pass_height = height.saturating_sub(y_start).div_ceil(y_step);

        if pass_width == 0 || pass_height == 0 {
            continue;
        }

        let row_bytes = pass_width * bpp;
        let scanline_len = (row_bytes + 1) * pass_height;

        if offset + scanline_len > data.len() {
            return Err("Incomplete Adam7 pass data".to_string());
        }

        let pass_data = &data[offset..offset + scanline_len];
        offset += scanline_len;

        let unfiltered = unfilter_scanlines(pass_data, pass_width, pass_height, bpp, raise_error)?;

        let mut i = 0;

        for y in 0..pass_height {
            for x in 0..pass_width {
                let pixel_offset = ((y_start + y * y_step) * width + (x_start + x * x_step)) * bpp;

                img[pixel_offset..pixel_offset + bpp].copy_from_slice(&unfiltered[i..i + bpp]);

                i += bpp;
            }
        }
    }

    Ok(img)
}

/// Parse PNG IDAT data and return pixel values.
fn parse_idat(
    unzip_idat_data: &[u8],
    width: usize,
    height: usize,
    bit_depth: u8,
    color_type: u8,
    interlace_method: u8,
    raise_error: bool,
) -> Result<Vec<u8>, String> {
    if bit_depth != 8 {
        return Err("Only 8-bit depth is supported in this implementation".to_string());
    }

    let bpp = match color_type {
        2 => 3, // RGB
        6 => 4, // RGBA
        _ => {
            return Err(format!("Unsupported color type: {color_type}"));
        }
    };

    match interlace_method {
        0 => unfilter_scanlines(unzip_idat_data, width, height, bpp, raise_error),
        1 => deinterlace_adam7(unzip_idat_data, width, height, bpp, raise_error),
        _ => Err(format!("Unsupported interlace method: {interlace_method}")),
    }
}
