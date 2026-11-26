//! zip

use std::{
    io::{Cursor, Read},
    ops::RangeInclusive,
    path::PathBuf,
};

use bladvak::eframe::egui::{self, Color32};
use zip::{DateTime, HasZipMetadata, System, unstable::format::compression::Compression};

use crate::{WombatApp, document::Document, windows::parsing::ParsingCache};

/// Entry type
#[derive(Debug)]
pub(crate) enum EntryType {
    /// file
    File,
    /// symlink
    Symlink,
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
}

/// zip data
#[derive(Debug)]
pub(crate) struct ZipData {
    /// files
    files: Vec<ZipFile>,
}

impl ZipData {
    /// parse zip data
    pub(crate) fn try_parse(binary_file: &[u8]) -> Result<Self, String> {
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
            let filename = match file.enclosed_name() {
                Some(p) => p,
                None => PathBuf::from("NO FILENAME"),
            };
            let entry_type = if file.is_dir() {
                EntryType::Directory
            } else if file.is_symlink() {
                EntryType::Symlink
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
            });
        }
        Ok(Self { files })
    }
}

impl WombatApp {
    /// show zip data
    pub(crate) fn parsing_ui_zip(&mut self, ui: &mut egui::Ui) -> Option<RangeInclusive<usize>> {
        let Some(document) = self.documents.get_current_doc_mut() else {
            ui.label("Failed to get document");
            return None;
        };
        let ParsingCache::Zip(data) = &document.windows_data.parsing.cache else {
            if let ParsingCache::ErrorMessage(err) = &document.windows_data.parsing.cache {
                ui.label(format!("Failed to parse zip {err}"));
                return None;
            }
            ui.label("Failed to get detection");
            return None;
        };

        let mut extracted_file = None;
        let mut go_to_range = None;
        if data.files.is_empty() {
            ui.label("This zip does not contains any file");
        }
        for (one_idx, one_file) in data.files.iter().enumerate() {
            let (go_to_range_zip, extracted_file_zip) =
                show_zip_file(ui, one_file, one_idx, &document.binary_file);
            if let Some(r) = go_to_range_zip {
                go_to_range = Some(r);
            }
            if let Some(f) = extracted_file_zip {
                extracted_file = Some(f);
            }
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

/// Show one zip file
#[allow(clippy::too_many_lines)]
fn show_zip_file(
    ui: &mut egui::Ui,
    one_file: &ZipFile,
    one_idx: usize,
    binary_file: &[u8],
) -> (Option<RangeInclusive<usize>>, Option<Document>) {
    let mut extracted_file = None;
    let mut go_to_range = None;
    egui::CollapsingHeader::new(format!(
        "{}: {} {}",
        one_file.index,
        match one_file.entry_type {
            EntryType::File => "🗋",
            EntryType::Symlink => "S",
            EntryType::Directory => "🗀",
        },
        one_file.filename.display(),
    ))
    .id_salt(format!("zip_{one_idx}"))
    .show(ui, |ui| {
        egui::Grid::new(format!("grid_{one_idx}"))
            .striped(true)
            .num_columns(3)
            .show(ui, |ui| {
                ui.label("");
                ui.horizontal(|ui| {
                    if ui.button("Local block").clicked() {
                        let start = one_file.local_header_start;
                        let end = start + (30 - 1);
                        go_to_range = Some(start..=end);
                    }
                    ui.label(format!("{}", one_file.local_header_start));
                });
                ui.horizontal(|ui| {
                    if ui.button("Central block").clicked() {
                        let start = one_file.central_header_start;
                        let end = start + (46 - 1);
                        go_to_range = Some(start..=end);
                    }
                    ui.label(format!("{}", one_file.central_header_start));
                });
                ui.end_row();
                ui.label("Signature");
                get_from_file::<4>(binary_file, one_file.local_header_start, ui, print_byte);
                get_from_file::<4>(binary_file, one_file.central_header_start, ui, print_byte);
                ui.end_row();
                ui.label("Version made by");
                ui.label("");
                get_from_file::<2>(
                    binary_file,
                    one_file.central_header_start + 4,
                    ui,
                    |ui, r| {
                        let value = u16::from_le_bytes(r);
                        let s = System::from_version_made_by(value);
                        ui.label(format!("{:?} - 0x{}", s, get_byte_repr(r)));
                    },
                );
                ui.end_row();
                ui.label("Version needed to extract");
                get_from_file::<2>(binary_file, one_file.local_header_start + 4, ui, print_byte);
                get_from_file::<2>(
                    binary_file,
                    one_file.central_header_start + 6,
                    ui,
                    print_byte,
                );
                ui.end_row();
                ui.label("General purpose bit flag");
                get_from_file::<2>(binary_file, one_file.local_header_start + 6, ui, print_byte);
                get_from_file::<2>(
                    binary_file,
                    one_file.central_header_start + 8,
                    ui,
                    print_byte,
                );
                ui.end_row();
                ui.label("Compression method");
                get_from_file::<2>(
                    binary_file,
                    one_file.local_header_start + 8,
                    ui,
                    print_compression,
                );
                get_from_file::<2>(
                    binary_file,
                    one_file.central_header_start + 10,
                    ui,
                    print_compression,
                );
                ui.end_row();
                let local_time =
                    get_bytes_from_file::<2>(binary_file, one_file.local_header_start + 10)
                        .unwrap_or([0, 0]);
                let central_time =
                    get_bytes_from_file::<2>(binary_file, one_file.central_header_start + 12)
                        .unwrap_or([0, 0]);
                let local_date =
                    get_bytes_from_file::<2>(binary_file, one_file.local_header_start + 12)
                        .unwrap_or([0, 0]);
                let central_date =
                    get_bytes_from_file::<2>(binary_file, one_file.central_header_start + 14)
                        .unwrap_or([0, 0]);
                ui.label("Last modified time");
                ui.horizontal(|ui| {
                    match DateTime::try_from_msdos(
                        u16::from_le_bytes(local_date),
                        u16::from_le_bytes(local_time),
                    ) {
                        Ok(d) => {
                            ui.label(format!(
                                "{:02}h{:02}m{:02}s",
                                d.hour(),
                                d.minute(),
                                d.second()
                            ));
                        }
                        Err(_) => {
                            ui.label("Invalid");
                        }
                    }
                    ui.label(format!("(0x{})", get_byte_repr(local_time)));
                });
                ui.horizontal(|ui| {
                    match DateTime::try_from_msdos(
                        u16::from_le_bytes(central_date),
                        u16::from_le_bytes(central_time),
                    ) {
                        Ok(d) => {
                            ui.label(format!(
                                "{:02}h{:02}m{:02}s",
                                d.hour(),
                                d.minute(),
                                d.second()
                            ));
                        }
                        Err(_) => {
                            ui.label("Invalid");
                        }
                    }
                    ui.label(format!("(0x{})", get_byte_repr(local_time)));
                });
                ui.end_row();
                ui.label("Last modified date");
                ui.horizontal(|ui| {
                    match DateTime::try_from_msdos(
                        u16::from_le_bytes(local_date),
                        u16::from_le_bytes(local_time),
                    ) {
                        Ok(d) => {
                            ui.label(format!("{}-{:02}-{:02}", d.year(), d.month(), d.day()));
                        }
                        Err(_) => {
                            ui.label("Invalid");
                        }
                    }
                    ui.label(format!("(0x{})", get_byte_repr(local_date)));
                });
                ui.horizontal(|ui| {
                    match DateTime::try_from_msdos(
                        u16::from_le_bytes(central_date),
                        u16::from_le_bytes(central_time),
                    ) {
                        Ok(d) => {
                            ui.label(format!("{}-{:02}-{:02}", d.year(), d.month(), d.day()));
                        }
                        Err(_) => {
                            ui.label("Invalid");
                        }
                    }
                    ui.label(format!("(0x{})", get_byte_repr(central_date)));
                });
                ui.end_row();
                ui.label("CRC-32");
                get_from_file::<4>(
                    binary_file,
                    one_file.local_header_start + 14,
                    ui,
                    print_byte,
                );
                get_from_file::<4>(
                    binary_file,
                    one_file.central_header_start + 16,
                    ui,
                    print_byte,
                );
                ui.end_row();
                ui.label("Compressed size");
                get_from_file::<4>(
                    binary_file,
                    one_file.local_header_start + 18,
                    ui,
                    print_size,
                );
                let central_compressed_size =
                    get_bytes_from_file(binary_file, one_file.central_header_start + 20)
                        .unwrap_or([0, 0, 0, 0]);
                print_size(ui, central_compressed_size);
                ui.end_row();
                ui.label("Uncompressed size");
                get_from_file::<4>(
                    binary_file,
                    one_file.local_header_start + 22,
                    ui,
                    print_size,
                );
                get_from_file::<4>(
                    binary_file,
                    one_file.central_header_start + 24,
                    ui,
                    print_size,
                );
                ui.end_row();
                ui.label("File name length");
                let local_file_name_len =
                    get_bytes_from_file(binary_file, one_file.local_header_start + 26)
                        .unwrap_or([0, 0]);
                print_size_2(ui, local_file_name_len);
                let central_file_name_len =
                    get_bytes_from_file(binary_file, one_file.central_header_start + 28)
                        .unwrap_or([0, 0]);
                print_size_2(ui, central_file_name_len);
                ui.end_row();

                ui.label("Extra field length");
                let local_extra_fields_len =
                    get_bytes_from_file(binary_file, one_file.local_header_start + 28)
                        .unwrap_or([0, 0]);
                print_size_2(ui, local_extra_fields_len);
                let central_extra_fields_len =
                    get_bytes_from_file(binary_file, one_file.central_header_start + 30)
                        .unwrap_or([0, 0]);
                print_size_2(ui, central_extra_fields_len);
                ui.end_row();

                ui.label("File comment length");
                ui.label("");
                let comment_len =
                    get_bytes_from_file(binary_file, one_file.central_header_start + 32)
                        .unwrap_or([0, 0]);
                print_size_2(ui, comment_len);
                ui.end_row();
                ui.label("Disk number start");
                ui.label("");
                get_from_file::<2>(
                    binary_file,
                    one_file.central_header_start + 34,
                    ui,
                    print_size_2,
                );
                ui.end_row();
                ui.label("Internal file attributes");
                ui.label("");
                get_from_file::<2>(
                    binary_file,
                    one_file.central_header_start + 36,
                    ui,
                    print_byte,
                );
                ui.end_row();
                ui.label("External file attributes");
                ui.label("");
                get_from_file::<4>(
                    binary_file,
                    one_file.central_header_start + 38,
                    ui,
                    print_byte,
                );
                ui.end_row();
                ui.label("Relative offset of local header");
                ui.label("");
                get_from_file::<4>(
                    binary_file,
                    one_file.central_header_start + 42,
                    ui,
                    |ui, r| {
                        let value = u32::from_le_bytes(r) as usize;
                        ui.colored_label(
                            if value == one_file.local_header_start {
                                Color32::GREEN
                            } else {
                                Color32::RED
                            },
                            format!("{value} (0x{})", get_byte_repr(r)),
                        );
                    },
                );
                ui.end_row();
                ui.label("File name");
                let local_file_name_len = u16::from_le_bytes(local_file_name_len);
                if local_file_name_len > 0 {
                    if ui.button("Show").clicked() {
                        let start = one_file.local_header_start + 30;
                        let end = (start + local_file_name_len as usize).saturating_sub(1);
                        go_to_range = Some(start..=end);
                    }
                } else {
                    ui.label("");
                }
                let central_file_name_len = u16::from_le_bytes(central_file_name_len);
                if central_file_name_len > 0 {
                    if ui.button("Show").clicked() {
                        let start = one_file.central_header_start + 46;
                        let end = (start + central_file_name_len as usize).saturating_sub(1);
                        go_to_range = Some(start..=end);
                    }
                } else {
                    ui.label("");
                }
                ui.end_row();
                ui.label("Extra fields");
                let local_extra_fields_len = u16::from_le_bytes(local_extra_fields_len);
                if local_extra_fields_len > 0 {
                    if ui.button("Show").clicked() {
                        let start = one_file.local_header_start + 30 + local_file_name_len as usize;
                        let end = (start + local_extra_fields_len as usize).saturating_sub(1);
                        go_to_range = Some(start..=end);
                    }
                } else {
                    ui.label("");
                }

                let central_extra_fields_len = u16::from_le_bytes(central_extra_fields_len);
                if central_extra_fields_len > 0 {
                    if ui.button("Show").clicked() {
                        let start =
                            one_file.central_header_start + 46 + central_file_name_len as usize;
                        let end = (start + central_extra_fields_len as usize).saturating_sub(1);
                        go_to_range = Some(start..=end);
                    }
                } else {
                    ui.label("");
                }

                ui.end_row();
                ui.label("Comment");
                ui.label("");
                let comment_len = u16::from_le_bytes(comment_len);
                if comment_len > 0 {
                    if ui.button("Show").clicked() {
                        let start = one_file.central_header_start
                            + 46
                            + central_file_name_len as usize
                            + central_extra_fields_len as usize;
                        let end = (start + comment_len as usize).saturating_sub(1);
                        go_to_range = Some(start..=end);
                    }
                } else {
                    ui.label("");
                }
                ui.end_row();
                ui.horizontal(|ui| {
                    if let EntryType::File = one_file.entry_type
                        && ui.button("Extract").clicked()
                    {
                        let reader = Cursor::new(binary_file);
                        let Ok(mut archive) = zip::ZipArchive::new(reader) else {
                            return;
                        };

                        if let Ok(mut file) = archive.by_index(one_file.index) {
                            let mut buffer = Vec::new();
                            if file.read_to_end(&mut buffer).is_ok() {
                                extracted_file =
                                    Some(Document::new(buffer, one_file.filename.clone()));
                            }
                        }
                    }
                    if ui.button("Show content").clicked() {
                        let size = u32::from_le_bytes(central_compressed_size);
                        let start = one_file.local_header_start
                            + 30
                            + local_file_name_len as usize
                            + local_extra_fields_len as usize;
                        let end = (start + size as usize).saturating_sub(1);
                        go_to_range = Some(start..=end);
                    }
                });
            });
    });

    (go_to_range, extracted_file)
}

/// get bytes from file
fn get_bytes_from_file<const N: usize>(file: &[u8], index: usize) -> Option<[u8; N]> {
    let bytes = file.get(index..).and_then(|s| s.get(..N))?;

    let Ok(data): Result<[u8; N], _> = bytes.try_into() else {
        return None;
    };
    Some(data)
}

/// Get from the file
fn get_from_file<const N: usize>(
    file: &[u8],
    index: usize,
    ui: &mut egui::Ui,
    fn_ui: impl FnOnce(&mut egui::Ui, [u8; N]),
) {
    let Some(data) = get_bytes_from_file(file, index) else {
        ui.label("Err");
        return;
    };

    fn_ui(ui, data);
}

/// get byte representation
fn get_byte_repr<const N: usize>(r: [u8; N]) -> String {
    r.iter().fold(String::new(), |mut acc, b| {
        use std::fmt::Write;
        write!(acc, "{b:02X}").unwrap_or_default();
        acc
    })
}

/// Print a byte
fn print_byte<const N: usize>(ui: &mut egui::Ui, r: [u8; N]) {
    ui.label(format!("0x{}", get_byte_repr(r)));
}

/// print compression
fn print_compression(ui: &mut egui::Ui, data: [u8; 2]) {
    ui.horizontal(|ui| {
        let value = u16::from_le_bytes(data);
        ui.label(format!(
            "{:?} (0x{})",
            Compression::from(value),
            get_byte_repr(data)
        ));
    });
}

/// print size
fn print_size_2(ui: &mut egui::Ui, data: [u8; 2]) {
    let value = u16::from_le_bytes(data);
    ui.label(format!("{value}"));
}

/// print size
fn print_size(ui: &mut egui::Ui, data: [u8; 4]) {
    let value = u32::from_le_bytes(data);
    ui.label(format!("{value}"));
}
