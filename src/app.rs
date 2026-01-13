//! Wombat App
use bladvak::app::BladvakPanel;
use bladvak::eframe::egui;
use bladvak::eframe::{self, CreationContext};
use bladvak::utils::is_native;
use bladvak::{File, egui_extras};
use bladvak::{
    app::BladvakApp,
    errors::{AppError, ErrorManager},
};
use std::fmt::Debug;
use std::path::PathBuf;

use crate::panels::{FileInfo, FileInfoData, FileSelection};
use crate::windows::{Histogram, WindowsData};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct WombatApp {
    /// Binary file data
    #[serde(skip)]
    pub(crate) binary_file: Vec<u8>,

    /// Filename of the file
    #[serde(skip)]
    pub(crate) filename: PathBuf,

    /// first ascii printable char
    pub(crate) start_ascii_printable: u8,

    /// Bytes per line
    pub(crate) bytes_per_line: usize,

    /// Selection
    pub(crate) selection: Option<(usize, usize)>,

    /// File info
    #[serde(skip)]
    pub(crate) file_format: Option<FileInfoData>,

    /// Windows
    #[serde(skip)]
    pub(crate) windows_data: WindowsData,
}

/// default file (wombat icon)
const LOGO_ASSET: &[u8] = include_bytes!("../assets/icon-1024.png");

impl Default for WombatApp {
    fn default() -> Self {
        let File { data, path } = Self::load_default_file();
        Self {
            binary_file: data,
            filename: path,
            start_ascii_printable: 0x21_u8,
            bytes_per_line: 32,
            selection: None,
            file_format: None,
            windows_data: WindowsData::new(),
        }
    }
}

impl WombatApp {
    /// Called once before the first frame.
    fn new_app(saved_state: Self, cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        egui_extras::install_image_loaders(&cc.egui_ctx);
        saved_state
    }

    /// Load the default file (wombat icon)
    #[must_use]
    pub fn load_default_file() -> File {
        File {
            data: LOGO_ASSET.to_vec(),
            path: PathBuf::from("wombat.png"),
        }
    }
}

impl BladvakApp<'_> for WombatApp {
    fn panel_list(&self) -> Vec<Box<dyn BladvakPanel<App = WombatApp>>> {
        vec![Box::new(FileInfo), Box::new(FileSelection)]
    }

    fn side_panel(&mut self, ui: &mut egui::Ui, func_ui: impl FnOnce(&mut egui::Ui, &mut Self)) {
        egui::Frame::central_panel(&ui.ctx().style()).show(ui, |ui| {
            func_ui(ui, self);
        });
    }

    fn is_side_panel(&self) -> bool {
        true
    }

    fn is_open_button(&self) -> bool {
        true
    }

    fn handle_file(&mut self, file: File) -> Result<(), AppError> {
        self.binary_file = file.data;
        let file_len = self.binary_file.len();
        self.filename = file.path;
        self.file_format = None;
        self.windows_data.histogram = Histogram::new();

        if self.binary_file.is_empty() {
            self.selection = None;
        } else if let Some((select1, select2)) = self.selection.as_mut() {
            if *select1 > file_len {
                *select1 = file_len - 1;
            }
            if *select2 > file_len {
                *select2 = file_len - 1;
            }
        }
        Ok(())
    }

    fn top_panel(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        ui.menu_button("Windows", |ui| {
            if ui.button("Histogram").clicked() {
                self.windows_data.histogram.is_open = true;
            }
            if ui.button("Import").clicked() {
                self.windows_data.importer.is_open = true;
            }
        });
        ui.separator();
        ui.label(format!("File: {}", self.filename.display()));
    }

    fn menu_file(&mut self, _ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {}

    fn central_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_central_panel(ui, error_manager);
        self.ui_windows(ui, error_manager);
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
    ) -> Result<Self, AppError> {
        if is_native() && args.len() > 1 {
            let path = &args[1];
            let bytes = std::fs::read(path)?;
            let mut app = Self::new_app(saved_state, cc);
            app.binary_file = bytes;
            app.filename = PathBuf::from(path);
            Ok(app)
        } else {
            Ok(Self::new_app(saved_state, cc))
        }
    }
}
