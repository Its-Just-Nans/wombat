//! Side panel

use bladvak::BladvakApp;
use bladvak::app::BladvakPanel;
use bladvak::eframe::egui;
use bladvak::errors::ErrorManager;
use file_format::FileFormat;
use std::path::PathBuf;

use crate::WombatApp;

/// File info
#[derive(Debug, Default)]
pub(crate) struct FileInfo;

/// File info
#[derive(Debug)]
pub struct FileInfoData {
    /// Kind of file
    pub(crate) kind: file_format::Kind,
    /// Type of file
    pub(crate) file_type: String,
    /// Extension of file format
    pub(crate) extension: String,
    /// format name
    pub(crate) name: String,
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
        ui.label(format!("{} bytes", app.binary_file.len()));
        #[allow(clippy::cast_precision_loss)]
        let binary_len = app.binary_file.len() as f64;

        let size_kb = binary_len / 1000.0;
        if size_kb > 1.0 {
            ui.label(format!("{:.3} KiB", binary_len / 1024.0))
                .on_hover_text(format!("{binary_len} / 1024"));
            ui.label(format!("{size_kb:.3} KB"))
                .on_hover_text(format!("{binary_len} / 1000"));
        }
        let size_megab = binary_len / 1000.0 / 1000.0;
        if size_megab > 1.0 {
            ui.label(format!("{:.3} MiB", binary_len / 1024.0 / 1024.0))
                .on_hover_text(format!("{binary_len} / (1024^2)"));
            ui.label(format!("{size_megab:.3} MB"))
                .on_hover_text(format!("{binary_len} / (1000^2)"));
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
    fn ui(&self, app: &mut WombatApp, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        if let Some((select1, select2)) = app.selection.as_mut() {
            let mut mark_stale = false;
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
                ui.separator();
                ui.label(format!("byte at index {select1}"));
                WombatApp::ui_table_u8(ui, *current);
            } else {
                let nb_selected = select2.checked_sub(*select1).map_or(0, |d| d as u64 + 1);
                ui.label(format!("{nb_selected} bytes selected"));
                let range = *select1..=*select2;
                if ui.button("Delete selection").clicked() {
                    app.binary_file.drain(range.clone());
                    *select2 = select1.checked_sub(1).unwrap_or(0);
                    mark_stale = true;
                }
                if let Some(slice) = app.binary_file.get(range) {
                    ui.collapsing("Export selection", |ui| {
                        if ui.button("Export as raw").clicked()
                            && let Err(e) =
                                bladvak::utils::save_file(slice, &PathBuf::from("exported.bin"))
                        {
                            error_manager.add_error(e);
                        }
                        if ui.button("Export as hex").clicked() {
                            let file_as_hex = slice
                                .iter()
                                .map(|byte| format!("{byte:02X}"))
                                .collect::<Vec<String>>()
                                .join(" ");
                            if let Err(e) = bladvak::utils::save_file(
                                file_as_hex.as_bytes(),
                                &PathBuf::from("exported.hex"),
                            ) {
                                error_manager.add_error(e);
                            }
                        }
                        if ui.button("Copy as hex").clicked() {
                            let file_as_hex = slice
                                .iter()
                                .map(|byte| format!("{byte:02X}"))
                                .collect::<Vec<String>>()
                                .join(" ");
                            ui.ctx().copy_text(file_as_hex);
                        }
                    });
                }
            }
            if mark_stale {
                app.stale();
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
