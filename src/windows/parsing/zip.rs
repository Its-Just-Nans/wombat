//! zip

use std::{
    io::{Cursor, Read},
    ops::RangeInclusive,
    path::PathBuf,
};

use bladvak::eframe::egui;
use zip::HasZipMetadata;

use crate::{WombatApp, document::Document, windows::parsing::ParsingCache};

/// Entry type
#[derive(Debug)]
pub(crate) enum EntryType {
    /// filej
    File,
    /// dir
    Directory,
}

/// Zip file
#[derive(Debug)]
pub(crate) struct ZipFile {
    /// index
    pub(crate) index: usize,
    /// local header start
    pub(crate) local_header_start: usize,
    /// central header start
    pub(crate) central_header_start: usize,
    /// file type
    pub(crate) entry_type: EntryType,
    /// filename
    pub(crate) filename: PathBuf,
    /// compression
    pub(crate) compression: String,
    /// uncompressed size
    pub(crate) uncompressed_size: u64,
    /// compressed size
    pub(crate) compressed_size: u64,
    /// unix mode
    pub(crate) unix_mode: Option<u32>,
    /// last modified date
    pub(crate) last_modified: Option<String>,
    /// comment
    pub(crate) comment: String,
}

/// zip data
#[derive(Debug)]
pub(crate) struct ZipData {
    /// files
    files: Vec<ZipFile>,
}

impl ZipData {
    /// parse zip data
    pub(crate) fn parse(binary_file: &[u8]) -> Result<Self, String> {
        let reader = Cursor::new(binary_file);
        let mut archive = match zip::ZipArchive::new(reader) {
            Ok(ar) => ar,
            Err(e) => {
                return Err(format!("Failed to parse zip: {e}"));
            }
        };

        let mut files = Vec::with_capacity(archive.len());

        for index in 0..archive.len() {
            let file = match archive.by_index(index) {
                Ok(file) => file,
                Err(_err) => {
                    continue;
                }
            };
            let Some(filename) = file.enclosed_name() else {
                continue;
            };
            let uncompressed_size = file.size();
            let compressed_size = file.compressed_size();
            let compression = file.compression().to_string();
            let unix_mode = file.unix_mode();
            let last_modified = file.last_modified().map(|lm| lm.to_string());

            let comment = file.comment().to_string();
            let entry_type = if file.is_dir() {
                EntryType::Directory
            } else {
                EntryType::File
            };
            #[allow(clippy::cast_possible_truncation)]
            let local_header_start = file.get_metadata().header_start as usize;
            #[allow(clippy::cast_possible_truncation)]
            let central_header_start = file.get_metadata().central_header_start as usize;
            files.push(ZipFile {
                index,
                local_header_start,
                central_header_start,
                entry_type,
                filename,
                compression,
                uncompressed_size,
                compressed_size,
                unix_mode,
                last_modified,
                comment,
            });
        }
        Ok(Self { files })
    }
}

impl WombatApp {
    /// show zip data
    pub(crate) fn parsing_ui_zip(&mut self, ui: &mut egui::Ui) -> Option<RangeInclusive<usize>> {
        let mut extracted_file = None;
        let Some(document) = self.documents.get_current_doc_mut() else {
            ui.label("Failed to get document");
            return None;
        };
        let ParsingCache::Zip(cached_data) = &document.windows_data.parsing.cache else {
            ui.label("Failed to get detection");
            return None;
        };
        let data = match cached_data {
            Ok(data) => data,
            Err(e) => {
                ui.label("Failed to parse zip data");
                ui.label(e);
                return None;
            }
        };
        let mut go_to_range = None;
        for (one_idx, one_file) in data.files.iter().enumerate() {
            egui::CollapsingHeader::new(format!(
                "{}: {} {}",
                one_file.index,
                if let EntryType::File = one_file.entry_type {
                    "🗋"
                } else {
                    "🗀"
                },
                one_file.filename.display(),
            ))
            .id_salt(format!("zip_{one_idx}"))
            .show(ui, |ui| {
                egui::Grid::new(format!("grid_{one_idx}"))
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Type");
                        ui.label(format!("{:?}", one_file.entry_type));
                        ui.end_row();
                        ui.label("Compression");
                        ui.label(&one_file.compression);
                        ui.end_row();
                        ui.label("Uncompressed size");
                        ui.label(one_file.uncompressed_size.to_string());
                        ui.end_row();
                        ui.label("Compressed size");
                        ui.label(one_file.compressed_size.to_string());
                        ui.end_row();
                        ui.label("Unix mode");
                        if let Some(mode) = one_file.unix_mode {
                            ui.label(format!("0o{mode:o}"));
                        } else {
                            ui.label("No UNIX mode");
                        }
                        ui.end_row();
                        ui.label("Last modified date");
                        if let Some(lm) = &one_file.last_modified {
                            ui.label(lm);
                        } else {
                            ui.label("No modified date");
                        }
                        ui.end_row();
                        ui.label("Comment");
                        ui.label(one_file.comment.clone());
                        ui.end_row();
                    });
                if ui.button("Local block").clicked() {
                    let start = one_file.local_header_start;
                    let end = start + (30 - 1);
                    go_to_range = Some(start..=end);
                }
                if ui.button("Central block").clicked() {
                    let start = one_file.central_header_start;
                    let end = start + (46 - 1);
                    go_to_range = Some(start..=end);
                }
                if let EntryType::File = one_file.entry_type
                    && ui.button("Extract").clicked()
                {
                    let reader = Cursor::new(&*document.binary_file);
                    let Ok(mut archive) = zip::ZipArchive::new(reader) else {
                        return;
                    };

                    if let Ok(mut file) = archive.by_index(one_file.index) {
                        let mut buffer = Vec::new();
                        if file.read_to_end(&mut buffer).is_ok() {
                            extracted_file = Some(Document::new(buffer, one_file.filename.clone()));
                        }
                    }
                }
            });
        }
        if let Some(range) = go_to_range {
            document.go_to_range(range);
        }
        if let Some(extracted) = extracted_file {
            self.documents.push(extracted);
        }
        None
    }
}
