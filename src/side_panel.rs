//! Side panel
use bladvak::eframe::egui;
use bladvak::egui_extras::{Column, TableBuilder};
use bladvak::errors::ErrorManager;

use crate::WombatApp;

impl WombatApp {
    /// Side panel
    pub(crate) fn app_side_panel(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        if !self.sidebar_as_window {
            self.file_info(ui);
        }
    }

    /// Image info
    pub(crate) fn file_info(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("File: {}", self.filename.display()));

        ui.label(format!("{} bytes", self.binary_file.len()));

        let size_kb = self.binary_file.len() as f32 / 1000.0;
        if size_kb > 1.0 {
            ui.label(format!("{:.3} KiB", self.binary_file.len() as f32 / 1024.0));
            ui.label(format!("{:.3} KB", size_kb));
        }
        let size_mb = self.binary_file.len() as f32 / 1000.0 / 1000.0;
        if size_mb > 1.0 {
            ui.label(format!(
                "{:.3} MiB",
                self.binary_file.len() as f32 / 1024.0 / 1024.0
            ));
            ui.label(format!("{:.3} MB", size_mb));
        }

        ui.separator();
        ui.label("Binary length");
        ui.add(egui::Slider::new(&mut self.bytes_per_line, 1..=64));

        if let Some((select1, select2)) = self.selection.as_mut() {
            ui.label("Selection");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(select1).range(0..=*select2));
                let max = self.binary_file.len() - 1;
                ui.label("->");
                ui.add(egui::DragValue::new(select2).range(*select1..=max));
            });
            if select1 == select2
                && let Some(current) = self.binary_file.get(*select1)
            {
                let ascii_char = match *current {
                    x if x >= self.start_ascii_printable && x <= 0x7E => {
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
    }
}
