//! zip

use std::{io::Cursor, ops::RangeInclusive, path::PathBuf};

use bladvak::eframe::egui;

/// Zip file
#[derive(Debug)]
pub(crate) struct ZipFile {
    /// file type
    pub(crate) file_type: String,
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
    pub(crate) fn parse(binary_file: &[u8]) -> Option<ZipData> {
        let reader = Cursor::new(binary_file);
        let mut archive = zip::ZipArchive::new(reader).ok()?;

        let mut files = Vec::with_capacity(archive.len());

        for i in 0..archive.len() {
            let file = match archive.by_index(i) {
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
            let file_type = if file.is_dir() {
                "directory".to_string()
            } else {
                "file".to_string()
            };
            files.push(ZipFile {
                file_type,
                filename,
                compression,
                uncompressed_size,
                compressed_size,
                unix_mode,
                last_modified,
                comment,
            });
        }
        Some(Self { files })
    }
}

/// show zip data
pub(crate) fn show_zip_data(
    ui: &mut egui::Ui,
    data: Option<&ZipData>,
) -> Option<RangeInclusive<usize>> {
    let Some(data) = data else {
        ui.label("Failed to parse zip data");
        return None;
    };
    egui::Grid::new("png_chunks_table")
        .striped(true)
        .show(ui, |ui| {
            ui.label("Type");
            ui.label("Filename");
            ui.label("Compression");
            ui.label("Uncompressed size");
            ui.label("Compressed size");
            ui.label("Unix mode");
            ui.label("Last modified date");
            ui.label("Comment");
            ui.end_row();

            for one_file in &data.files {
                ui.label(one_file.file_type.clone());
                ui.label(format!("{}", one_file.filename.display()));
                ui.label(&one_file.compression);
                ui.label(one_file.uncompressed_size.to_string());
                ui.label(one_file.compressed_size.to_string());
                if let Some(mode) = one_file.unix_mode {
                    ui.label(format!("0o{mode:o}"));
                } else {
                    ui.label("No UNIX mode");
                }
                if let Some(lm) = &one_file.last_modified {
                    ui.label(lm);
                } else {
                    ui.label("No modified date");
                }
                ui.label(one_file.comment.clone());
                ui.end_row();
            }
        });
    None
}
