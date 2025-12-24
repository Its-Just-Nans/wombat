//! Side panel
use bladvak::BladvakApp;
use bladvak::app::BladvakPanel;
use bladvak::eframe::egui;
use bladvak::egui_extras::{Column, TableBuilder};
use bladvak::errors::ErrorManager;

use crate::WombatApp;

/// File info
#[derive(Debug, Default)]
pub(crate) struct FileInfo;

impl BladvakPanel for FileInfo {
    type App = WombatApp;
    fn name(&self, _app: &WombatApp) -> &str {
        "File info"
    }
    fn has_ui(&self, _app: &WombatApp) -> bool {
        true
    }
    fn has_settings(&self, _app: &WombatApp) -> bool {
        true
    }
    fn ui(&self, app: &mut WombatApp, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        egui::Frame::central_panel(&ui.ctx().style()).show(ui, |ui| {
            ui.label(format!("File: {}", app.filename.display()));

            ui.label(format!("{} bytes", app.binary_file.len()));

            let size_kb = app.binary_file.len() as f32 / 1000.0;
            if size_kb > 1.0 {
                ui.label(format!("{:.3} KiB", app.binary_file.len() as f32 / 1024.0));
                ui.label(format!("{:.3} KB", size_kb));
            }
            let size_mb = app.binary_file.len() as f32 / 1000.0 / 1000.0;
            if size_mb > 1.0 {
                ui.label(format!(
                    "{:.3} MiB",
                    app.binary_file.len() as f32 / 1024.0 / 1024.0
                ));
                ui.label(format!("{:.3} MB", size_mb));
            }

            ui.separator();
            ui.label("Binary length");
            ui.add(egui::Slider::new(&mut app.bytes_per_line, 1..=64));

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
                    ui.label(format!("byte at index {}", select1));
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
                }
            } else {
                ui.label("No selection");
            }
        });
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
