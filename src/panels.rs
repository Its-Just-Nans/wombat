//! Side panel
use bladvak::BladvakApp;
use bladvak::app::BladvakPanel;
use bladvak::eframe::egui;
use bladvak::egui_extras::{Column, TableBuilder};
use bladvak::errors::ErrorManager;
use file_format::FileFormat;

use crate::WombatApp;

/// File info
#[derive(Debug, Default)]
pub(crate) struct FileInfo;

/// File info
#[derive(Debug)]
pub struct FileInfoData {
    /// Kind of file
    kind: file_format::Kind,
    /// Type of file
    file_type: String,
    /// Extension of file format
    extension: String,
    /// format name
    name: String,
}

impl BladvakPanel for FileInfo {
    type App = WombatApp;
    fn name(&self) -> &'static str {
        "File info"
    }
    fn has_ui(&self) -> bool {
        true
    }
    fn has_settings(&self) -> bool {
        true
    }
    fn ui(&self, app: &mut WombatApp, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        ui.label(format!("File: {}", app.filename.display()));
        let binary_len = app.binary_file.len();
        ui.label(format!("{} bytes", app.binary_file.len()));

        let size_kb = binary_len / 1000;
        if size_kb > 1 {
            ui.label(format!("{:.3} KiB", binary_len / 1024));
            ui.label(format!("{size_kb:.3} KB"));
        }
        let size_megab = binary_len / 1000 / 1000;
        if size_megab > 1 {
            ui.label(format!("{:.3} MiB", binary_len / 1024 / 1024));
            ui.label(format!("{size_megab:.3} MB"));
        }

        if let Some(fmt) = &app.file_format {
            ui.collapsing("File info", |ui| {
                ui.label(format!("Kind: {:?}", fmt.kind));
                ui.label(format!("Type: {}", fmt.file_type));
                ui.label(format!("Name: {}", fmt.name));
                ui.label(format!("Extension: .{}", fmt.extension));
            });
        } else if ui.button("Get file info").clicked() {
            let file_fmt = FileFormat::from_bytes(&app.binary_file);
            let data = FileInfoData {
                kind: file_fmt.kind(),
                file_type: file_fmt.media_type().to_string(),
                extension: file_fmt.extension().to_string(),
                name: file_fmt.name().to_string(),
            };
            app.file_format = Some(data);
        }

        ui.separator();
        ui.label("Binary length");
        ui.add(egui::Slider::new(&mut app.bytes_per_line, 1..=64));
    }

    fn ui_settings(
        &self,
        app: &mut WombatApp,
        ui: &mut egui::Ui,
        error_manager: &mut ErrorManager,
    ) {
        if ui.button("Reset default file").clicked() {
            let default_file = WombatApp::load_default_file();
            if let Err(err) = app.handle_file(default_file) {
                error_manager.add_error(err);
            }
        }
    }
}

/// File info
#[derive(Debug, Default)]
pub(crate) struct FileSelection;

impl BladvakPanel for FileSelection {
    type App = WombatApp;
    fn name(&self) -> &'static str {
        "File selection"
    }
    fn has_ui(&self) -> bool {
        true
    }
    fn has_settings(&self) -> bool {
        false
    }
    fn ui(&self, app: &mut WombatApp, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        if let Some((select1, select2)) = app.selection.as_mut() {
            ui.label("Selection");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(select1).range(0..=*select2));
                let max = app.binary_file.len() - 1;
                ui.label("->");
                ui.add(egui::DragValue::new(select2).range(*select1..=max));
            });
            if select1 == select2
                && let Some(current) = app.binary_file.get(*select1)
            {
                let ascii_char = match *current {
                    x if x >= app.start_ascii_printable && x <= 0x7E => {
                        &(*current as char).to_string()
                    }
                    _ => "unprintable",
                };
                ui.separator();
                ui.label(format!("byte at index {select1}"));
                TableBuilder::new(ui)
                    .column(Column::auto().resizable(true))
                    .column(Column::remainder())
                    .body(|mut body| {
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label("hex");
                                ui.label("octal");
                                ui.label("bin");
                                ui.label("ascci");
                            });
                            row.col(|ui| {
                                ui.label(format!("0x{current:02X}"));
                                ui.label(format!("0o{current:03o}"));
                                ui.label(format!("0b{current:08b}"));
                                ui.label(ascii_char);
                            });
                        });
                    });
            } else {
                let nb_selected = (*select2 as u64) - (*select1 as u64) + 1;
                ui.label(format!("{nb_selected} bytes selected"));
            }
        } else {
            ui.label("No selection");
        }
    }

    fn ui_settings(
        &self,
        _app: &mut WombatApp,
        _ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
    ) {
    }
}
