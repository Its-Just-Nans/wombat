//! stts atom

use bladvak::eframe::egui;

/// stts atom
#[derive(Debug)]
pub struct SttsEntry {
    /// sample count
    pub sample_count: u32,
    /// sample delta
    pub sample_delta: u32,
}

/// stts box
#[derive(Debug)]
pub struct SttsBox {
    /// list of entries
    pub entries: Vec<SttsEntry>,
}

impl SttsBox {
    /// parset stts atom
    #[allow(clippy::unnecessary_wraps)]
    pub fn parse(data: &[u8]) -> Option<Self> {
        let mut offset = 0;

        // version + flags
        offset += 4;

        let entry_count = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        let mut entries = Vec::with_capacity(entry_count as usize);

        for _ in 0..entry_count {
            let sample_count = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;

            let sample_delta = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;

            entries.push(SttsEntry {
                sample_count,
                sample_delta,
            });
        }

        Some(Self { entries })
    }

    /// show struct
    pub(crate) fn show(&self, ui: &mut egui::Ui) {
        for one_entry in &self.entries {
            ui.label(format!("Sample count: {}", one_entry.sample_count));
            ui.label(format!("Sample delta: {}", one_entry.sample_delta));
            ui.separator();
        }
    }
}
