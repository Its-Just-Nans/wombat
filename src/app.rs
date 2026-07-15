//! Wombat App

use bladvak::app::BladvakPanel;
use bladvak::eframe::CreationContext;
use bladvak::eframe::egui;
use bladvak::utils::is_native;
use bladvak::{File, egui_extras};
use bladvak::{
    app::BladvakApp,
    errors::{AppError, ErrorManager},
    utils::Documents,
};
use std::fmt::Debug;
use std::path::PathBuf;

use crate::display_settings::DisplaySettings;
use crate::document::Document;
use crate::panels::FileInfo;
use crate::selection::PanelSelection;
use crate::windows::exporter::Exporter;
use crate::windows::importer::Importer;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct WombatApp {
    /// Documents
    #[serde(skip)]
    pub(crate) documents: Documents<Document>,
    /// Display settings
    pub(crate) display_settings: DisplaySettings,
    /// Visual debug
    pub(crate) visual_debug: bool,
    /// importer
    pub(crate) importer: Importer,
    /// exporter
    pub(crate) exporter: Exporter,
}

/// default file (wombat icon)
const LOGO_ASSET: &[u8] = include_bytes!("../assets/icon-1024.png");

impl Default for WombatApp {
    fn default() -> Self {
        let File { data, path } = Self::load_default_file();
        let document = Document {
            binary_file: data,
            filename: path,
            ..Default::default()
        };
        let mut documents = Documents::default();
        documents.push(document);
        Self {
            documents,
            display_settings: DisplaySettings::default(),
            visual_debug: false,
            importer: Importer::new(),
            exporter: Exporter::new(),
        }
    }
}

impl WombatApp {
    /// Load the default file (wombat icon)
    #[must_use]
    pub fn load_default_file() -> File {
        File {
            data: LOGO_ASSET.to_vec(),
            path: PathBuf::from("wombat.png"),
        }
    }

    /// Mark data as stale
    pub(crate) fn stale(&mut self) {
        self.importer.reset();
        self.exporter.reset();
        if let Some(document) = self.documents.get_current_doc_mut() {
            document.stale();
            document.windows_data.reset();
        }
    }

    /// Mark selection as stale
    pub(crate) fn stale_selection(&mut self) {
        if let Some(document) = self.documents.get_current_doc_mut() {
            document.windows_data.selection_stale();
        }
    }
}

impl BladvakApp<'_> for WombatApp {
    fn panel_list(&self) -> Vec<Box<dyn BladvakPanel<App = WombatApp>>> {
        vec![Box::new(FileInfo), Box::new(PanelSelection)]
    }

    fn side_panel(&mut self, ui: &mut egui::Ui, func_ui: impl FnOnce(&mut egui::Ui, &mut Self)) {
        egui::Frame::central_panel(&ui.ctx().global_style()).show(ui, |ui| {
            func_ui(ui, self);
        });
    }

    fn is_side_panel(&self) -> bool {
        self.documents.is_some()
    }

    fn is_open_button(&self) -> bool {
        true
    }

    fn handle_file(&mut self, file: File) -> Result<(), AppError> {
        let mut document = Document {
            binary_file: file.data,
            filename: file.path,
            ..Default::default()
        };
        let file_len = document.binary_file.len();
        self.stale();
        self.stale_selection();

        if document.binary_file.is_empty() {
            document.selection.reset();
        } else if let Some((select1, select2)) = document.selection.range.as_mut() {
            if *select1 > file_len {
                *select1 = file_len - 1;
            }
            if *select2 > file_len {
                *select2 = file_len - 1;
            }
        }
        self.documents.push(document);
        Ok(())
    }

    fn top_panel(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        ui.separator();
        if let Some(document) = self.documents.get_current_doc_mut() {
            ui.menu_button("Windows", |ui| {
                document.windows_data.ui_top_bar(ui);
            });
            ui.separator();
        }
        self.documents.show_file_list(ui);
        if !self.documents.is_some() {
            self.exporter.is_open = false;
        }
    }

    fn menu_file(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        ui.toggle_value(&mut self.importer.is_open, "Import");
        if self.documents.get_current_doc_mut().is_some() {
            ui.toggle_value(&mut self.exporter.is_open, "Export");
        }
    }

    fn central_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_central_panel(ui, error_manager);
        self.ui_windows(ui, error_manager);
        if let Some(document) = self.documents.get_current_doc_mut() {
            self.exporter.ui(
                &document.binary_file,
                &document.filename,
                &document.selection,
                ui,
                error_manager,
            );
        }
        if let Some(data) = self.importer.ui(ui, error_manager)
            && let Err(e) = self.handle_file(File {
                data,
                path: PathBuf::from("imported.bin"),
            })
        {
            error_manager.add_error(e);
        }
    }

    fn name() -> String {
        env!("CARGO_PKG_NAME").to_string()
    }

    fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn repo_url() -> String {
        "https://github.com/Its-Just-Nans/wombat".to_string()
    }

    fn icon() -> &'static [u8] {
        &include_bytes!("../assets/icon-256.png")[..]
    }

    fn try_new_with_args(
        saved_state: Self,
        cc: &CreationContext<'_>,
        args: &[String],
        _error_manager: &mut ErrorManager,
    ) -> Result<Self, AppError> {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        egui_extras::install_image_loaders(&cc.egui_ctx);

        if is_native() && args.len() > 1 {
            use std::fs;
            let mut app = saved_state;
            app.documents.clear();
            for one_path in &args[1..] {
                let absolute_path = fs::canonicalize(one_path)
                    .map_err(|e| format!("Unable to canonicalize path '{one_path}': {e}"))?;
                let bytes = std::fs::read(&absolute_path).map_err(|e| {
                    format!("Unable to read file '{}': {e}", absolute_path.display())
                })?;
                let document = Document {
                    binary_file: bytes,
                    filename: absolute_path,
                    ..Default::default()
                };
                app.documents.push(document);
            }
            Ok(app)
        } else {
            Ok(saved_state)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::WombatApp;

    #[test]
    fn test_to_ascii() {
        let wombat = WombatApp::default();

        for i in 0u8..=u8::MAX {
            let text = wombat.display_settings.ascii_to_string(i);
            if i > 127 {
                // extended ASCII
                assert_eq!(text, "extended ASCII", "{i}");
            } else {
                if i == 32 {
                    // space
                }
                assert_ne!(text, "extended ASCII", "{i}"); // not equal
            }
        }
    }
}
