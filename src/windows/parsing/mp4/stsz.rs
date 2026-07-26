//! stsz atom

use bladvak::eframe::egui;

/// stsz atom
#[derive(Debug)]
#[allow(clippy::struct_field_names)]
pub struct StszBox {
    /// sample size
    pub sample_size: u32,
    /// sample count
    pub sample_count: u32,
    /// sample sizes
    pub sample_sizes: Vec<u32>,
}

impl StszBox {
    /// parse stsz
    #[allow(clippy::unnecessary_wraps)]
    pub fn parse(data: &[u8]) -> Option<StszBox> {
        let mut offset = 0;

        // version + flags
        offset += 4;

        let sample_size = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        let sample_count = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        let mut sample_sizes = Vec::with_capacity(sample_count as usize);

        if sample_size == 0 {
            for _ in 0..sample_count {
                let size = u32::from_be_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                offset += 4;

                sample_sizes.push(size);
            }
        }

        Some(StszBox {
            sample_size,
            sample_count,
            sample_sizes,
        })
    }

    /// Show data
    pub(crate) fn show(&self, ui: &mut egui::Ui) {
        ui.label(format!("Sample size: {}", self.sample_size));
        ui.label(format!("Sample count: {}", self.sample_count));
        let str_rep = self
            .sample_sizes
            .iter()
            .map(|one_u32| format!("{one_u32}"))
            .collect::<Vec<String>>()
            .join(", ");
        ui.label(format!("Sample sizes: {str_rep}"));
    }
}
