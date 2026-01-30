//! PNG

use bladvak::eframe::egui;
use std::ops::RangeInclusive;

/// png chunk
#[derive(Debug)]
struct PngChunk {
    /// chunk size
    size: usize,
    /// chunk type
    chunk_type: String,
    /// chunk start
    start: usize,
    /// chunk end
    end: usize,
    /// chunk crc
    crc: String,
    /// chunk crc valid
    crc_valid: bool,
}

/// png data
#[derive(Debug)]
pub(crate) struct PngData {
    /// png chunks
    chunks: Vec<PngChunk>,
    /// png signature
    signature: String,
}

/// png signature
const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

impl PngData {
    /// parse the data
    pub(crate) fn parse(binary_data: &[u8]) -> Option<Self> {
        if binary_data.len() < 8 || &binary_data[..8] != PNG_SIGNATURE {
            return None;
        }

        let signature_hex = PNG_SIGNATURE
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(" ");

        let mut png_data = Self {
            chunks: vec![],
            signature: signature_hex,
        };
        let mut offset = 8;
        while offset + 12 <= binary_data.len() {
            let chunk_start = offset;

            // Length
            let length = u32::from_be_bytes([
                binary_data[offset],
                binary_data[offset + 1],
                binary_data[offset + 2],
                binary_data[offset + 3],
            ]) as usize;

            // Bounds check
            if offset + 12 + length > binary_data.len() {
                let new_chunk = PngChunk {
                    size: 0,
                    chunk_type: "Invalid".to_string(),
                    start: chunk_start,
                    end: binary_data.len(),
                    crc: "Out of bound".to_string(),
                    crc_valid: false,
                };
                png_data.chunks.push(new_chunk);
                break;
            }

            // Chunk type
            let chunk_type_bytes = &binary_data[offset + 4..offset + 8];
            let chunk_type = std::str::from_utf8(chunk_type_bytes)
                .unwrap_or("????")
                .to_string();

            // Chunk data
            let data_start = offset + 8;
            let data_end = data_start + length;
            let chunk_data = &binary_data[data_start..data_end];

            // Stored CRC
            let stored_crc = u32::from_be_bytes([
                binary_data[data_end],
                binary_data[data_end + 1],
                binary_data[data_end + 2],
                binary_data[data_end + 3],
            ]);

            // Compute CRC over type + data
            let mut crc_input = Vec::with_capacity(4 + length);
            crc_input.extend_from_slice(chunk_type_bytes);
            crc_input.extend_from_slice(chunk_data);
            let computed_crc = crc32(&crc_input);

            let crc_ok = stored_crc == computed_crc;

            let end = data_end + 3; // inclusive

            let crc = stored_crc
                .to_be_bytes()
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect::<Vec<_>>()
                .join(" ");

            let new_chunk = PngChunk {
                size: length,
                chunk_type,
                start: chunk_start,
                end,
                crc,
                crc_valid: crc_ok,
            };
            png_data.chunks.push(new_chunk);
            offset += 12 + length;
        }
        Some(png_data)
    }
}
/// CRC32 (IEEE) implementation
fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = if crc & 1 != 0 { 0xEDB8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    !crc
}

/// show PNG chunks
pub fn show_png_chunks(
    ui: &mut egui::Ui,
    png_data: Option<&PngData>,
) -> Option<RangeInclusive<usize>> {
    let Some(png_data) = png_data else {
        ui.label("Failed to parse png");
        return None;
    };

    ui.label("PNG Chunks");

    let mut return_range = None;

    ui.horizontal(|ui| {
        ui.label(format!("png signature: {}", png_data.signature));
        if ui.button("Show").clicked() {
            let range = 0..=(PNG_SIGNATURE.len() - 1);
            return_range = Some(range);
        }
    });

    egui::Grid::new("png_chunks_table")
        .striped(true)
        .show(ui, |ui| {
            ui.label("Index");
            ui.label("Data size");
            ui.label("Chunk");
            ui.label("Start");
            ui.label("End");
            ui.label("CRC");
            ui.end_row();

            for (index, one_chunk) in png_data.chunks.iter().enumerate() {
                let PngChunk {
                    size,
                    chunk_type,
                    start,
                    end,
                    crc,
                    crc_valid,
                } = one_chunk;
                ui.label(index.to_string());
                ui.label(size.to_string());
                ui.label(chunk_type);
                ui.label(start.to_string());
                ui.label(end.to_string());
                ui.colored_label(
                    if *crc_valid {
                        egui::Color32::GREEN
                    } else {
                        egui::Color32::RED
                    },
                    crc,
                );
                if ui.button("Show").clicked() {
                    let range = *start..=*end;
                    return_range = Some(range);
                }
                ui.end_row();
            }
        });
    return_range
}
