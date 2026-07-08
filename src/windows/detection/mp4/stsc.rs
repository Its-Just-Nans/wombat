//! stsc atom

use bladvak::eframe::egui;

/// stsc entry
#[derive(Debug)]
pub struct StscEntry {
    /// firt chunk
    pub first_chunk: u32,
    /// samples per chunk
    pub samples_per_chunk: u32,
    /// sample description
    pub sample_description_index: u32,
}

/// stsc atom
#[derive(Debug)]
pub struct StscBox {
    /// entries
    pub entries: Vec<StscEntry>,
}

impl StscBox {
    /// parse stsc
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
            let first_chunk = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;

            let samples_per_chunk = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;

            let sample_description_index = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;

            entries.push(StscEntry {
                first_chunk,
                samples_per_chunk,
                sample_description_index,
            });
        }

        Some(Self { entries })
    }

    /// show stcs
    pub(crate) fn show(&self, ui: &mut egui::Ui) {
        for one_entry in &self.entries {
            ui.label(format!("First chunk: {}", one_entry.first_chunk));
            ui.label(format!("Sample per chunk: {}", one_entry.samples_per_chunk));
            ui.label(format!(
                "Sample description index: {}",
                one_entry.sample_description_index
            ));
            ui.separator();
        }
    }
}
