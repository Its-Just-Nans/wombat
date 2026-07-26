//! Metadata

use std::io::Cursor;

use bladvak::ErrorManager;
use bladvak::eframe::egui;
use bladvak::egui_extras::{Column, TableBuilder};
use exif::Exif;

use crate::WombatApp;

/// Metadata
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub(crate) struct Metadata {
    /// Is open
    pub(crate) is_open: bool,
    /// exif
    #[serde(skip)]
    exif: Option<Result<Exif, String>>,
}

impl std::fmt::Debug for Metadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metadata")
            .field("is_open", &self.is_open)
            .finish_non_exhaustive()
    }
}

impl Metadata {
    /// reset
    pub(crate) fn reset(&mut self) {
        self.exif = None;
    }

    /// parse exif
    pub(crate) fn parse_exif(&mut self, binary_file: &[u8]) {
        let cursor = Cursor::new(binary_file);
        let mut bufreader = std::io::BufReader::new(cursor);
        self.exif = Some(
            exif::Reader::new()
                .read_from_container(&mut bufreader)
                .map_err(|e| format!("Failed to parse exif: {e}")),
        );
    }

    /// show exif
    fn show_exif(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        if let Some(exif) = &self.exif {
            match exif {
                Ok(exif) => {
                    ui.collapsing("Exif info", |ui| {
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
                                            ui.label(format!(
                                                "{}",
                                                field.display_value().with_unit(exif)
                                            ));
                                        });
                                    });
                                }
                            });
                        if let Some(lat) = exif.get_field(exif::Tag::GPSLatitude, exif::In::PRIMARY)
                            && let Some(lat_ref) =
                                exif.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY)
                            && let Some(lon) =
                                exif.get_field(exif::Tag::GPSLongitude, exif::In::PRIMARY)
                            && let Some(lon_ref) =
                                exif.get_field(exif::Tag::GPSLongitudeRef, exif::In::PRIMARY)
                        {
                            let north =
                                matches!(&lat_ref.value, exif::Value::Ascii(v) if v[0] == b"N");
                            let east =
                                matches!(&lon_ref.value, exif::Value::Ascii(v) if v[0] == b"E");
                            if let Some(latitude) = dms_to_decimal(lat, north)
                                && let Some(longitude) = dms_to_decimal(lon, east)
                            {
                                ui.horizontal(|ui| {
                                    ui.label("GPS detected: ");
                                    let geo_url = format!("geo:{latitude:.8},{longitude:.8}");
                                    ui.add(egui::Hyperlink::new(geo_url).open_in_new_tab(true));
                                });
                            } else {
                                ui.label("Failed to parse GPS data");
                            }
                        } else {
                            ui.label("No GPS data found");
                        }
                        if let Some(jpeg_interchange_format) =
                            exif.get_field(exif::Tag::JPEGInterchangeFormat, exif::In::THUMBNAIL)
                            && let Some(jpeg_interchange_format_length) = exif.get_field(
                                exif::Tag::JPEGInterchangeFormatLength,
                                exif::In::THUMBNAIL,
                            )
                        {
                            if let exif::Value::Long(offset) = &jpeg_interchange_format.value
                                && let Some(offset) = offset.first()
                                && let exif::Value::Long(length) =
                                    &jpeg_interchange_format_length.value
                                && let Some(length) = length.first()
                            {
                                let jpeg_size = offset + length;
                                ui.horizontal(|ui| {
                                    ui.label("Thumbnail found");
                                    ui.label(format!("Position: {offset} to {jpeg_size} of Exif"));
                                });
                                // let thumbnail = &exif_data[offset..offset + length];
                            } else {
                                ui.label("Failed to get Thumbnail");
                            }
                        } else {
                            ui.label("No Thumbnail detected");
                        }
                    });
                }
                Err(s) => {
                    ui.label("No exif detected");
                    ui.label(s);
                }
            }
        }
    }
}

impl WombatApp {
    /// show metadata ui
    pub(crate) fn show_metadata_ui(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        let current_index = self.documents.get_current_index();
        let Some(document) = self.documents.get_current_doc_mut() else {
            return;
        };
        let metadata = &mut document.windows_data.metadata;
        if metadata.is_open {
            let mut is_open = metadata.is_open;
            egui::Window::new("Metadata")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    if metadata.exif.is_none() {
                        metadata.parse_exif(&document.binary_file);
                    }
                    metadata.show_exif(ui, error_manager);
                });
            if let Some(document) = self.documents.get_mut(current_index) {
                document.windows_data.metadata.is_open = is_open;
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
