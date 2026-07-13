//! JPG format

use std::ops::RangeInclusive;

use bladvak::eframe::egui;

/// Jpg data
#[derive(Debug)]
pub(crate) struct JpgData {
    /// jgp segments
    segments: Vec<Segment>,
}

impl JpgData {
    /// parse the data
    pub(crate) fn parse(binary_file: &[u8]) -> Option<Self> {
        let segments = parse_jpeg(binary_file).ok()?;
        Some(Self { segments })
    }
}

/// Jpg marker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Marker {
    /// Start of image
    SOI,
    /// End of image
    EOI,
    /// Start of scan
    SOS,
    /// Define Quantization Table
    DQT,
    /// Define Huffman Table
    DHT,
    /// Comment
    COM,
    /// APP0..APP15
    APP(u8),
    /// SOF0..SOF15
    SOF(u8),
    /// RST0..RST7
    RST(u8),
    /// Unknown marker
    Unknown(u8),
    /// Extra data
    Extra,
}

impl Marker {
    /// get information about marker
    pub(crate) fn info(self) -> &'static str {
        match self {
            Marker::SOI => "Start Of Image",
            Marker::EOI => "End Of Image",
            Marker::SOS => "Start Of Scan",
            Marker::DQT => "Define Quantization Table",
            Marker::DHT => "Define Huffman Table",
            Marker::COM => "Comment",

            Marker::APP(_) => "Application-specific segment",
            Marker::SOF(0) => "Baseline DCT frame",
            Marker::SOF(1) => "Extended Sequential DCT frame",
            Marker::SOF(2) => "Progressive DCT frame",
            Marker::SOF(3) => "Lossless frame",
            Marker::SOF(5) => "Differential Sequential DCT frame",
            Marker::SOF(6) => "Differential Progressive DCT frame",
            Marker::SOF(7) => "Differential Lossless frame",
            Marker::SOF(9) => "Extended Sequential (Arithmetic) frame",
            Marker::SOF(10) => "Progressive (Arithmetic) frame",
            Marker::SOF(11) => "Lossless (Arithmetic) frame",
            Marker::SOF(13) => "Differential Sequential (Arithmetic)",
            Marker::SOF(14) => "Differential Progressive (Arithmetic)",
            Marker::SOF(15) => "Differential Lossless (Arithmetic)",
            Marker::SOF(_) => "Reserved Start Of Frame",

            Marker::RST(_) => "Restart marker",

            Marker::Unknown(_) => "Unknown JPEG marker",
            Marker::Extra => "Stuffed 0xFF byte (not a marker)",
        }
    }
}

/// Jpg Segment
#[derive(Debug, Clone, Copy)]
pub struct Segment {
    /// marker
    pub marker: Marker,
    /// Start of the payload (after the length field).
    pub start: usize,
    /// End of the payload (exclusive).
    pub end: usize,
}

/// parse the jpeg
pub fn parse_jpeg(data: &[u8]) -> Result<Vec<Segment>, String> {
    let mut out = Vec::new();
    let mut pos = 0;

    while pos + 1 < data.len() {
        if data[pos] != 0xFF {
            return Err("InvalidMarker".to_string());
        }

        let id = data[pos + 1];

        match id {
            // Standalone markers
            0xD8 => {
                let start = pos;
                pos += 2;
                out.push(Segment {
                    marker: Marker::SOI,
                    start,
                    end: start + 1,
                });
            }

            0xD9 => {
                let start = pos;
                out.push(Segment {
                    marker: Marker::EOI,
                    start,
                    end: start + 1,
                });
                pos += 2;
                break;
            }

            0xD0..=0xD7 => {
                let start = pos;
                pos += 2;
                out.push(Segment {
                    marker: Marker::RST(id - 0xD0),
                    start,
                    end: start + 1,
                });
            }

            // Start of Scan
            0xDA => {
                let start = pos;
                pos += 2;
                let len = read_len(data, &mut pos)?;

                let header_end = pos + len - 2;

                if header_end > data.len() {
                    return Err("UnexpectedEof".to_string());
                }

                pos = header_end;

                while pos + 1 < data.len() {
                    if data[pos] != 0xFF {
                        pos += 1;
                        continue;
                    }

                    match data[pos + 1] {
                        // 0x00 is stuffed FF
                        0x00 | 0xD0..=0xD7 => pos += 2, // restart
                        _ => break,
                    }
                }

                out.push(Segment {
                    marker: Marker::SOS,
                    start,
                    end: pos - 1,
                });
            }

            // All other markers have a length
            _ => {
                let start = pos;
                pos += 2;
                let len = read_len(data, &mut pos)?;

                if len < 2 {
                    return Err("InvalidLength".to_string());
                }

                let end = pos + len - 2;

                if end > data.len() {
                    return Err("UnexpectedEof".to_string());
                }

                out.push(Segment {
                    marker: marker(id),
                    start,
                    end: end - 1,
                });

                pos = end;
            }
        }
    }

    if pos != data.len() {
        out.push(Segment {
            marker: Marker::Extra,
            start: pos,
            end: data.len().saturating_sub(1),
        });
    }

    Ok(out)
}

/// Read a len
fn read_len(data: &[u8], pos: &mut usize) -> Result<usize, String> {
    if *pos + 2 > data.len() {
        return Err("UnexpectedEof".to_string());
    }

    let len = u16::from_be_bytes([data[*pos], data[*pos + 1]]) as usize;
    *pos += 2;
    Ok(len)
}

/// marker from u8
fn marker(id: u8) -> Marker {
    match id {
        0xDB => Marker::DQT,
        0xC4 => Marker::DHT,
        0xDA => Marker::SOS,
        0xFE => Marker::COM,

        0xE0..=0xEF => Marker::APP(id - 0xE0),

        0xC0..=0xCF if id != 0xC4 && id != 0xC8 && id != 0xCC => Marker::SOF(id - 0xC0),

        _ => Marker::Unknown(id),
    }
}

/// show jpg data
pub(crate) fn show_jpg_data(
    ui: &mut egui::Ui,
    jpg_data: Option<&JpgData>,
) -> Option<RangeInclusive<usize>> {
    let Some(data) = jpg_data else {
        ui.label("Failed to parse the JPG data");
        return None;
    };
    let mut return_range = None;
    egui::Grid::new("jpg_table").striped(true).show(ui, |ui| {
        ui.label("Marker");
        ui.label("Start");
        ui.label("End");
        ui.end_row();

        for one_segment in &data.segments {
            ui.label(format!("{:?}", one_segment.marker))
                .on_hover_ui(|ui| {
                    ui.label(one_segment.marker.info());
                });
            ui.label(one_segment.start.to_string());
            ui.label(one_segment.end.to_string());
            if ui.button("Show").clicked() {
                return_range = Some(RangeInclusive::new(one_segment.start, one_segment.end));
            }
            ui.end_row();
        }
    });
    return_range
}
