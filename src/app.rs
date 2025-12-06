//! Wombat App
use bladvak::eframe::egui;
use bladvak::eframe::{self, CreationContext};
use bladvak::egui_extras;
use bladvak::{
    app::BladvakApp,
    errors::{AppError, ErrorManager},
};
use std::fmt::Debug;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct WombatApp {
    /// Binary file data
    #[serde(skip)]
    pub binary_file: Vec<u8>,

    /// Bytes per line
    pub bytes_per_line: usize,

    /// Sidebar as window
    pub sidebar_as_window: bool,
}

/// default file (wombat icon)
const LOGO_ASSET: &[u8] = include_bytes!("../assets/icon-1024.png");

impl Default for WombatApp {
    fn default() -> Self {
        let binary_file = Self::load_default_file();
        Self {
            binary_file,
            bytes_per_line: 32,
            sidebar_as_window: false,
        }
    }
}

impl WombatApp {
    /// Called once before the first frame.
    fn new_app(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        egui_extras::install_image_loaders(&cc.egui_ctx);
        bladvak::utils::get_saved_app_state::<Self>(cc)
    }
    /// Create a new Wombat App with an image
    /// # Errors
    /// Return error if fail to load image
    pub fn new_app_with_args(cc: &CreationContext<'_>, args: &[String]) -> Result<Self, AppError> {
        if args.len() > 1 {
            let path = &args[1];
            let bytes = std::fs::read(path)?;
            let mut app = Self::new_app(cc);
            app.binary_file = bytes;
            Ok(app)
        } else {
            Ok(WombatApp::new_app(cc))
        }
    }

    /// Load the default file (wombat icon)
    pub fn load_default_file() -> Vec<u8> {
        LOGO_ASSET.to_vec()
    }
}

impl BladvakApp for WombatApp {
    fn settings(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        ui.separator();
        ui.checkbox(&mut self.sidebar_as_window, "Viewer settings windows");
        ui.separator();
        if ui.button("Default file").clicked() {
            let binary = Self::load_default_file();
            if let Err(err) = self.handle_file(&binary) {
                error_manager.add_error(err);
            }
        }
    }

    fn is_side_panel(&self) -> bool {
        !self.sidebar_as_window
    }

    fn is_open_button(&self) -> bool {
        true
    }

    fn handle_file(&mut self, bytes: &[u8]) -> Result<(), AppError> {
        self.binary_file = bytes.to_vec();
        Ok(())
    }

    fn top_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_top_panel(ui, error_manager);
    }

    fn menu_file(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_menu_file(ui, error_manager)
    }

    fn central_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.app_central_panel(ui, error_manager)
    }

    fn name() -> String {
        env!("CARGO_PKG_NAME").to_string()
    }

    fn repo_url() -> String {
        "https://github.com/Its-Just-Nans/wombat".to_string()
    }

    fn side_panel(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        egui::Frame::central_panel(&ui.ctx().style()).show(ui, |parent_ui| {
            self.app_side_panel(parent_ui, error_manager)
        });
    }

    fn new(cc: &eframe::CreationContext<'_>) -> Result<Self, AppError> {
        Ok(WombatApp::new_app(cc))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn new_with_args(cc: &CreationContext<'_>, args: &[String]) -> Result<Self, AppError> {
        WombatApp::new_app_with_args(cc, args)
    }
}
