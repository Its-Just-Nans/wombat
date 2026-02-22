//! Exporter

use bladvak::eframe::egui::{self, FontId, RichText, TextEdit};
use bladvak::eframe::egui::{Color32, Widget};
use bladvak::errors::ErrorManager;

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

/// Histogram data
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

    /// Show the exporter ui
    pub(crate) fn ui(
        &mut self,
        selection: &[u8],
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
    ) -> Option<Vec<u8>> {
        if self.is_open {
            let mut is_open = self.is_open;
            egui::Window::new("Exporter")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
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
                    if self.value_type != ExportType::Decimal {
                        ui.checkbox(&mut self.prefix, "Prefix");
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Copy to clipboard").clicked() {
                            let data = Self::format_export(
                                selection,
                                &self.value_type,
                                self.prefix,
                                &self.separator,
                            );
                            ui.ctx().copy_text(data);
                        }
                        // if ui.button("Export to file").clicked() {
                        //     // TODO
                        // }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Preview on 50 bytes");
                        if let Some(err) = &self.export_error {
                            ui.label(RichText::new(err).color(Color32::LIGHT_RED));
                        }
                    });
                    let max_preview = 50.min(selection.len());
                    if let Some(preview_value) = selection.get(0..max_preview) {
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
                });
            self.is_open = is_open;
        }
        None
    }
}
