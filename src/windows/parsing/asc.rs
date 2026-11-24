//! ASC

use std::{ops::RangeInclusive, path::PathBuf};

use bladvak::{ErrorManager, eframe::egui};

use crate::{document::Document, format::FileFormat};

/// ASC PGP key
#[derive(Debug)]
pub(crate) enum AscData {
    /// unknown
    Unknown,
    /// as text
    Gpg,
    /// as raw
    GpgRaw,
}

impl AscData {
    /// parse data
    pub(crate) fn parse(binary_file: &[u8]) -> Result<Self, String> {
        let res = if binary_file.starts_with(b"-----BEGIN PGP PUBLIC KEY BLOCK-----") {
            AscData::Gpg
        } else {
            // TODO
            AscData::GpgRaw
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
            Self::Gpg => {
                ui.label("Try to parse as Base64 (without spaces)");
                let mut new_range = None;
                let mut new_doc = None;
                if ui.button("Decode").clicked() {
                    use base64::{Engine, alphabet::STANDARD, engine};
                    let engine_config = engine::GeneralPurposeConfig::new()
                        .with_decode_allow_trailing_bits(true)
                        .with_decode_padding_mode(engine::DecodePaddingMode::Indifferent);
                    let process_engine = engine::GeneralPurpose::new(&STANDARD, engine_config);
                    let Some(start) = binary_file.iter().position(|&b| b == b'\n') else {
                        error_manager.add_error("Cannot find start of key");
                        return (None, None);
                    };

                    let Some(end) = binary_file[start..].windows(2).position(|w| w == b"\n=")
                    else {
                        error_manager.add_error("Cannot find end of key");
                        return (None, None);
                    };
                    let end = end.saturating_add(start);
                    let range = start..=end;
                    new_range = Some(start..=end);
                    let Some(selection) = binary_file.get(range) else {
                        error_manager.add_error("Cannot find the key range");
                        return (new_range, None);
                    };
                    let selection = selection
                        .iter()
                        .copied()
                        .filter(|b| !b.is_ascii_whitespace())
                        .collect::<Vec<u8>>();
                    let decoded = process_engine.decode(&selection);
                    match decoded {
                        Ok(res) => {
                            let mut doc =
                                Document::new(res, PathBuf::from("selected_base64_decode.asc"));
                            doc.file_format = FileFormat::PgpPublicKeyBlock;
                            doc.windows_data.parsing.is_open = true;
                            new_doc = Some(doc);
                        }
                        Err(err) => {
                            error_manager
                                .add_error(format!("Failed to base64 decode selection: {err}"));
                        }
                    }
                }
                (new_range, new_doc)
            }
            Self::GpgRaw => {
                ui.label("TODO");
                (None, None)
            }
            Self::Unknown => {
                ui.label("Unknown ASCII file");
                (None, None)
            }
        }
    }
}
