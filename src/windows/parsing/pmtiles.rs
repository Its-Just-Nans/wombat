//! PM tiles format

use std::ops::RangeInclusive;

use bladvak::eframe::egui;

/// pmtiles data
#[derive(Debug)]
pub(crate) struct PmTilesData {
    /// header
    header: PmTilesHeader,
}

/// Position
#[derive(Debug)]
pub(crate) struct Position {
    /// latitude
    lat: i32,
    /// longitude
    lon: i32,
}

impl Position {
    /// decode the u64
    fn decode(pos: [u8; 8]) -> Self {
        Self {
            lon: i32::from_le_bytes([pos[0], pos[1], pos[2], pos[3]]) / 10_000_000,
            lat: i32::from_le_bytes([pos[4], pos[5], pos[6], pos[7]]) / 10_000_000,
        }
    }
}

/// pmtiles header
#[derive(Debug)]
pub(crate) struct PmTilesHeader {
    /// Root Directory Offset
    root_directory_offset: u64,
    /// Root Directory Length
    root_directory_len: u64,
    /// Metadata Offset
    metadata_offset: u64,
    /// Metadata Length
    metadata_len: u64,
    /// Leaf Directories Offset
    leaf_directories_offset: u64,
    /// Leaf Directories Length
    leaf_directories_len: u64,
    /// Tile Data Offset
    tile_data_offset: u64,
    /// Tile Data Offset
    tile_data_len: u64,
    /// Num of Addressed Tiles
    num_addressed_tiles: u64,
    /// Num of Addressed Tiles
    number_tiles_entries: u64,
    /// Number of Tile Contents
    number_tiles_content: u64,
    /// Clustered
    clustered: u8,
    /// Internat Compression
    internal_compression: u8,
    /// Tile compression
    tile_compression: u8,
    /// Tile type
    tile_type: u8,
    /// Min zoom
    min_zoom: u8,
    /// Max zoom
    max_zoom: u8,
    /// Min Position
    min_position: Position,
    /// Max Position
    max_position: Position,
    /// Center zoom
    center_zoom: u8,
    /// Center Position
    center_position: Position,
}

/// png signature
const PMTILES_SIGNATURE: &[u8; 7] = b"PMTiles";

impl PmTilesData {
    /// parse the data
    pub(crate) fn parse(binary_data: &[u8]) -> Option<Self> {
        if binary_data[0..7] != PMTILES_SIGNATURE[..] {
            return None;
        }
        if binary_data[7] != 3 {
            return None;
        }
        let header = PmTilesHeader {
            root_directory_offset: u64::from_le_bytes(binary_data.get(8..16)?.try_into().ok()?),
            root_directory_len: u64::from_le_bytes(binary_data.get(16..24)?.try_into().ok()?),
            metadata_offset: u64::from_le_bytes(binary_data.get(24..32)?.try_into().ok()?),
            metadata_len: u64::from_le_bytes(binary_data.get(32..40)?.try_into().ok()?),
            leaf_directories_offset: u64::from_le_bytes(binary_data.get(40..48)?.try_into().ok()?),
            leaf_directories_len: u64::from_le_bytes(binary_data.get(48..56)?.try_into().ok()?),
            tile_data_offset: u64::from_le_bytes(binary_data.get(56..64)?.try_into().ok()?),
            tile_data_len: u64::from_le_bytes(binary_data.get(64..72)?.try_into().ok()?),
            num_addressed_tiles: u64::from_le_bytes(binary_data.get(72..80)?.try_into().ok()?),
            number_tiles_entries: u64::from_le_bytes(binary_data.get(80..88)?.try_into().ok()?),
            number_tiles_content: u64::from_le_bytes(binary_data.get(88..96)?.try_into().ok()?),
            clustered: *binary_data.get(96)?,
            internal_compression: *binary_data.get(97)?,
            tile_compression: *binary_data.get(98)?,
            tile_type: *binary_data.get(99)?,
            min_zoom: *binary_data.get(100)?,
            max_zoom: *binary_data.get(101)?,
            min_position: Position::decode(binary_data.get(102..110)?.try_into().ok()?),
            max_position: Position::decode(binary_data.get(110..118)?.try_into().ok()?),
            center_zoom: *binary_data.get(118)?,
            center_position: Position::decode(binary_data.get(118..126)?.try_into().ok()?),
        };
        Some(PmTilesData { header })
    }
}

impl PmTilesHeader {
    /// Show the header
    #[allow(clippy::too_many_lines)]
    pub(crate) fn ui(&self, ui: &mut egui::Ui) -> Option<RangeInclusive<usize>> {
        let mut return_range = None;
        ui.horizontal(|ui| {
            ui.label("Root Directory Offset");
            ui.label(self.root_directory_offset.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(8..=15);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Root Directory Length");
            ui.label(self.root_directory_len.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(16..=23);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Metaself.Offset");
            ui.label(self.metadata_offset.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(24..=31);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Metaself.Length");
            ui.label(self.metadata_len.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(32..=39);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Leaf Directories Offset");
            ui.label(self.leaf_directories_offset.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(40..=47);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Leaf Directories Length");
            ui.label(self.leaf_directories_len.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(48..=55);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Tile Data Offset");
            ui.label(self.tile_data_offset.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(56..=63);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Tile Data Length");
            ui.label(self.tile_data_len.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(64..=71);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Num of Addressed Tiles");
            ui.label(self.num_addressed_tiles.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(72..=79);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Number of Tile Entries");
            ui.label(self.number_tiles_entries.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(80..=87);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Number of Tile Contents");
            ui.label(self.number_tiles_content.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(88..=95);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Clustered");
            ui.label(self.clustered.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(96..=96);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Internal Compression");
            ui.label(self.internal_compression.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(97..=97);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Tile Compression");
            ui.label(self.tile_compression.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(98..=98);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Tile Type");
            ui.label(self.tile_type.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(99..=99);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Min Zoom");
            ui.label(self.min_zoom.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(100..=100);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Max Zoom");
            ui.label(self.max_zoom.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(101..=101);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Min Position");
            ui.label(format!(
                "lon: {}, lat: {}",
                self.min_position.lon, self.min_position.lat
            ));
            if ui.button("Show").clicked() {
                return_range = Some(102..=109);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Max Position");
            ui.label(format!(
                "lon: {}, lat: {}",
                self.max_position.lon, self.max_position.lat
            ));
            if ui.button("Show").clicked() {
                return_range = Some(110..=117);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Center Zoom");
            ui.label(self.center_zoom.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(118..=118);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Center Position");
            ui.label(format!(
                "lon: {}, lat: {}",
                self.center_position.lon, self.center_position.lat
            ));
            if ui.button("Show").clicked() {
                return_range = Some(119..=126);
            }
        });
        return_range
    }
}

/// Show the pmtiles
pub(crate) fn show_pmtiles_ui(
    ui: &mut egui::Ui,
    opt_pmtiles: Option<&PmTilesData>,
) -> Option<RangeInclusive<usize>> {
    let Some(data) = opt_pmtiles else {
        ui.label("Failed to parse pmtiles");
        return None;
    };
    let mut return_range = None;
    ui.horizontal(|ui| {
        ui.label("PMTiles");
        if ui.button("Show").clicked() {
            return_range = Some(0..=6);
        }
    });
    ui.horizontal(|ui| {
        ui.label("Version");
        if ui.button("Show").clicked() {
            return_range = Some(7..=7);
        }
    });
    ui.collapsing("Header", |ui| {
        if let Some(range) = data.header.ui(ui) {
            return_range = Some(range);
        }
    });
    return_range
}
