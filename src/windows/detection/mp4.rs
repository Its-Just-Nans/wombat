//! MP4 parser

use std::ops::RangeInclusive;

use bladvak::eframe::egui;

/// Box header size
const BOX_HEADER_SIZE: usize = 8;

/// Ftyp atom
#[derive(Debug)]
pub(crate) struct Ftyp {
    /// Brand
    major_brand: String,
    /// Minor version
    minor_version: u32,
    /// Compatible brands
    compatible_brands: Vec<String>,
}

impl Ftyp {
    /// Parse the Ftyp box
    pub(crate) fn parse(data: &[u8]) -> Option<Self> {
        let major = data.get(0..4)?;
        let major_brand = String::from_utf8_lossy(major).to_string();

        let minor_version = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        let mut compatible_brands = Vec::new();

        let mut i = 8;
        while i + 4 <= data.len() {
            compatible_brands.push(String::from_utf8_lossy(&data[i..i + 4]).to_string());
            i += 4;
        }

        Some(Self {
            major_brand,
            minor_version,
            compatible_brands,
        })
    }
}

/// Different Mp4 box data
#[derive(Debug)]
enum Mp4BoxData {
    /// Ftyp
    Ftyp(Ftyp),
    /// Unknown box
    Unknown,
}

/// A MP4 Box
#[derive(Debug)]
struct Mp4Box {
    /// offset
    offset: u64,
    /// size
    size: u64,
    /// name
    name: String,
    /// data
    data: Mp4BoxData,
}

/// Mp4 Data
#[derive(Debug)]
pub(crate) struct Mp4Data {
    /// Boxes
    boxes: Vec<Mp4Box>,
}

impl Mp4Data {
    /// Parse an mp4 file
    pub(crate) fn parse(binary_data: &[u8]) -> Option<Self> {
        let mut boxes = Vec::new();
        let mut offset = 0;
        let length = binary_data.len();

        if length == 0 {
            return None;
        }

        while offset < length {
            let Some(header) = binary_data.get(offset..(offset + BOX_HEADER_SIZE)) else {
                break;
            };

            let size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;

            if size == 0 {
                break;
            }

            let name = String::from_utf8_lossy(&header[4..8]).to_string();
            let Some(raw_data) = binary_data.get((offset + BOX_HEADER_SIZE)..(offset + size))
            else {
                break;
            };
            let data = match name.as_str() {
                "ftyp" => {
                    let Some(ftyp) = Ftyp::parse(raw_data) else {
                        break;
                    };
                    Mp4BoxData::Ftyp(ftyp)
                }
                _ => Mp4BoxData::Unknown,
            };
            boxes.push(Mp4Box {
                name,
                size: size as u64,
                offset: offset as u64,
                data,
            });

            offset += size;
        }
        Some(Mp4Data { boxes })
    }
}

/// Show the UI of the cached data
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn show_mp4_ui(
    ui: &mut egui::Ui,
    mp4_data: Option<&Mp4Data>,
) -> Option<RangeInclusive<usize>> {
    ui.label("MP4");
    let Some(data) = mp4_data else {
        ui.label("Parsing dailed");
        return None;
    };
    let mut return_range = None;
    egui::Grid::new("png_chunks_table")
        .striped(true)
        .show(ui, |ui| {
            ui.label("Size");
            ui.label("Name");
            ui.label("Start");
            ui.label("End");
            ui.end_row();

            for one_box in &data.boxes {
                let start = one_box.offset as usize;
                let end = (one_box.offset + one_box.size) as usize;
                ui.label(format!("{}", one_box.size));
                ui.label(&one_box.name);
                ui.label(format!("{start}"));
                ui.label(format!("{end}"));
                if ui.button("Show").clicked() {
                    let range = start..=end;
                    return_range = Some(range);
                }
                ui.end_row();
            }
        });
    for one_box in &data.boxes {
        ui.collapsing(&one_box.name, |ui| match &one_box.data {
            Mp4BoxData::Ftyp(Ftyp {
                major_brand,
                minor_version,
                compatible_brands,
            }) => {
                ui.label(format!("Major brand: {major_brand}"));
                ui.label(format!("Minor version: {minor_version}"));
                let comp = compatible_brands.join(", ");
                ui.label(format!("Compatible brands: {comp}"));
            }
            Mp4BoxData::Unknown => {
                ui.label("not parsed");
            }
        });
    }
    return_range
}
