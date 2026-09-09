//! PM tiles format

use std::ops::RangeInclusive;

use bladvak::eframe::egui;
use flate2::read::GzDecoder;
use serde_json::Value;

/// pmtiles data
#[derive(Debug)]
pub(crate) struct PmTilesData {
    /// header
    header: PmTilesHeader,
    /// metadata
    metadata: Result<Value, String>,
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

/// read byte
fn read_bytes(binary_data: &[u8], start: usize) -> Result<[u8; 8], String> {
    let bytes = binary_data
        .get(start..start + 8)
        .ok_or_else(|| format!("Not enough data at offset {start}"))?;

    let bytes: [u8; 8] = bytes
        .try_into()
        .map_err(|_| format!("Failed to read u64 at offset {start}"))?;

    Ok(bytes)
}

/// read a u64 le
fn read_u64_le(binary_data: &[u8], start: usize) -> Result<u64, String> {
    let bytes = read_bytes(binary_data, start)?;
    Ok(u64::from_le_bytes(bytes))
}

impl PmTilesData {
    /// parse the data
    pub(crate) fn parse(binary_data: &[u8]) -> Result<Self, String> {
        if binary_data[0..7] != PMTILES_SIGNATURE[..] {
            return Err("Wrong PMTiles Signature".to_string());
        }
        if binary_data[7] != 3 {
            return Err("Wrong PMTiles version".to_string());
        }
        let header = PmTilesHeader {
            root_directory_offset: read_u64_le(binary_data, 8)?,
            root_directory_len: read_u64_le(binary_data, 16)?,
            metadata_offset: read_u64_le(binary_data, 24)?,
            metadata_len: read_u64_le(binary_data, 32)?,
            leaf_directories_offset: read_u64_le(binary_data, 40)?,
            leaf_directories_len: read_u64_le(binary_data, 48)?,
            tile_data_offset: read_u64_le(binary_data, 56)?,
            tile_data_len: read_u64_le(binary_data, 64)?,
            num_addressed_tiles: read_u64_le(binary_data, 72)?,
            number_tiles_entries: read_u64_le(binary_data, 80)?,
            number_tiles_content: read_u64_le(binary_data, 88)?,
            clustered: *binary_data
                .get(96)
                .ok_or_else(|| format!("Not enough data at offset {}", 96))?,
            internal_compression: *binary_data
                .get(97)
                .ok_or_else(|| format!("Not enough data at offset {}", 97))?,
            tile_compression: *binary_data
                .get(98)
                .ok_or_else(|| format!("Not enough data at offset {}", 98))?,
            tile_type: *binary_data
                .get(99)
                .ok_or_else(|| format!("Not enough data at offset {}", 99))?,
            min_zoom: *binary_data
                .get(100)
                .ok_or_else(|| format!("Not enough data at offset {}", 100))?,
            max_zoom: *binary_data
                .get(101)
                .ok_or_else(|| format!("Not enough data at offset {}", 101))?,
            min_position: Position::decode(read_bytes(binary_data, 102)?),
            max_position: Position::decode(read_bytes(binary_data, 110)?),
            center_zoom: *binary_data
                .get(118)
                .ok_or_else(|| format!("Not enough data at offset {}", 118))?,
            center_position: Position::decode(read_bytes(binary_data, 118)?),
        };

        #[allow(clippy::cast_possible_truncation)]
        let metadata_start = header.metadata_offset as usize;
        #[allow(clippy::cast_possible_truncation)]
        let metadata_end =
            ((header.metadata_offset + header.metadata_len) as usize).saturating_sub(1);
        let metadata = match binary_data.get(metadata_start..=metadata_end) {
            Some(metadata) => {
                let raw_metadata = if header.internal_compression == 0 {
                    match std::str::from_utf8(metadata) {
                        Ok(res) => Ok(res.to_string()),
                        Err(_err) => Err("Cannot convert metadata to string".to_string()),
                    }
                } else if header.internal_compression == 2 {
                    use std::io::Read;
                    let mut d = GzDecoder::new(metadata);
                    let mut s = String::new();
                    if let Err(err) = d.read_to_string(&mut s) {
                        Err(format!("Failed to decompressed metadata {err}"))
                    } else {
                        Ok(s)
                    }
                } else {
                    Err("No compression".to_string())
                };
                match raw_metadata {
                    Ok(meta) => match serde_json::from_str(&meta) {
                        Ok(j) => Ok(j),
                        Err(e) => Err(format!("Error parsing the metadata: {e}")),
                    },
                    Err(err) => Err(err),
                }
            }
            None => Err("Cannot find metadata".to_string()),
        };
        Ok(PmTilesData { header, metadata })
    }

    /// Show ui
    pub(crate) fn ui(&self, ui: &mut egui::Ui) -> Option<RangeInclusive<usize>> {
        show_pmtiles_ui(ui, self)
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
            ui.label("Metadata Offset");
            ui.label(self.metadata_offset.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(24..=31);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Metadata Length");
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
    data: &PmTilesData,
) -> Option<RangeInclusive<usize>> {
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

    ui.collapsing("Root directory", |ui| {
        #[allow(clippy::cast_possible_truncation)]
        if ui.button("Show").clicked() {
            let start = data.header.root_directory_offset as usize;
            let end = ((data.header.root_directory_offset + data.header.root_directory_len)
                as usize)
                .saturating_sub(1);
            return_range = Some(start..=end);
        }
    });

    ui.collapsing("Metadata", |ui| {
        #[allow(clippy::cast_possible_truncation)]
        if ui.button("Show").clicked() {
            let metadata_start = data.header.metadata_offset as usize;
            let metadata_end = ((data.header.metadata_offset + data.header.metadata_len) as usize)
                .saturating_sub(1);
            return_range = Some(metadata_start..=metadata_end);
        }
        match &data.metadata {
            Ok(meta) => match serde_json::to_string_pretty(meta) {
                Ok(res) => {
                    ui.label(res);
                }
                Err(err) => {
                    ui.label(format!("Failed to format JSON metadata {err}"));
                }
            },
            Err(err) => {
                ui.label(err);
            }
        }
    });

    ui.collapsing("Leaf directories", |ui| {
        #[allow(clippy::cast_possible_truncation)]
        if ui.button("Show").clicked() {
            let start = data.header.leaf_directories_offset as usize;
            let end = ((data.header.leaf_directories_offset + data.header.leaf_directories_len)
                as usize)
                .saturating_sub(1);
            return_range = Some(start..=end);
        }
    });
    ui.collapsing("Tile Data", |ui| {
        #[allow(clippy::cast_possible_truncation)]
        if ui.button("Show").clicked() {
            let start = data.header.tile_data_offset as usize;
            let end = ((data.header.tile_data_offset + data.header.tile_data_len) as usize)
                .saturating_sub(1);
            return_range = Some(start..=end);
        }
    });
    return_range
}
