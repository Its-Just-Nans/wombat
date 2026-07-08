//! stsd atom

use bladvak::eframe::egui;

/// stsd entry
#[derive(Debug)]
pub struct StsdEntry {
    /// codec
    pub codec: [u8; 4],
    /// data index
    pub data: Vec<u8>,
}

/// stsd atom
#[derive(Debug)]
pub struct StsdBox {
    /// entries
    pub entries: Vec<StsdEntry>,
}

impl StsdBox {
    /// parse
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
            let size = u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            let codec = [
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ];
            offset += 4;

            let payload_size = size.checked_sub(8)?;
            let payload = data.get(offset..offset + payload_size)?;
            offset += payload_size;

            entries.push(StsdEntry {
                codec,
                data: payload.to_vec(),
            });
            offset += payload_size;
        }

        Some(Self { entries })
    }

    /// show
    pub(crate) fn show(&self, ui: &mut egui::Ui) {
        for one_entry in &self.entries {
            ui.label(format!("Codec: {:?}", one_entry.codec));
            ui.label(format!("Data: {:?}", one_entry.data));
            ui.separator();
        }
    }
}
