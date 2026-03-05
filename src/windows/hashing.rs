//! Hashing

use bladvak::eframe::egui;
use bladvak::errors::ErrorManager;
use md5::Md5;
use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha512};

/// Different hashes
#[derive(Debug)]
struct Hashes {
    /// sha1
    sha1: String,
    /// sha256
    sha256: String,
    /// sha512
    sha512: String,
    /// md5
    md5: String,
}

/// Hashing data
#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub(crate) struct Hashing {
    /// is open
    pub(crate) is_open: bool,

    #[serde(skip)]
    /// optional data hashes
    data: Option<Hashes>,
}

impl Hashing {
    /// Create empty Hashing
    pub(crate) fn new() -> Self {
        Self {
            is_open: false,
            ..Self::default()
        }
    }

    /// reset data
    pub(crate) fn reset(&mut self) {
        self.data = None;
    }

    /// Window title
    pub(crate) fn window_title() -> &'static str {
        "Hashing"
    }

    /// Calculate the histogram
    fn calculate_hashes(binary_data: &[u8]) -> Hashes {
        let sha1 = format!("{:x}", Sha1::digest(binary_data));
        let sha256 = format!("{:x}", Sha256::digest(binary_data));
        let sha512 = format!("{:x}", Sha512::digest(binary_data));
        let md5 = format!("{:x}", Md5::digest(binary_data));
        Hashes {
            sha1,
            sha256,
            sha512,
            md5,
        }
    }

    /// Show the histogram ui
    pub(crate) fn ui(
        &mut self,
        binary_data: &[u8],
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
    ) {
        if self.is_open {
            let mut is_open = self.is_open;
            if let Some(data) = &self.data {
                egui::Window::new(Self::window_title())
                    .open(&mut is_open)
                    .vscroll(true)
                    .show(ui.ctx(), |ui| {
                        ui.label("sha1");
                        ui.label(&data.sha1);
                        ui.separator();
                        ui.label("sha256");
                        ui.label(&data.sha256);
                        ui.separator();
                        ui.label("sha512");
                        ui.label(&data.sha512);
                        ui.separator();
                        ui.label("md5");
                        ui.label(&data.md5);
                    });
                self.is_open = is_open;
            } else if binary_data.is_empty() {
                egui::Window::new(Self::window_title())
                    .open(&mut is_open)
                    .vscroll(true)
                    .show(ui.ctx(), |ui| {
                        ui.label("File is empty");
                    });
                self.is_open = is_open;
            } else {
                self.data = Some(Self::calculate_hashes(binary_data));
            }
        }
    }
}
