//! Importer

mod import_binary;
mod import_decimal;
mod import_hex;
mod import_octal;

use std::fmt::Display;
use std::path::PathBuf;

use bladvak::utils::BladvakClipBoard;
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
    /// base 64
    Base64,
    /// base 64 (URL safe)
    Base64Url,
}

impl Display for ImportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hex => write!(f, "Hex"),
            Self::String => write!(f, "String"),
            Self::Binary => write!(f, "Binary"),
            Self::Octal => write!(f, "Octal"),
            Self::DecimalBigEndian => write!(f, "Decimal (big endian)"),
            Self::DecimalLittleEndian => write!(f, "Decimal (little endian)"),
            Self::Base64 => write!(f, "Base 64"),
            Self::Base64Url => write!(f, "Base 64 (url)"),
        }
    }
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

    /// import error
    #[serde(skip)]
    import_error: Option<String>,

    /// clipboard
    #[serde(skip)]
    clipboard: BladvakClipBoard,
}

impl Importer {
    /// New import data
    pub(crate) fn new() -> Self {
        Self {
            is_open: false,
            value: String::new(),
            value_type: ImportType::String,
            import_error: None,
            clipboard: BladvakClipBoard::default(),
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
            ImportType::Base64 => {
                use base64::{Engine, alphabet::STANDARD, engine};
                let engine_config = engine::GeneralPurposeConfig::new()
                    .with_decode_allow_trailing_bits(true)
                    .with_decode_padding_mode(engine::DecodePaddingMode::Indifferent);
                let process_engine = engine::GeneralPurpose::new(&STANDARD, engine_config);
                let decoded = process_engine
                    .decode(value.as_bytes())
                    .map_err(|err| format!("Error importing base64: {err}"))?;
                Ok(decoded)
            }
            ImportType::Base64Url => {
                use base64::{Engine, alphabet::URL_SAFE, engine};
                let engine_config = engine::GeneralPurposeConfig::new()
                    .with_decode_allow_trailing_bits(true)
                    .with_decode_padding_mode(engine::DecodePaddingMode::Indifferent);
                let process_engine = engine::GeneralPurpose::new(&URL_SAFE, engine_config);
                let decoded = process_engine
                    .decode(value.as_bytes())
                    .map_err(|err| format!("Error importing base64 (url safe): {err}"))?;
                Ok(decoded)
            }
        }
    }

    /// combo box import type
    fn combo_box_ui(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_id_salt("import_combo_box")
            .selected_text(self.value_type.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.value_type,
                    ImportType::String,
                    ImportType::String.to_string(),
                );
                ui.selectable_value(
                    &mut self.value_type,
                    ImportType::Hex,
                    ImportType::Hex.to_string(),
                );
                ui.selectable_value(
                    &mut self.value_type,
                    ImportType::Binary,
                    ImportType::Binary.to_string(),
                );
                ui.selectable_value(
                    &mut self.value_type,
                    ImportType::Octal,
                    ImportType::Octal.to_string(),
                );
                ui.selectable_value(
                    &mut self.value_type,
                    ImportType::DecimalLittleEndian,
                    ImportType::DecimalLittleEndian.to_string(),
                );
                ui.selectable_value(
                    &mut self.value_type,
                    ImportType::DecimalBigEndian,
                    ImportType::DecimalBigEndian.to_string(),
                );
                ui.selectable_value(&mut self.value_type, ImportType::Base64, "Base 64");
                ui.selectable_value(
                    &mut self.value_type,
                    ImportType::Base64Url,
                    ImportType::Base64Url.to_string(),
                );
            });
    }

    /// clipboard import ui
    fn clipboard_import_ui(
        &mut self,
        ui: &mut egui::Ui,
        error_manager: &mut ErrorManager,
    ) -> Option<Result<(PathBuf, Vec<u8>), String>> {
        let mut ret = None;
        if ui.button("Import file from clipboard").clicked()
            && let Err(err) = self.clipboard.launch_get_file()
        {
            error_manager.add_error(err);
        }
        match self.clipboard.files(ui.ctx()) {
            Some(Ok(files_list)) => {
                if let Some(file) = files_list.into_iter().nth(0) {
                    match file.get_data() {
                        Ok(d) => {
                            ret = Some(Ok((PathBuf::from("imported_file.bin"), d)));
                        }
                        Err(err) => {
                            ret = Some(Err(err));
                        }
                    }
                }
            }
            Some(Err(err)) => {
                ret = Some(Err(err));
            }
            None => {}
        }
        if ui.button("Import text from clipboard").clicked()
            && let Err(err) = self.clipboard.launch_get_text()
        {
            error_manager.add_error(err);
        }
        match self.clipboard.text(ui.ctx()) {
            Some(Ok(text)) => {
                ret = Some(Ok((PathBuf::from("imported_text.txt"), text.into_bytes())));
            }
            Some(Err(err)) => {
                ret = Some(Err(err));
            }
            None => {}
        }
        ret
    }

    /// Show the importer ui
    pub(crate) fn ui(
        &mut self,
        ui: &mut egui::Ui,
        error_manager: &mut ErrorManager,
    ) -> Option<(PathBuf, Vec<u8>)> {
        if self.is_open {
            let mut is_open = self.is_open;
            let mut ret = None;
            egui::Window::new("Import")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    ret = self.clipboard_import_ui(ui, error_manager);
                    ui.separator();
                    let previous_import_type = self.value_type.clone();
                    ui.horizontal(|ui| {
                        ui.label("Import from:");
                        self.combo_box_ui(ui);
                    });
                    if previous_import_type != self.value_type {
                        self.import_error = None;
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Import").clicked() {
                            if self.value.is_empty() {
                                self.import_error = Some("Input cannot be empty".into());
                            } else {
                                match Self::import(&self.value, &self.value_type) {
                                    Ok(d) => {
                                        ret = Some(Ok((PathBuf::from("imported.bin"), d)));
                                    }
                                    Err(err) => {
                                        ret = Some(Err(err));
                                    }
                                }
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
                    let preview_len = self.value.len().min(50);
                    ui.label(format!("Import preview for {preview_len} chars"));
                    if let Some(val) = self.value.get(0..preview_len) {
                        match Self::import(val, &self.value_type) {
                            Ok(res) => {
                                let import_preview = res
                                    .iter()
                                    .map(|one_u8| format!("0x{one_u8:02X}"))
                                    .collect::<Vec<String>>()
                                    .join(",");
                                ui.label(import_preview);
                            }
                            Err(err) => {
                                ui.label(RichText::new(err).color(Color32::LIGHT_RED));
                            }
                        }
                    }
                });
            self.is_open = is_open;
            if let Some(import_result) = ret {
                match import_result {
                    Ok((filename, data)) => return Some((filename, data)),
                    Err(import_err) => self.import_error = Some(import_err),
                }
            }
        }
        None
    }
}
