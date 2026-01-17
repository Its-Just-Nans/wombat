//! Detection

use std::ops::RangeInclusive;

use bladvak::eframe::egui;
use bladvak::errors::ErrorManager;

use crate::panels::FileInfoData;

/// Histogram data
#[derive(Debug)]
pub(crate) struct Detection {
    /// is open
    pub(crate) is_open: bool,
}

impl Detection {
    /// New import data
    pub(crate) fn new() -> Self {
        Self { is_open: false }
    }

    /// Show the detection ui
    pub(crate) fn ui(
        &mut self,
        binary_data: &[u8],
        file_info: &FileInfoData,
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
    ) -> Option<RangeInclusive<usize>> {
        if self.is_open {
            let mut is_open = self.is_open;
            let mut ret = None;
            egui::Window::new("Detection")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    ui.label(format!(
                        "Name: {} ({}) - {}",
                        file_info.name, file_info.file_type, file_info.extension
                    ));
                    ui.separator();
                    match file_info.kind {
                        file_format::Kind::Image => {
                            if file_info.extension == "png" {
                                ret = show_png_chunks(ui, binary_data);
                            } else {
                                ui.label(format!("Image {}", file_info.extension));
                            }
                        }
                        _ => {
                            ui.label(format!("Kind: {:?}", file_info.kind));
                        }
                    }
                });
            self.is_open = is_open;
            return ret;
        }
        None
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
pub fn show_png_chunks(ui: &mut egui::Ui, binary_data: &[u8]) -> Option<RangeInclusive<usize>> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

    if binary_data.len() < 8 || &binary_data[..8] != PNG_SIGNATURE {
        ui.label("Not a PNG file");
        return None;
    }

    ui.label("PNG Chunks");

    let mut offset = 8;
    let mut index = 0;

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
                    ui.label(index.to_string());
                    ui.label("-");
                    ui.label("Invalid");
                    ui.label(chunk_start.to_string());
                    ui.label(binary_data.len().to_string());
                    ui.label("Out of bounds");
                    if ui.button("Show").clicked() {
                        return_range = Some(chunk_start..=binary_data.len());
                    }
                    ui.end_row();
                    break;
                }

                // Chunk type
                let chunk_type_bytes = &binary_data[offset + 4..offset + 8];
                let chunk_type = std::str::from_utf8(chunk_type_bytes).unwrap_or("????");

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

                let chunk_end = data_end + 3; // inclusive

                ui.label(index.to_string());
                ui.label(length.to_string());
                ui.label(chunk_type);
                ui.label(chunk_start.to_string());
                ui.label(chunk_end.to_string());
                ui.colored_label(
                    if crc_ok {
                        egui::Color32::GREEN
                    } else {
                        egui::Color32::RED
                    },
                    if crc_ok {
                        format!("{stored_crc:08X} valid")
                    } else {
                        format!("{stored_crc:08X} invalid")
                    },
                );
                if ui.button("Show").clicked() {
                    return_range = Some(chunk_start..=chunk_end);
                }
                ui.end_row();

                offset += 12 + length;
                index += 1;
            }
        });
    return_range
}
