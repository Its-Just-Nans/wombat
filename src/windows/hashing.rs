//! Hashing

use bladvak::eframe::egui;
use bladvak::errors::ErrorManager;

use base16ct::HexDisplay;
use md5::Md5;
use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha512};

use crate::selection::Selection;

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

/// Different part
#[derive(Debug)]
struct HashData {
    /// file
    file: Hashes,
    /// selection hashes
    selection: Option<Hashes>,
}

/// Hashing data
#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub(crate) struct Hashing {
    /// is open
    pub(crate) is_open: bool,

    #[serde(skip)]
    /// optional data hashes
    data: Option<HashData>,
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

    /// Mark selection stale
    pub(crate) fn selection_stale(&mut self) {
        if let Some(data) = &mut self.data {
            data.selection = None;
        }
    }

    /// Window title
    pub(crate) fn window_title() -> &'static str {
        "Hashing"
    }

    /// Calculate the hashing
    fn calculate_hashes(binary_data: &[u8]) -> Hashes {
        let sha1 = format!("{:x}", HexDisplay(&Sha1::digest(binary_data)));
        let sha256 = format!("{:x}", HexDisplay(&Sha256::digest(binary_data)));
        let sha512 = format!("{:x}", HexDisplay(&Sha512::digest(binary_data)));
        let md5 = format!("{:x}", HexDisplay(&Md5::digest(binary_data)));
        Hashes {
            sha1,
            sha256,
            sha512,
            md5,
        }
    }

    /// Calculate hashes of the selection
    fn calculate_hashes_selection(binary_data: &[u8], selection: &Selection) -> Option<Hashes> {
        if let Some(range) = selection.range
            && let Some(slice) = binary_data.get(range.0..=range.1)
        {
            Some(Self::calculate_hashes(slice))
        } else {
            None
        }
    }

    /// Calculate the hashing data
    fn calculate_hash_data(binary_data: &[u8], selection: &Selection) -> HashData {
        HashData {
            file: Self::calculate_hashes(binary_data),
            selection: Self::calculate_hashes_selection(binary_data, selection),
        }
    }

    /// Show the hashing ui
    pub(crate) fn ui(
        &mut self,
        binary_data: &[u8],
        selection: &Selection,
        ui: &mut egui::Ui,
        _error_manager: &mut ErrorManager,
    ) {
        if self.is_open {
            let mut is_open = self.is_open;
            if let Some(data) = &mut self.data {
                if selection.range.is_some() && data.selection.is_none() {
                    data.selection = Self::calculate_hashes_selection(binary_data, selection);
                }
                egui::Window::new(Self::window_title())
                    .open(&mut is_open)
                    .vscroll(true)
                    .show(ui.ctx(), |ui| {
                        ui.label("File hashes:");
                        ui.label("sha1");
                        ui.label(&data.file.sha1);
                        ui.separator();
                        ui.label("sha256");
                        ui.label(&data.file.sha256);
                        ui.separator();
                        ui.label("sha512");
                        ui.label(&data.file.sha512);
                        ui.separator();
                        ui.label("md5");
                        ui.label(&data.file.md5);

                        if let Some(select) = &data.selection {
                            ui.separator();
                            ui.label("Selection hashes");
                            ui.label("sha1");
                            ui.label(&select.sha1);
                            ui.separator();
                            ui.label("sha256");
                            ui.label(&select.sha256);
                            ui.separator();
                            ui.label("sha512");
                            ui.label(&select.sha512);
                            ui.separator();
                            ui.label("md5");
                            ui.label(&select.md5);
                        }
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
                self.data = Some(Self::calculate_hash_data(binary_data, selection));
            }
        }
    }
}
