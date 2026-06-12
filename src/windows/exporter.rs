//! Exporter

use bladvak::eframe::egui::{self, FontId, RichText, TextEdit};
use bladvak::eframe::egui::{Color32, Widget};
use bladvak::errors::ErrorManager;
use std::path::{Path, PathBuf};

use crate::selection::Selection;

/// export type
#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) enum ExportType {
    /// hex
    Hex,
    /// binary
    Binary,
    /// octal
    Octal,
    /// decimal
    Decimal,
}

/// Exporter data
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct Exporter {
    /// is open
    pub(crate) is_open: bool,

    /// prefix value
    prefix: bool,

    /// separator value
    separator: String,

    /// value type
    pub(crate) value_type: ExportType,

    #[serde(skip)]
    /// export error
    export_error: Option<String>,
}

impl Exporter {
    /// New import data
    pub(crate) fn new() -> Self {
        Self {
            is_open: false,
            prefix: true,
            separator: " ".to_string(),
            value_type: ExportType::Hex,
            export_error: None,
        }
    }

    /// reset data
    pub(crate) fn reset(&mut self) {
        self.export_error = None;
    }

    /// Import
    /// # Errors
    /// return error if fails to parse the `value`
    fn format_export(
        selection: &[u8],
        export_type: &ExportType,
        prefix: bool,
        separator: &str,
    ) -> String {
        let prefix = if prefix {
            match export_type {
                ExportType::Binary => "0b",
                ExportType::Decimal => "",
                ExportType::Hex => "0x",
                ExportType::Octal => "0o",
            }
        } else {
            ""
        };
        let tokens = selection
            .iter()
            .map(|one_u8| match export_type {
                ExportType::Binary => format!("{prefix}{one_u8:08b}"),
                ExportType::Hex => format!("{prefix}{one_u8:02X}"),
                ExportType::Octal => format!("{prefix}{one_u8:03o}"),
                ExportType::Decimal => format!("{one_u8}"),
            })
            .collect::<Vec<String>>();
        tokens.join(separator)
    }

    /// Ui inside the windows of the exporter
    pub(crate) fn windows_ui(
        &mut self,
        binary_file: &[u8],
        file_path: &Path,
        selection: &Selection,
        ui: &mut egui::Ui,
        error_manager: &mut ErrorManager,
    ) {
        let previous_import_type = self.value_type.clone();
        ui.horizontal(|ui| {
            ui.label("Export selection to:");
            ui.selectable_value(&mut self.value_type, ExportType::Hex, "Hex");
            ui.selectable_value(&mut self.value_type, ExportType::Binary, "Binary");
            ui.selectable_value(&mut self.value_type, ExportType::Octal, "Octal");
            ui.selectable_value(&mut self.value_type, ExportType::Decimal, "Decimal");
        });
        if previous_import_type != self.value_type {
            self.export_error = None;
        }

        ui.horizontal(|ui| {
            ui.label("Separator");
            ui.text_edit_singleline(&mut self.separator);
        });
        if self.value_type == ExportType::Decimal {
            ui.add_enabled_ui(false, |ui| {
                let mut value = false;
                ui.checkbox(&mut value, "Prefix");
            });
        } else {
            ui.checkbox(&mut self.prefix, "Prefix");
        }
        if binary_file.is_empty() {
            ui.label("File is empty - no selection");
        } else {
            ui.horizontal(|ui| {
                let export_selection = match selection.range {
                    Some(curr_select) => curr_select.0..=curr_select.1,
                    None => 0..=(binary_file.len() - 1),
                };
                if ui.button("Copy to clipboard").clicked() {
                    if let Some(file_selection) = binary_file.get(export_selection.clone()) {
                        let data = Self::format_export(
                            file_selection,
                            &self.value_type,
                            self.prefix,
                            &self.separator,
                        );
                        ui.ctx().copy_text(data);
                    } else {
                        self.export_error = Some("Cannot determine selection".to_string());
                    }
                }
                if let Some(slice) = binary_file.get(export_selection)
                    && ui.button("Export as raw").clicked()
                {
                    let file_name = file_path
                        .file_name()
                        .map_or_else(|| PathBuf::from("exported"), PathBuf::from);
                    if let Err(e) =
                        bladvak::utils::save_file(slice, &file_name.with_extension("bin"))
                    {
                        error_manager.add_error(e);
                    }
                }
                // if ui.button("Export to file").clicked() {
                //     // TODO
                // }
            });
            let selected_preview = if let Some(range) = selection.range {
                let stop = range.1.min(range.0 + 49);
                range.0..=stop
            } else {
                let max = (binary_file.len() - 1).min(49);
                0..=max
            };
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Preview on {} bytes (of {})",
                    selected_preview.clone().count(),
                    if let Some(range) = selection.range {
                        let range_len = range.1 - range.0 + 1;
                        format!("{range_len} bytes selection")
                    } else {
                        "file".to_string()
                    }
                ));
                if let Some(err) = &self.export_error {
                    ui.label(RichText::new(err).color(Color32::LIGHT_RED));
                }
            });
            if let Some(preview_value) = binary_file.get(selected_preview) {
                let mut formatted = Self::format_export(
                    preview_value,
                    &self.value_type,
                    self.prefix,
                    &self.separator,
                );
                TextEdit::multiline(&mut formatted)
                    .min_size(ui.available_size())
                    .desired_width(f32::INFINITY)
                    .font(FontId::monospace(12.0))
                    .ui(ui);
            }
        }
    }

    /// Show the exporter ui
    pub(crate) fn ui(
        &mut self,
        binary_file: &[u8],
        file_path: &Path,
        selection: &Selection,
        ui: &mut egui::Ui,
        error_manager: &mut ErrorManager,
    ) -> Option<Vec<u8>> {
        if self.is_open {
            let mut is_open = self.is_open;
            egui::Window::new("Exporter")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    self.windows_ui(binary_file, file_path, selection, ui, error_manager);
                });
            self.is_open = is_open;
        }
        None
    }
}
