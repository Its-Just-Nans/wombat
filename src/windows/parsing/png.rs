//! PNG

use bladvak::eframe::egui::{self, CollapsingHeader};
use std::ops::RangeInclusive;

/// png chunk
#[derive(Debug)]
pub(crate) struct PngChunk {
    /// chunk size
    size: usize,
    /// chunk type
    pub(crate) chunk_type: String,
    /// chunk start
    pub(crate) start: usize,
    /// chunk end
    pub(crate) end: usize,
    /// chunk crc
    crc: String,
    /// chunk crc valid
    crc_valid: bool,
}

/// Invalid chunk
pub(crate) const INVALID_PNG_CHUNK: &str = "Invalid";

/// png data
#[derive(Debug)]
pub(crate) struct PngData {
    /// png chunks
    pub(crate) chunks: Vec<PngChunk>,
    /// png signature
    signature: String,
}

/// png signature
const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

impl PngData {
    /// parse the data
    pub(crate) fn parse(binary_data: &[u8]) -> Result<Self, String> {
        if binary_data.len() < 8 || &binary_data[..8] != PNG_SIGNATURE {
            return Err("Not a PNG signature".to_string());
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
                    chunk_type: INVALID_PNG_CHUNK.to_string(),
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
        Ok(png_data)
    }
}

/// PNG data viewer
#[derive(Default, Debug)]
pub(crate) struct PngDataViewer {
    /// show details
    details: bool,
}

impl PngDataViewer {
    /// Show the ui
    pub(crate) fn ui(
        &mut self,
        ui: &mut egui::Ui,
        png_data: &PngData,
        binary_file: &[u8],
    ) -> Option<RangeInclusive<usize>> {
        show_png_chunks(ui, self, png_data, binary_file)
    }
}

/// CRC32 (IEEE) implementation
pub(crate) fn crc32(data: &[u8]) -> u32 {
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
#[allow(clippy::too_many_lines)]
pub fn show_png_chunks(
    ui: &mut egui::Ui,
    png_data_viewer: &mut PngDataViewer,
    png_data: &PngData,
    binary_file: &[u8],
) -> Option<RangeInclusive<usize>> {
    let mut return_range = None;

    ui.horizontal(|ui| {
        ui.label(format!("PNG signature: {}", png_data.signature));
        if ui.button("Show").clicked() {
            let range = 0..=(PNG_SIGNATURE.len() - 1);
            return_range = Some(range);
        }
    });

    ui.horizontal(|ui| {
        ui.label("PNG Chunks:");
        ui.checkbox(&mut png_data_viewer.details, "Show details");
    });
    if png_data_viewer.details {
        if let Some(rr) = show_png_chunks_details(ui, png_data, binary_file) {
            return_range = Some(rr);
        }
    } else if let Some(rr) = show_png_chunks_grid(ui, png_data) {
        return_range = Some(rr);
    }
    return_range
}

/// Show png chunks
pub(crate) fn show_png_chunks_grid(
    ui: &mut egui::Ui,
    png_data: &PngData,
) -> Option<RangeInclusive<usize>> {
    let mut return_range = None;

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

/// Show png chunks with details
pub(crate) fn show_png_chunks_details(
    ui: &mut egui::Ui,
    png_data: &PngData,
    binary_file: &[u8],
) -> Option<RangeInclusive<usize>> {
    let color_type = png_data.chunks.first().and_then(|c| {
        if c.chunk_type == "IHDR" {
            return binary_file
                .get((c.start + 8)..=(c.end - 4))
                .and_then(|c| c.get(9).copied());
        }
        None
    });
    let mut return_range = None;
    for (index, one_chunk) in png_data.chunks.iter().enumerate() {
        let PngChunk {
            size,
            chunk_type,
            start,
            end,
            crc,
            crc_valid,
        } = one_chunk;

        egui::Grid::new(format!("png_chunks_table_{index}"))
            .striped(true)
            .show(ui, |ui| {
                if index == 0 {
                    ui.label("Index");
                    ui.label("Data size");
                    ui.label("Chunk");
                    ui.label("Start");
                    ui.label("End");
                    ui.label("CRC");
                    ui.end_row();
                }
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
            });
        show_chunk_details(ui, binary_file, one_chunk, color_type);
    }
    return_range
}

/// show chunk details
#[allow(clippy::too_many_lines)]
fn show_chunk_details(
    ui: &mut egui::Ui,
    binary_file: &[u8],
    chunk: &PngChunk,
    color_type: Option<u8>,
) {
    let PngChunk {
        chunk_type,
        start,
        end,
        ..
    } = chunk;

    if chunk_type == "IHDR"
        && let Some(chunk) = binary_file.get((*start + 8)..=(*end - 4))
    {
        let width = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let height = u32::from_be_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
        let bit_depth = chunk[8];
        let color_type = chunk[9];
        let compression = chunk[10];
        let filter = chunk[11];
        let interlace = chunk[12];
        CollapsingHeader::new("Details of Image header")
            .id_salt("details_ihdr")
            .show(ui, |ui| {
                ui.label(format!("width = {width}"));
                ui.label(format!("height = {height}"));
                ui.label(format!("bit depth = {bit_depth}"));
                ui.label(format!("color type = {color_type}"));
                ui.label(format!("compression = {compression}"));
                ui.label(format!("filter = {filter}"));
                ui.label(format!("interlace = {interlace}"));
            });
    }
    if chunk_type == "tIME"
        && let Some(chunk) = binary_file.get((*start + 8)..=(*end - 4))
    {
        let year = u16::from_be_bytes([chunk[0], chunk[1]]);
        let month = chunk[2];
        let day = chunk[3];
        let hours = chunk[4];
        let minutes = chunk[5];
        let seconds = chunk[6];
        CollapsingHeader::new("Details of Image last-modification time")
            .id_salt("details_time")
            .show(ui, |ui| {
                ui.label(format!("{year}-{month}-{day} {hours}h{minutes}m{seconds}s"));
            });
    }
    if chunk_type == "pHYs"
        && let Some(chunk) = binary_file.get((*start + 8)..=(*end - 4))
    {
        CollapsingHeader::new("Details of Physical pixel dimensions")
            .id_salt("details_phys")
            .show(ui, |ui| {
                let width = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                let height = u32::from_be_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
                let unit = if chunk[8] == 0 { "Unknown" } else { "Meter" };
                ui.label(format!("Pixels per unit (X axis) = {width}"));
                ui.label(format!("Pixels per unit (Y axis) = {height}"));
                ui.label(format!("Unit = {unit}"));
            });
    }
    if chunk_type == "cHRM"
        && let Some(chunk) = binary_file.get((*start + 8)..=(*end - 4))
    {
        CollapsingHeader::new("Details of Primary chromaticities and white point")
            .id_salt("details_chrm")
            .show(ui, |ui| {
                let white_point_x = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                let white_point_y = u32::from_be_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
                let red_x = u32::from_be_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]);
                let red_y = u32::from_be_bytes([chunk[12], chunk[13], chunk[14], chunk[15]]);
                let green_x = u32::from_be_bytes([chunk[16], chunk[17], chunk[18], chunk[19]]);
                let green_y = u32::from_be_bytes([chunk[20], chunk[21], chunk[22], chunk[23]]);
                let blue_x = u32::from_be_bytes([chunk[24], chunk[25], chunk[26], chunk[27]]);
                let blue_y = u32::from_be_bytes([chunk[28], chunk[29], chunk[30], chunk[31]]);
                ui.label(format!("White Point = ({white_point_x}, {white_point_y})"));
                ui.label(format!("Red = ({red_x}, {red_y})"));
                ui.label(format!("Green = ({green_x}, {green_y})"));
                ui.label(format!("Blue = ({blue_x}, {blue_y})"));
            });
    }
    if chunk_type == "bKGD"
        && let Some(chunk) = binary_file.get((*start + 8)..=(*end - 4))
    {
        CollapsingHeader::new("Details of Background color")
            .id_salt("details_background")
            .show(ui, |ui| {
                if let Some(ct) = color_type {
                    if ct == 0 || ct == 4 {
                        let greyscale = u16::from_be_bytes([chunk[0], chunk[1]]);
                        ui.label(format!("Greyscale = {greyscale}"));
                    } else if ct == 2 || ct == 6 {
                        let red = u16::from_be_bytes([chunk[0], chunk[1]]);
                        let green = u16::from_be_bytes([chunk[2], chunk[3]]);
                        let blue = u16::from_be_bytes([chunk[4], chunk[5]]);
                        ui.label(format!("Red = {red}"));
                        ui.label(format!("Green = {green}"));
                        ui.label(format!("Blue = {blue}"));
                    } else if ct == 3 {
                        let palette_idx = chunk[0];
                        ui.label(format!("Palette index = {palette_idx}"));
                    }
                } else {
                    ui.label("No color type: cannot parse the chunk");
                }
            });
    }
    if chunk_type == "tEXt"
        && let Some(chunk) = binary_file.get((*start + 8)..=(*end - 4))
    {
        CollapsingHeader::new("Details of Textual data")
            .id_salt("details_text")
            .show(ui, |ui| {
                if let Ok(utf_str) = std::str::from_utf8(chunk) {
                    ui.label(utf_str);
                } else {
                    ui.label(format!("{chunk:?}"));
                }
            });
    }
}
