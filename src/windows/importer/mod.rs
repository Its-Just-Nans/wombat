//! Importer

mod import_binary;
mod import_decimal;
mod import_hex;
mod import_octal;

pub use import_binary::parse_binary_string;
pub use import_decimal::parse_decimal_string;
pub use import_hex::parse_hex_string;
pub use import_octal::parse_octal_string;

use bladvak::eframe::egui::{self, RichText, TextEdit};
use bladvak::eframe::egui::{Color32, Widget};
use bladvak::errors::ErrorManager;

/// import type
#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) enum ImportType {
    /// hex import
    Hex,
    /// string import
    String,
    /// binary import
    Binary,
    /// octal import
    Octal,
    /// decimal import big endian
    DecimalBigEndian,
    /// decimal import little endian
    DecimalLittleEndian,
}

/// Histogram data
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct Importer {
    /// is open
    pub(crate) is_open: bool,

    /// current value
    value: String,
    /// value type
    pub(crate) value_type: ImportType,

    #[serde(skip)]
    /// import error
    import_error: Option<String>,
}

impl Importer {
    /// New import data
    pub(crate) fn new() -> Self {
        Self {
            is_open: false,
            value: String::new(),
            value_type: ImportType::String,
            import_error: None,
        }
    }

    /// reset data
    pub(crate) fn reset(&mut self) {
        self.import_error = None;
    }

    /// Import
    /// # Errors
    /// return error if fails to parse the `value`
    fn import(value: &str, value_type: &ImportType) -> Result<Vec<u8>, String> {
        match value_type {
            ImportType::String => Ok(value.as_bytes().to_vec()),
            ImportType::Hex => parse_hex_string(value),
            ImportType::Binary => parse_binary_string(value),
            ImportType::Octal => parse_octal_string(value),
            ImportType::DecimalBigEndian => parse_decimal_string(value, true),
            ImportType::DecimalLittleEndian => parse_decimal_string(value, false),
        }
    }
    /// Show the importer ui
    pub(crate) fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
    ) -> Option<Vec<u8>> {
        if self.is_open {
            let mut is_open = self.is_open;
            let mut ret = None;
            egui::Window::new("Import")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    let previous_import_type = self.value_type.clone();
                    ui.horizontal(|ui| {
                        ui.label("Import from:");
                        ui.selectable_value(&mut self.value_type, ImportType::String, "String");
                        ui.selectable_value(&mut self.value_type, ImportType::Hex, "Hex");
                        ui.selectable_value(&mut self.value_type, ImportType::Binary, "Binary");
                        ui.selectable_value(&mut self.value_type, ImportType::Octal, "Octal");
                        ui.selectable_value(
                            &mut self.value_type,
                            ImportType::DecimalBigEndian,
                            "Decimal (Big Endian)",
                        );
                        ui.selectable_value(
                            &mut self.value_type,
                            ImportType::DecimalLittleEndian,
                            "Decimal (Little Endian)",
                        );
                    });
                    if previous_import_type != self.value_type {
                        self.import_error = None;
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Import").clicked() {
                            if self.value.is_empty() {
                                self.import_error = Some("Input cannot be empty".into());
                            } else {
                                ret = Some(Self::import(&self.value, &self.value_type));
                            }
                        }
                        if ui.button("Clear").clicked() {
                            self.value.clear();
                            self.import_error = None;
                        }
                        if let Some(err) = &self.import_error {
                            ui.label(RichText::new(err).color(Color32::LIGHT_RED));
                        }
                    });
                    if TextEdit::multiline(&mut self.value)
                        .min_size(ui.available_size())
                        .desired_width(f32::INFINITY)
                        .ui(ui)
                        .changed()
                    {
                        self.import_error = None;
                    }
                });
            self.is_open = is_open;
            if let Some(import_result) = ret {
                match import_result {
                    Ok(res) => return Some(res),
                    Err(import_err) => self.import_error = Some(import_err),
                }
            }
        }
        None
    }
}
