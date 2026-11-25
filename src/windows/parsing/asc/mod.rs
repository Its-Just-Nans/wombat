//! ASC

use base64::{Engine, engine::general_purpose::STANDARD};
use bladvak::{
    ErrorManager,
    eframe::egui::{self, Color32},
};
use std::{ops::RangeInclusive, path::PathBuf};

use crate::{document::Document, format::FileFormat, windows::parsing::asc::pgp::PgpRawData};

mod pgp;

/// ASC data
#[derive(Debug)]
pub(crate) enum AscData {
    /// as text
    Pgp((usize, bool, Option<String>)),
    /// as raw
    PgpRaw(PgpRawData),
}

impl AscData {
    /// parse data
    pub(crate) fn parse(binary_file: &[u8]) -> Result<Self, String> {
        let res = if binary_file.starts_with(b"-----BEGIN") {
            let Some(base_64_start) = find_base64_start(binary_file) else {
                return Err("Cannot get start of base64".to_string());
            };
            let (range, data) = decode_base64_from_asc(binary_file, base_64_start)?;
            let start_checksum = range.end().saturating_add(1);
            let checksum = binary_file.get(start_checksum..).and_then(|remaining| {
                let mut splits = remaining.split(|&b| b == b'\n');
                // skip empty line
                splits.next()?;
                let checksum_line = splits.next()?;
                checksum_line
                    .strip_prefix(b"=")
                    .filter(|crc| crc.len() == 4)
            });
            let Some(checksum) = checksum else {
                return Ok(AscData::Pgp((
                    base_64_start,
                    false,
                    Some("Checksum not found".to_string()),
                )));
            };
            let expected = STANDARD.decode(checksum).map_err(|e| e.to_string())?;

            if expected.len() != 3 {
                return Ok(AscData::Pgp((
                    base_64_start,
                    false,
                    Some("Invalid length for checksum".to_string()),
                )));
            }

            // Calculate CRC-24 on decoded PGP data.
            let crc = crc24(&data);

            #[allow(clippy::cast_possible_truncation)]
            let actual = [(crc >> 16) as u8, (crc >> 8) as u8, crc as u8];

            let is_valid = actual.as_slice() == expected.as_slice();

            return Ok(AscData::Pgp((base_64_start, is_valid, None)));
        } else {
            match PgpRawData::parse(binary_file) {
                Ok(p) => AscData::PgpRaw(p),
                Err(err) => {
                    return Err(format!("Unknown ASC {err}"));
                }
            }
        };
        Ok(res)
    }

    /// show ui
    pub(crate) fn ui(
        &self,
        ui: &mut egui::Ui,
        binary_file: &[u8],
        error_manager: &mut ErrorManager,
    ) -> (Option<RangeInclusive<usize>>, Option<Document>) {
        match self {
            Self::Pgp((start, is_valid, msg)) => {
                ui.label("Try to parse as Base64 (without spaces)");
                ui.horizontal(|ui| {
                    ui.label("Checksum: ");
                    ui.colored_label(
                        if *is_valid {
                            Color32::GREEN
                        } else {
                            Color32::RED
                        },
                        if *is_valid { "Valid" } else { "Invalid" },
                    );
                    if let Some(msg) = msg {
                        ui.label(format!("({msg})"));
                    }
                });
                if ui.button("Decode").clicked() {
                    match decode_base64_from_asc(binary_file, *start) {
                        Ok((range, res)) => {
                            let mut doc =
                                Document::new(res, PathBuf::from("selected_base64_decode.asc"));
                            doc.file_format = FileFormat::PgpPublicKeyBlock;
                            doc.windows_data.parsing.is_open = true;
                            return (Some(range), Some(doc));
                        }
                        Err(err) => {
                            error_manager.add_error(err);
                        }
                    }
                }
                (None, None)
            }
            Self::PgpRaw(data) => data.ui(ui, binary_file, error_manager),
        }
    }
}

/// decode baese 64
fn decode_base64_from_asc(
    binary_file: &[u8],
    start: usize,
) -> Result<(RangeInclusive<usize>, Vec<u8>), String> {
    use base64::{Engine, alphabet::STANDARD, engine};
    let engine_config = engine::GeneralPurposeConfig::new()
        .with_decode_allow_trailing_bits(true)
        .with_decode_padding_mode(engine::DecodePaddingMode::Indifferent);
    let process_engine = engine::GeneralPurpose::new(&STANDARD, engine_config);
    let Some(end) = binary_file[start..].windows(2).position(|w| w == b"\n=") else {
        return Err("Cannot find end of key".to_string());
    };
    let end = end.saturating_add(start).saturating_sub(1);
    let range = start..=end;
    let Some(selection) = binary_file.get(range.clone()) else {
        return Err("Cannot find the key range".to_string());
    };
    let selection = selection
        .iter()
        .copied()
        .filter(|b| !b.is_ascii_whitespace())
        .collect::<Vec<u8>>();
    let decoded = process_engine.decode(&selection);
    match decoded {
        Ok(res) => Ok((range, res)),
        Err(err) => Err(format!("Failed to base64 decode selection: {err}")),
    }
}

/// Calculate crc24
#[allow(clippy::cast_possible_truncation)]
fn crc24(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xB7_04CE;

    for &byte in data {
        crc ^= u32::from(byte) << 16;

        for _ in 0..8 {
            crc <<= 1;

            if crc & 0x100_0000 != 0 {
                crc ^= 0x186_4CFB;
            }
        }
    }

    crc & 0xFFF_FFF
}

/// Find the start of the base 64
fn find_base64_start(data: &[u8]) -> Option<usize> {
    const PREFIX: &[u8] = b"-----BEGIN ";

    // Find BEGIN marker
    let begin = data.windows(PREFIX.len()).position(|w| w == PREFIX)?;

    // Find end of BEGIN line
    let line_end = data[begin..]
        .windows(2)
        .position(|w| w == b"\r\n")
        .map(|p| (begin + p, 2))
        .or_else(|| {
            data[begin..]
                .iter()
                .position(|&b| b == b'\n')
                .map(|p| (begin + p, 1))
        })?;

    let mut pos = line_end.0 + line_end.1;

    // Skip optional headers until empty line
    loop {
        if pos >= data.len() {
            return None;
        }

        let next_line = data[pos..]
            .windows(2)
            .position(|w| w == b"\r\n")
            .map(|p| (pos + p, 2))
            .or_else(|| {
                data[pos..]
                    .iter()
                    .position(|&b| b == b'\n')
                    .map(|p| (pos + p, 1))
            })?;

        let (end, newline_len) = next_line;

        // Empty line: Base64 starts here
        if end == pos {
            return Some(pos + newline_len);
        }

        // No colon means this is probably the first Base64 line,
        // rather than an optional PEM header.
        if !data[pos..end].contains(&b':') {
            return Some(pos);
        }

        pos = end + newline_len;
    }
}
