//! Ui for the table view

use bladvak::eframe::egui::{self, Color32, RichText, Theme};
use bladvak::egui_extras::{Column, TableBuilder};

use crate::display_settings::{Accent, DisplaySettings};

impl DisplaySettings {
    /// Ui for the table representation of a u8
    pub(crate) fn ui_table_u8(&self, ui: &mut egui::Ui, current: u8, accent_ui: &Accent) {
        let accent = if ui.ctx().theme() == Theme::Light {
            Color32::BLACK
        } else {
            Color32::WHITE
        };

        let accent_label = |ui: &mut egui::Ui, current_accent: Accent, text: String| {
            if accent_ui == &current_accent {
                ui.monospace(RichText::new(text).color(accent));
            } else {
                ui.monospace(text);
            }
        };
        TableBuilder::new(ui)
            .striped(true)
            .column(Column::auto().resizable(true))
            .column(Column::remainder())
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label("u8");
                });
                header.col(|ui| {
                    ui.label("Value");
                });
            })
            .body(|mut body| {
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        ui.label("Decimal");
                    });
                    row.col(|ui| {
                        accent_label(ui, Accent::Decimal, format!("{current}"));
                    });
                });
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        ui.label("Hex");
                    });
                    row.col(|ui| {
                        accent_label(ui, Accent::Hex, format!("0x{current:02X}"));
                    });
                });
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        ui.label("Octal");
                    });
                    row.col(|ui| {
                        accent_label(ui, Accent::Octal, format!("0o{current:03o}"));
                    });
                });
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        ui.label("Bin");
                    });
                    row.col(|ui| {
                        accent_label(ui, Accent::Binary, format!("0b{current:08b}"));
                    });
                });
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        ui.label("ASCII");
                    });
                    row.col(|ui| {
                        let ascii_char = self.ascii_to_string(current);
                        accent_label(ui, Accent::Ascii, ascii_char);
                    });
                });
            });
    }
}

/// Ui for the table representation of a u16
pub(crate) fn ui_table_u16(ui: &mut egui::Ui, bytes: [u8; 2]) {
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::auto().resizable(true))
        .column(Column::remainder())
        .column(Column::remainder())
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.label("u16");
            });
            header.col(|ui| {
                ui.label("L-Endian");
            });
            header.col(|ui| {
                ui.label("B-Endian");
            });
        })
        .body(|mut body| {
            let range_little_endian = u16::from_le_bytes(bytes);
            let range_be = u16::from_be_bytes(bytes);
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label("Decimal");
                });
                row.col(|ui| {
                    ui.label(format!("{range_little_endian}"));
                });
                row.col(|ui| {
                    ui.label(format!("{range_be}"));
                });
            });
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label("Hex");
                });
                row.col(|ui| {
                    ui.label(format!("0x{range_little_endian:X}"));
                });
                row.col(|ui| {
                    ui.label(format!("0x{range_be:X}"));
                });
            });
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label("Octal");
                });
                row.col(|ui| {
                    ui.label(format!("0o{range_little_endian:o}"));
                });
                row.col(|ui| {
                    ui.label(format!("0o{range_be:o}"));
                });
            });
        });
    ui.horizontal(|ui| {
        ui.label("String");
        match std::str::from_utf8(&bytes) {
            Ok(string) => ui.label(string),
            Err(_) => ui.label("NONE"),
        };
    });
}

/// Ui for the table representation of a u32
pub(crate) fn ui_table_u32(ui: &mut egui::Ui, bytes: [u8; 4]) {
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::auto().resizable(true))
        .column(Column::remainder())
        .column(Column::remainder())
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.label("u32");
            });
            header.col(|ui| {
                ui.label("L-Endian");
            });
            header.col(|ui| {
                ui.label("B-Endian");
            });
        })
        .body(|mut body| {
            let range_little_endian = u32::from_le_bytes(bytes);
            let range_be = u32::from_be_bytes(bytes);
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label("Decimal");
                });
                row.col(|ui| {
                    ui.label(format!("{range_little_endian}"));
                });
                row.col(|ui| {
                    ui.label(format!("{range_be}"));
                });
            });
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label("Hex");
                });
                row.col(|ui| {
                    ui.label(format!("0x{range_little_endian:X}"));
                });
                row.col(|ui| {
                    ui.label(format!("0x{range_be:X}"));
                });
            });
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label("Octal");
                });
                row.col(|ui| {
                    ui.label(format!("0o{range_little_endian:o}"));
                });
                row.col(|ui| {
                    ui.label(format!("0o{range_be:o}"));
                });
            });
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label("Unicode");
                });
                row.col(|ui| {
                    if let Some(charac) = std::char::from_u32(range_little_endian) {
                        ui.label(format!("{charac}"));
                    } else {
                        ui.label("NONE");
                    }
                });
                row.col(|ui| {
                    if let Some(charac) = std::char::from_u32(range_be) {
                        ui.label(format!("{charac}"));
                    } else {
                        ui.label("NONE");
                    }
                });
            });
        });
    ui.horizontal(|ui| {
        ui.label("String");
        match std::str::from_utf8(&bytes) {
            Ok(string) => ui.label(string),
            Err(_) => ui.label("NONE"),
        };
    });
}

/// Ui for the table representation of a u64
pub(crate) fn ui_table_u64(ui: &mut egui::Ui, bytes: [u8; 8]) {
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::auto().resizable(true))
        .column(Column::remainder())
        .column(Column::remainder())
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.label("u64");
            });
            header.col(|ui| {
                ui.label("L-Endian");
            });
            header.col(|ui| {
                ui.label("B-Endian");
            });
        })
        .body(|mut body| {
            let range_little_endian = u64::from_le_bytes(bytes);
            let range_be = u64::from_be_bytes(bytes);
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label("Decimal");
                });
                row.col(|ui| {
                    ui.label(format!("{range_little_endian}"));
                });
                row.col(|ui| {
                    ui.label(format!("{range_be}"));
                });
            });
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label("Hex");
                });
                row.col(|ui| {
                    ui.label(format!("0x{range_little_endian:X}"));
                });
                row.col(|ui| {
                    ui.label(format!("0x{range_be:X}"));
                });
            });
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label("Octal");
                });
                row.col(|ui| {
                    ui.label(format!("0o{range_little_endian:o}"));
                });
                row.col(|ui| {
                    ui.label(format!("0o{range_be:o}"));
                });
            });
        });
    ui.horizontal(|ui| {
        ui.label("String");
        match std::str::from_utf8(&bytes) {
            Ok(string) => ui.label(string),
            Err(_) => ui.label("NONE"),
        };
    });
}

/// Ui for the table representation of a u128
pub(crate) fn ui_table_u128(ui: &mut egui::Ui, bytes: [u8; 16]) {
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::auto().resizable(true))
        .column(Column::remainder())
        .column(Column::remainder())
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.label("u128");
            });
            header.col(|ui| {
                ui.label("L-Endian");
            });
            header.col(|ui| {
                ui.label("B-Endian");
            });
        })
        .body(|mut body| {
            let range_little_endian = u128::from_le_bytes(bytes);
            let range_be = u128::from_be_bytes(bytes);
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label("Decimal");
                });
                row.col(|ui| {
                    ui.label(format!("{range_little_endian}"));
                });
                row.col(|ui| {
                    ui.label(format!("{range_be}"));
                });
            });
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label("Hex");
                });
                row.col(|ui| {
                    ui.label(format!("0x{range_little_endian:X}"));
                });
                row.col(|ui| {
                    ui.label(format!("0x{range_be:X}"));
                });
            });
            body.row(30.0, |mut row| {
                row.col(|ui| {
                    ui.label("Octal");
                });
                row.col(|ui| {
                    ui.label(format!("0o{range_little_endian:o}"));
                });
                row.col(|ui| {
                    ui.label(format!("0o{range_be:o}"));
                });
            });
        });
    ui.horizontal(|ui| {
        ui.label("String");
        match std::str::from_utf8(&bytes) {
            Ok(string) => ui.label(string),
            Err(_) => ui.label("NONE"),
        };
    });
}
