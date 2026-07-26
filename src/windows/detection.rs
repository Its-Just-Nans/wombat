//! Detection

use std::io::Cursor;
use std::ops::RangeInclusive;
use std::path::PathBuf;

use bladvak::ErrorManager;
use bladvak::eframe::egui;
use bladvak::egui_extras::{Column, TableBuilder};
use exif::Exif;

use crate::WombatApp;
use crate::document::Document;
use crate::windows::parsing::jpg::{Marker, parse_jpeg};
use crate::windows::parsing::png::PngData;

/// Exif data
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct ExifData {
    /// exif
    #[serde(skip)]
    exif: Option<Exif>,
    /// geo
    geo: Option<Result<String, ()>>,
    /// thumbnail,
    thumbnail: Option<Result<(usize, usize), String>>,
}

impl std::fmt::Debug for ExifData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExifData")
            .field("geo", &self.geo)
            .field("thumbnail", &self.thumbnail)
            .finish_non_exhaustive()
    }
}

/// Offset JPG of the exif = 2 bytes (marker) + 2 bytes (length) + 6 byes ("Exif\0\0")
const OFFSET_EXIF_JPG: usize = 2 + 2 + 6;

/// Offset PNG of the exif
const OFFSET_EXIF_PNG: usize = 4 + 4;

/// Detection
#[derive(serde::Serialize, serde::Deserialize, Default, Debug)]
pub(crate) struct Detection {
    /// Is open
    pub(crate) is_open: bool,
    /// exif
    exif_data: Option<Result<ExifData, String>>,
}

impl Detection {
    /// reset
    pub(crate) fn reset(&mut self) {
        self.exif_data = None;
    }

    /// parse exif
    pub(crate) fn parse_exif(&mut self, binary_file: &[u8], file_extension: &str) {
        let cursor = Cursor::new(binary_file);
        let mut bufreader = std::io::BufReader::new(cursor);
        let parsed_exif = exif::Reader::new().read_from_container(&mut bufreader);
        match parsed_exif {
            Ok(exif) => {
                let geo = if let Some(lat) =
                    exif.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY)
                    && let Some(lat_ref) =
                        exif.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY)
                    && let Some(lon) = exif.get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY)
                    && let Some(lon_ref) =
                        exif.get_field(exif::Tag::GPSLongitudeRef, exif::In::PRIMARY)
                {
                    let north = matches!(&lat_ref.value, exif::Value::Ascii(v) if v[0] == b"N");
                    let east = matches!(&lon_ref.value, exif::Value::Ascii(v) if v[0] == b"E");
                    if let Some(latitude) = dms_to_decimal(lat, north)
                        && let Some(longitude) = dms_to_decimal(lon, east)
                    {
                        let geo_url = format!("geo:{latitude:.8},{longitude:.8}");
                        Some(Ok(geo_url))
                    } else {
                        Some(Err(()))
                    }
                } else {
                    None
                };
                let thumbnail = extract_thumbnail_position(&exif, binary_file, file_extension);
                self.exif_data = Some(Ok(ExifData {
                    exif: Some(exif),
                    geo,
                    thumbnail,
                }));
            }
            Err(e) => {
                self.exif_data = Some(Err(format!("Failed to parse exif: {e}")));
            }
        }
    }
}

/// Extract the thumbnail position
fn extract_thumbnail_position(
    exif: &Exif,
    binary_file: &[u8],
    file_extension: &str,
) -> Option<Result<(usize, usize), String>> {
    if let Some(jpeg_interchange_format) =
        exif.get_field(exif::Tag::JPEGInterchangeFormat, exif::In::THUMBNAIL)
        && let Some(jpeg_interchange_format_length) =
            exif.get_field(exif::Tag::JPEGInterchangeFormatLength, exif::In::THUMBNAIL)
    {
        if let exif::Value::Long(offset) = &jpeg_interchange_format.value
            && let Some(offset) = offset.first()
            && let exif::Value::Long(length) = &jpeg_interchange_format_length.value
            && let Some(length) = length.first()
        {
            if file_extension == "jpg" || file_extension == "jpeg" {
                let jpeg_size = offset + length;
                let parsed_data = parse_jpeg(binary_file);
                if let Ok(data) = parsed_data {
                    if let Some(seg_exif) = data.iter().find(|seg| {
                        if let Marker::APP(_i) = seg.marker
                            && let Some(seg_part) =
                                binary_file.get(seg.start..=seg.start + OFFSET_EXIF_JPG)
                            && seg_part[4..=9] == *b"Exif\0\0"
                        {
                            true
                        } else {
                            false
                        }
                    }) {
                        let offset_exif = seg_exif.start + OFFSET_EXIF_JPG;
                        let start = offset_exif + *offset as usize;
                        let end = offset_exif + jpeg_size as usize - 1;
                        let range_res = (start, end);
                        Some(Ok(range_res))
                    } else {
                        Some(Err("No exif segment found in file".to_string()))
                    }
                } else {
                    Some(Err(
                        "Failed to parse jpg: cannot get exif segment".to_string()
                    ))
                }
            } else if file_extension == "png" {
                let png_size = offset + length;
                let parsed_data = PngData::parse(binary_file);
                if let Some(png_data) = parsed_data {
                    if let Some(chunk) = png_data
                        .chunks
                        .iter()
                        .find(|chunk| chunk.chunk_type == "eXIf")
                    {
                        let offset_exif = chunk.start + OFFSET_EXIF_PNG;
                        let start = offset_exif + *offset as usize;
                        let end = offset_exif + png_size as usize - 1;
                        let range_res = (start, end);
                        Some(Ok(range_res))
                    } else {
                        Some(Err("No exif chunk found in file".to_string()))
                    }
                } else {
                    Some(Err(
                        "Failed to parse png: cannot get exif segment".to_string()
                    ))
                }
            } else {
                Some(Err(format!(
                    "Cannot get exif chunk of file type: {file_extension}"
                )))
            }
        } else {
            Some(Err("Failed to get Thumbnail".to_string()))
        }
    } else {
        None
    }
}

/// show exif table
fn show_exif_table(ui: &mut egui::Ui, exif: &Exif) {
    TableBuilder::new(ui)
        .max_scroll_height(100.0)
        .striped(true)
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::remainder())
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.label("Exif tag");
            });
            header.col(|ui| {
                ui.label("IFD idx");
            });
            header.col(|ui| {
                ui.label("exif value");
            });
        })
        .body(|mut body| {
            for field in exif.fields() {
                body.row(30.0, |mut row| {
                    row.col(|ui| {
                        ui.label(format!("{}", field.tag));
                    });
                    row.col(|ui| {
                        ui.label(format!("{}", field.ifd_num));
                    });
                    row.col(|ui| {
                        ui.label(format!("{}", field.display_value().with_unit(exif)));
                    });
                });
            }
        });
}

impl WombatApp {
    /// show exif
    fn show_detection_exif(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        let Some(document) = self.documents.get_current_doc_mut() else {
            return;
        };
        let exif_data = &document.windows_data.detection.exif_data;
        let mut doc = None;
        let mut range = None;
        if let Some(exif_data) = exif_data {
            match exif_data {
                Ok(exif_data) => {
                    let Some(exif) = &exif_data.exif else {
                        return;
                    };
                    ui.collapsing("Exif info", |ui| {
                        show_exif_table(ui, exif);
                        if let Some(geo) = &exif_data.geo {
                            if let Ok(geo_url) = geo {
                                ui.horizontal(|ui| {
                                    ui.label("GPS detected: ");
                                    ui.add(egui::Hyperlink::new(geo_url).open_in_new_tab(true));
                                });
                            } else {
                                ui.label("Failed to parse GPS data");
                            }
                        } else {
                            ui.label("No GPS data found");
                        }
                        if let Some(thumbnail) = &exif_data.thumbnail {
                            match thumbnail {
                                Ok(thumb) => {
                                    ui.label(format!("Thumbnail found at {} {}", thumb.0, thumb.1));
                                    range = if ui.button("Show thumbnail range").clicked() {
                                        Some(RangeInclusive::new(thumb.0, thumb.1))
                                    } else {
                                        None
                                    };
                                    doc = if let Some(data) =
                                        document.binary_file.get(thumb.0..=thumb.1)
                                        && ui.button("Open thumbnail in new document").clicked()
                                    {
                                        let doc = Document::new(
                                            data.to_vec(),
                                            PathBuf::from("extracted.jpg"),
                                        );
                                        Some(doc)
                                    } else {
                                        None
                                    };
                                }
                                Err(e) => {
                                    ui.label(e);
                                }
                            }
                        } else {
                            ui.label("No Thumbnail detected");
                        }
                    });
                }
                Err(s) => {
                    ui.label(s);
                }
            }
        }
        if let Some(range) = range {
            document.go_to_range(range);
        }
        if let Some(doc) = doc {
            self.documents.push(doc);
        }
    }

    /// show detection ui
    pub(crate) fn show_detection_ui(
        &mut self,
        ui: &mut egui::Ui,
        error_manager: &mut ErrorManager,
    ) {
        let current_index = self.documents.get_current_index();
        let Some(document) = self.documents.get_current_doc_mut() else {
            return;
        };
        let extension = document.get_file_format().extension.clone();
        let detection = &mut document.windows_data.detection;
        if detection.is_open {
            let mut is_open = detection.is_open;
            if detection.exif_data.is_none() {
                detection.parse_exif(&document.binary_file, &extension);
            }
            egui::Window::new("Detection")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    self.show_detection_exif(ui, error_manager);
                });
            if let Some(document) = self.documents.get_mut(current_index) {
                document.windows_data.detection.is_open = is_open;
            }
        }
    }
}

/// DMS to decimal
fn dms_to_decimal(field: &exif::Field, positive: bool) -> Option<f64> {
    match &field.value {
        exif::Value::Rational(values) if values.len() == 3 => {
            let deg = values[0].to_f64();
            let min = values[1].to_f64();
            let sec = values[2].to_f64();

            let decimal = deg + min / 60.0 + sec / 3600.0;
            Some(if positive { decimal } else { -decimal })
        }
        _ => None,
    }
}
