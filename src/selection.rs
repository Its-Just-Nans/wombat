//! Selection

use bladvak::{
    ErrorManager,
    app::BladvakPanel,
    eframe::egui::{self, Color32, Theme},
};

use crate::{WombatApp, app::Accent};

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
        if app.binary_file.is_empty() {
            app.selection.range = None;
        }
        let mut mark_stale = false;
        if let Some((select1, select2)) = app.selection.range.as_mut() {
            ui.horizontal(|ui| {
                ui.label("Selection");
                let color_to_edit = if ui.ctx().theme() == Theme::Light {
                    &mut app.selection.color.0
                } else {
                    &mut app.selection.color.1
                };
                ui.color_edit_button_srgba(color_to_edit);
            });
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(select1).range(0..=*select2));
                let max = if app.binary_file.is_empty() {
                    0
                } else {
                    app.binary_file.len() - 1
                };
                ui.label("->");
                ui.add(egui::DragValue::new(select2).range(*select1..=max));
            });
            let range = *select1..=*select2;
            if app.binary_file.get(range.clone()).is_some()
                && ui.button("Delete selection").clicked()
            {
                app.binary_file.drain(range.clone());
                *select2 = select1.checked_sub(1).unwrap_or(0);
                mark_stale = true;
            }
        } else {
            ui.label("No selection");
        }
        if let Some((select1, select2)) = app.selection.range {
            if select1 == select2
                && let Some(current) = app.binary_file.get(select1)
            {
                ui.separator();
                ui.label(format!("byte at index {select1}"));
                app.ui_table_u8(ui, *current, &Accent::Hex);
            } else {
                let nb_selected = select2.checked_sub(select1).map_or(0, |d| d as u64 + 1);
                ui.label(format!("{nb_selected} bytes selected"));
                let range = select1..=select2;

                if nb_selected == 2
                    && let Some(slice) = app.binary_file.get(range.clone())
                    && let Ok(bytes) = <[u8; 2]>::try_from(slice)
                {
                    WombatApp::ui_table_u16(ui, bytes);
                }

                if nb_selected == 4
                    && let Some(slice) = app.binary_file.get(range.clone())
                    && let Ok(bytes) = <[u8; 4]>::try_from(slice)
                {
                    WombatApp::ui_table_u32(ui, bytes);
                }
            }
            if mark_stale {
                app.stale();
            }
            if app.binary_file.is_empty() {
                app.selection.range = None;
            }
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
