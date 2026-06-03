//! Wombat App

use bladvak::app::BladvakPanel;
use bladvak::eframe::CreationContext;
use bladvak::eframe::egui;
use bladvak::utils::is_native;
use bladvak::{File, egui_extras};
use bladvak::{
    app::BladvakApp,
    errors::{AppError, ErrorManager},
};
use std::fmt::Debug;
use std::ops::RangeInclusive;
use std::path::PathBuf;

use crate::panels::{FileInfo, FileInfoData};
use crate::selection::{PanelSelection, Selection};
use crate::windows::WindowsData;

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

    /// Display settings
    pub(crate) display_settings: DisplaySettings,
    /// Selection
    pub(crate) selection: Selection,

    /// File info
    #[serde(skip)]
    pub(crate) file_format: Option<FileInfoData>,

    /// Windows
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
            display_settings: DisplaySettings::default(),
            selection: Selection::default(),
            file_format: None,
            windows_data: WindowsData::new(),
        }
    }
}

/// Display setting
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct DisplaySettings {
    /// Least Significant Bit
    pub(crate) display_lsb: bool,
    /// Bytes per line
    pub(crate) bytes_per_line: usize,
    /// limit ascii
    pub(crate) limit_to_base_ascii: bool,
    /// show color picket
    pub(crate) show_color_picker: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            display_lsb: false,
            bytes_per_line: 32,
            limit_to_base_ascii: true,
            show_color_picker: true,
        }
    }
}

/// Accent type
#[derive(Debug, PartialEq)]
pub(crate) enum Accent {
    /// decimal
    Decimal,
    /// hexe
    Hex,
    /// octal
    Octal,
    /// binary
    Binary,
    /// ascii
    Ascii,
}

impl WombatApp {
    /// start ASCII printable char (after space)
    pub(crate) const RANGE_ASCII_PRINTABLE: RangeInclusive<u8> = 0x21_u8..=0x7E;

    /// Load the default file (wombat icon)
    #[must_use]
    pub fn load_default_file() -> File {
        File {
            data: LOGO_ASSET.to_vec(),
            path: PathBuf::from("wombat.png"),
        }
    }

    /// ascii u8 to string
    pub(crate) fn ascii_to_string(&self, c: u8) -> String {
        match c {
            0x0 => "NUL (Null character)".to_string(),
            0x01 => "SOH (Start of Heading)".to_string(),
            0x02 => "STX (Start of Text)".to_string(),
            0x03 => "ETX (End of Text)".to_string(),
            0x04 => "EOT (End of Transmission)".to_string(),
            0x05 => "ENQ (Enquiry)".to_string(),
            0x06 => "ACK (Acknowledge)".to_string(),
            0x07 => "BEL (Bell, Alert)".to_string(),
            0x08 => "BS (Backspace)".to_string(),
            0x09 => "HT (Horizontal Tab)".to_string(),
            0x0A => "LF (Line Feed)".to_string(),
            0x0B => "VT (Vertical Tabulation)".to_string(),
            0x0C => "FF (Form Feed)".to_string(),
            0x0D => "CR (Carriage Return)".to_string(),
            0x0E => "SO (Shift Out)".to_string(),
            0x0F => "SI (Shift In)".to_string(),
            0x10 => "DLE (Data Link Escape)".to_string(),
            0x11 => "DC1 (Device Control One (XON))".to_string(),
            0x12 => "DC2 (Device Control Two)".to_string(),
            0x13 => "DC3 (Device Control Three (XOFF))".to_string(),
            0x14 => "DC4 (Device Control Four)".to_string(),
            0x15 => "NAK (Negative Acknowledge)".to_string(),
            0x16 => "SYN (Synchronous Idle)".to_string(),
            0x17 => "ETB (End of Transmission Block)".to_string(),
            0x18 => "CAN (Cancel)".to_string(),
            0x19 => "EM (End of medium)".to_string(),
            0x1A => "SUB (Substitute)".to_string(),
            0x1B => "ESC (Escape)".to_string(),
            0x1C => "FS (File Separator)".to_string(),
            0x1D => "GS (Group Separator)".to_string(),
            0x1E => "RS (Record Separator)".to_string(),
            0x1F => "US (Unit Separator)".to_string(),
            0x20 => "SP (Space)".to_string(),
            x if Self::RANGE_ASCII_PRINTABLE.contains(&x) => (c as char).to_string(),
            0x7F => "DEL (Delete)".to_string(),
            c => {
                if self.display_settings.limit_to_base_ascii {
                    "extended ASCII".to_string()
                } else {
                    format!("{} (extended ASCII)", c as char)
                }
            }
        }
    }

    /// Mark data as stale
    pub(crate) fn stale(&mut self) {
        self.file_format = None;
        self.windows_data.reset();
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
        true
    }

    fn is_open_button(&self) -> bool {
        true
    }

    fn handle_file(&mut self, file: File) -> Result<(), AppError> {
        self.binary_file = file.data;
        let file_len = self.binary_file.len();
        self.filename = file.path;
        self.stale();

        if self.binary_file.is_empty() {
            self.selection.reset();
        } else if let Some((select1, select2)) = self.selection.range.as_mut() {
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
            self.windows_data.ui_top_bar(ui);
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
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        egui_extras::install_image_loaders(&cc.egui_ctx);

        if is_native() && args.len() > 1 {
            let path = &args[1];
            let bytes = std::fs::read(path)?;
            let mut app = saved_state;
            app.binary_file = bytes;
            app.filename = PathBuf::from(path);
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
            let text = wombat.ascii_to_string(i);
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
