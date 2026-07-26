//! co64 atom

use bladvak::eframe::egui;

/// co64 atom
#[derive(Debug)]
pub struct Co64Box {
    /// 64-bit chunk offsets
    pub chunk_offsets: Vec<u64>,
}

impl Co64Box {
    /// parse co64
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
            chunk_offsets.push(u64::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]));
            offset += 8;
        }

        Some(Self { chunk_offsets })
    }

    /// show co64
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
