//! Mp4 ui

use std::ops::RangeInclusive;

use crate::windows::parsing::mp4::Ftyp;
use crate::windows::parsing::mp4::{Mp4Box, Mp4BoxData, Mp4Data};
use bladvak::eframe::egui::{self, CollapsingHeader};

/// Show the UI of the cached data
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn show_mp4_ui(
    ui: &mut egui::Ui,
    mp4_data: Option<&Mp4Data>,
) -> Option<RangeInclusive<usize>> {
    ui.label("MP4");
    let Some(data) = mp4_data else {
        ui.label("Parsing failed");
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
                    let range = start..=(end - 1);
                    return_range = Some(range);
                }
                ui.end_row();
            }
        });
    for (idx, one_box) in data.boxes.iter().enumerate() {
        if let Some(range) = show_box(ui, one_box, idx) {
            return_range = Some(range);
        }
    }
    return_range
}

/// Show the boxes
#[allow(clippy::cast_possible_truncation)]
#[must_use]
fn show_box(ui: &mut egui::Ui, one_box: &Mp4Box, idx: usize) -> Option<RangeInclusive<usize>> {
    let mut return_range = None;
    CollapsingHeader::new(&one_box.name)
        .id_salt(format!("{}_{idx}", one_box.name))
        .show(ui, |ui| {
            if ui.button("Show").clicked() {
                let start = one_box.offset as usize;
                let end = (one_box.offset + one_box.size) as usize;
                let range = start..=(end - 1);
                return_range = Some(range);
            }
            match &one_box.data {
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
                Mp4BoxData::Container(data) => {
                    for (index, one_box) in data.iter().enumerate() {
                        if let Some(range) = show_box(ui, one_box, idx + index) {
                            return_range = Some(range);
                        }
                    }
                }
                Mp4BoxData::Meta(meta_box) => {
                    ui.label(format!("Version: {}", meta_box.version));
                    ui.label(format!("Flags: {}", meta_box.flags));
                    for (index, one_box) in meta_box.children.iter().enumerate() {
                        if let Some(range) = show_box(ui, one_box, idx + index) {
                            return_range = Some(range);
                        }
                    }
                }
                Mp4BoxData::Mvhd(data) => {
                    ui.label(format!("Version: {}", data.version));
                    ui.label(format!("Flags: {}", data.flags));
                    ui.label(format!("Creation Time: {}", data.creation_time));
                    ui.label(format!("Modification Time: {}", data.modification_time));
                    ui.label(format!("Timescale: {}", data.timescale));
                    ui.label(format!("Duration: {}", data.duration));
                }
                Mp4BoxData::MDat(data) => {
                    ui.label(format!("Size: {}", data.size));
                }
                Mp4BoxData::Free(data) => {
                    ui.label(format!("Size: {}", data.size));
                }
                Mp4BoxData::Stzs(data) => {
                    data.show(ui);
                }
                Mp4BoxData::Stts(data) => {
                    data.show(ui);
                }
                Mp4BoxData::Stco(data) => {
                    data.show(ui);
                }
                Mp4BoxData::Stsc(data) => {
                    data.show(ui);
                }
                Mp4BoxData::Co64(data) => {
                    data.show(ui);
                }
                Mp4BoxData::Stsd(data) => {
                    data.show(ui);
                }
                Mp4BoxData::Unknown(data) => {
                    ui.label(format!("{data:?}"));
                }
            }
        });
    return_range
}
