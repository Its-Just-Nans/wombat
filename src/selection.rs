//! Selection

use std::path::PathBuf;

use bladvak::{
    ErrorManager,
    app::BladvakPanel,
    eframe::egui::{self, Color32, Theme},
};

use crate::display_settings::Accent;
use crate::ui_table::ui_table_u16;
use crate::ui_table::ui_table_u32;
use crate::ui_table::ui_table_u64;
use crate::ui_table::ui_table_u128;
use crate::{WombatApp, document::Document};

/// Selection
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct Selection {
    /// Selection range
    pub(crate) range: Option<(usize, usize)>,
    /// Selection color
    pub(crate) color: (Color32, Color32),
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            range: None,
            color: (Color32::ORANGE, Color32::GOLD),
        }
    }
}

impl Selection {
    /// reset selection
    pub(crate) fn reset(&mut self) {
        self.range = None;
    }
}

impl WombatApp {
    /// Ui for the selection
    fn ui_selection(&mut self, ui: &mut egui::Ui) {
        let Some(document) = self.documents.get_current_doc_mut() else {
            return;
        };
        if let Some((select1, select2)) = document.selection.range {
            if select1 == select2
                && let Some(current) = document.binary_file.get(select1)
            {
                ui.separator();
                ui.label(format!("1 byte at index 0x{select1:X} ({select1})"));
                self.display_settings
                    .ui_table_u8(ui, *current, &Accent::Hex);
            } else {
                ui.separator();
                let nb_selected = select2.checked_sub(select1).map_or(0, |d| d as u64 + 1);
                ui.label(format!("{nb_selected} bytes selected"));
                let range = select1..=select2;

                if nb_selected == 2
                    && let Some(slice) = document.binary_file.get(range.clone())
                    && let Ok(bytes) = <[u8; 2]>::try_from(slice)
                {
                    ui_table_u16(ui, bytes);
                }

                if nb_selected == 3
                    && let Some(slice) = document.binary_file.get(range.clone())
                    && let Ok(bytes) = <[u8; 3]>::try_from(slice)
                {
                    let mut bytes_table = [0u8; 4];
                    bytes_table[..bytes.len()].copy_from_slice(&bytes);
                    ui_table_u32(ui, bytes_table);
                    if self.display_settings.show_color_picker {
                        ui.horizontal(|ui| {
                            ui.label("Color preview");
                            let mut rgb = bytes;
                            ui.color_edit_button_srgb(&mut rgb);
                        });
                    }
                }

                if nb_selected == 4
                    && let Some(slice) = document.binary_file.get(range.clone())
                    && let Ok(bytes) = <[u8; 4]>::try_from(slice)
                {
                    ui_table_u32(ui, bytes);
                    if self.display_settings.show_color_picker {
                        ui.horizontal(|ui| {
                            ui.label("Color preview");
                            let mut rgb = bytes;
                            ui.color_edit_button_srgba_unmultiplied(&mut rgb);
                        });
                    }
                }

                if nb_selected > 4
                    && nb_selected <= 8
                    && let Some(slice) = document.binary_file.get(range.clone())
                {
                    let mut bytes = [0u8; 8];
                    bytes[..slice.len()].copy_from_slice(slice);
                    ui_table_u64(ui, bytes);
                }

                if nb_selected > 8
                    && nb_selected <= 16
                    && let Some(slice) = document.binary_file.get(range.clone())
                {
                    let mut bytes = [0u8; 16];
                    bytes[..slice.len()].copy_from_slice(slice);
                    ui_table_u128(ui, bytes);
                }
            }
            if document.binary_file.is_empty() {
                document.selection.range = None;
            }
        }
    }
}

/// selection panel
#[derive(Debug)]
pub(crate) struct PanelSelection;

impl BladvakPanel for PanelSelection {
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
        let Some(document) = app.documents.get_current_doc_mut() else {
            return;
        };
        if document.binary_file.is_empty() {
            document.selection.range = None;
        }
        let mut mark_stale = false;
        let mut mark_selection_stale = false;
        if let Some((select1, select2)) = document.selection.range.as_mut() {
            ui.horizontal(|ui| {
                ui.label("Selection");
                let color_to_edit = if ui.ctx().theme() == Theme::Light {
                    &mut document.selection.color.0
                } else {
                    &mut document.selection.color.1
                };
                ui.color_edit_button_srgba(color_to_edit);
            });
            ui.horizontal(|ui| {
                if ui
                    .button("▶")
                    .on_hover_text("Go to start of selection")
                    .clicked()
                {
                    document
                        .offset
                        .go_to_index(*select1, document.bytes_per_line);
                }
                if ui
                    .add(egui::DragValue::new(select1).range(0..=*select2))
                    .changed()
                {
                    mark_selection_stale = true;
                }
                let max = if document.binary_file.is_empty() {
                    0
                } else {
                    document.binary_file.len() - 1
                };
                ui.label("->")
                    .on_hover_text(format!("0x{select1:X} to 0x{select2:X}"));
                if ui
                    .add(egui::DragValue::new(select2).range(*select1..=max))
                    .changed()
                {
                    mark_selection_stale = true;
                }
                if ui
                    .button("◀")
                    .on_hover_text("Go to end of selection")
                    .clicked()
                {
                    document
                        .offset
                        .go_to_index(*select2, document.bytes_per_line);
                }
            });
            let range = *select1..=*select2;
            if document.binary_file.get(range.clone()).is_some()
                && ui.button("Delete selection").clicked()
            {
                document.binary_file.drain(range.clone());
                *select1 = select1.checked_sub(1).unwrap_or(0);
                *select2 = select1.checked_add(1).unwrap_or(0);
                mark_selection_stale = true;
                mark_stale = true;
            }
            if let Some(selection) = document.binary_file.get(range.clone())
                && ui.button("Open in new document").clicked()
            {
                let doc = Document {
                    binary_file: selection.to_vec(),
                    filename: PathBuf::from("selected.bin"),
                    ..Default::default()
                };
                app.documents.push(doc);
            }
        } else {
            ui.label("No selection");
        }
        app.ui_selection(ui);
        if mark_stale {
            app.stale();
        }
        if mark_selection_stale {
            app.stale_selection();
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
