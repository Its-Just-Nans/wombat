//! stco atom

use bladvak::eframe::egui;

/// stco atom
#[derive(Debug)]
pub struct StcoBox {
    /// chunk offsets
    pub chunk_offsets: Vec<u32>,
}

impl StcoBox {
    /// parse the stco
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

        let mut chunk_offsets = Vec::with_capacity(entry_count as usize);

        for _ in 0..entry_count {
            chunk_offsets.push(u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]));
            offset += 4;
        }

        Some(Self { chunk_offsets })
    }

    /// Show struct
    pub(crate) fn show(&self, ui: &mut egui::Ui) {
        let str_rep = self
            .chunk_offsets
            .iter()
            .map(|one_u32| format!("{one_u32}"))
            .collect::<Vec<String>>()
            .join(", ");
        ui.label(format!("Chunk offsets: {str_rep}"));
    }
}
