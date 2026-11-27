//! ICC profile
//!
//! <https://www.color.org/icc_specs2/>

use std::{fmt::Display, ops::RangeInclusive};

use bladvak::eframe::egui::{self, Id, collapsing_header::CollapsingState};

use crate::windows::parsing::{
    icc::InnerTag::ParsingError,
    utils::{read_buffer, read_buffer_dyn, read_u16, read_u32, read_u64},
};

#[derive(Debug)]
#[allow(clippy::missing_docs_in_private_items)]
struct DateTimeNumber {
    year: u16,
    month: u16,
    day: u16,
    hours: u16,
    minutes: u16,
    seconds: u16,
}

impl DateTimeNumber {
    fn parse(buffer: [u8; 12]) -> Self {
        Self {
            year: u16::from_be_bytes([buffer[0], buffer[1]]),
            month: u16::from_be_bytes([buffer[2], buffer[3]]),
            day: u16::from_be_bytes([buffer[4], buffer[5]]),
            hours: u16::from_be_bytes([buffer[6], buffer[7]]),
            minutes: u16::from_be_bytes([buffer[8], buffer[9]]),
            seconds: u16::from_be_bytes([buffer[10], buffer[11]]),
        }
    }
}

#[derive(Debug)]
#[allow(clippy::missing_docs_in_private_items)]
enum ColourSpace {
    Xyz,
    Lab,
    Luv,
    YCbr,
    Yxy,
    Rgb,
    Gray,
    Hsv,
    Hls,
    Cmyk,
    Cmy,
    Clr2,
    Clr3,
    Clr4,
    Clr5,
    Clr6,
    Clr7,
    Clr8,
    Clr9,
    ClrA,
    ClrB,
    ClrC,
    ClrD,
    ClrE,
    ClrF,
    Unknown([u8; 4]),
}

impl ColourSpace {
    fn parse(buffer: [u8; 4]) -> Self {
        match &buffer {
            b"XYZ " => Self::Xyz,
            b"Lab " => Self::Lab,
            b"Luv " => Self::Luv,
            b"YCbr" => Self::YCbr,
            b"Yxy " => Self::Yxy,
            b"RGB " => Self::Rgb,
            b"GRAY" => Self::Gray,
            b"HSV " => Self::Hsv,
            b"HLS " => Self::Hls,
            b"CMYK" => Self::Cmyk,
            b"CMY " => Self::Cmy,
            b"2CLR" => Self::Clr2,
            b"3CLR" => Self::Clr3,
            b"4CLR" => Self::Clr4,
            b"5CLR" => Self::Clr5,
            b"6CLR" => Self::Clr6,
            b"7CLR" => Self::Clr7,
            b"8CLR" => Self::Clr8,
            b"9CLR" => Self::Clr9,
            b"ACLR" => Self::ClrA,
            b"BCLR" => Self::ClrB,
            b"CCLR" => Self::ClrC,
            b"DCLR" => Self::ClrD,
            b"ECLR" => Self::ClrE,
            b"FCLR" => Self::ClrF,
            _ => Self::Unknown(buffer),
        }
    }
}

impl Display for ColourSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Xyz => write!(f, "nCIEXYZ or PCSXYZ"),
            Self::Lab => write!(f, "CIELAB or PCSLAB"),
            Self::Luv => write!(f, "CIELUV"),
            Self::YCbr => write!(f, "YCbCr"),
            Self::Yxy => write!(f, "CIEYxy"),
            Self::Rgb => write!(f, "RGB"),
            Self::Gray => write!(f, "GRAY"),
            Self::Hsv => write!(f, "HSV"),
            Self::Hls => write!(f, "HLS"),
            Self::Cmyk => write!(f, "CMYK"),
            Self::Cmy => write!(f, "CMY"),
            Self::Clr2 => write!(f, "2 colour"),
            Self::Clr3 => write!(f, "3 colour"),
            Self::Clr4 => write!(f, "4 colour"),
            Self::Clr5 => write!(f, "5 colour"),
            Self::Clr6 => write!(f, "6 colour"),
            Self::Clr7 => write!(f, "7 colour"),
            Self::Clr8 => write!(f, "8 colour"),
            Self::Clr9 => write!(f, "9 colour"),
            Self::ClrA => write!(f, "10 colour"),
            Self::ClrB => write!(f, "11 colour"),
            Self::ClrC => write!(f, "12 colour"),
            Self::ClrD => write!(f, "13 colour"),
            Self::ClrE => write!(f, "14 colour"),
            Self::ClrF => write!(f, "15 colour"),
            Self::Unknown(d) => write!(f, "unknown: {d:?}"),
        }
    }
}

struct XYZType {
    reserved: [u8; 4],
    x: u32,
    y: u32,
    z: u32,
}

#[derive(Debug)]
struct TagSignature([u8; 4]);

impl Display for TagSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            self.0[0] as char, self.0[1] as char, self.0[2] as char, self.0[3] as char
        )
    }
}

#[derive(Debug)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct Tag {
    signature: TagSignature,
    offset: u32,
    length: u32,
    inner: InnerTag,
}

#[derive(Debug)]
#[allow(clippy::missing_docs_in_private_items)]
struct LocalizedUnicodeData {
    first_record_lang_code: u16,
    first_record_country_code: u16,
    first_record_string_length: u32,
    first_record_string_offset: u32,
    value: String,
}

#[derive(Debug)]
#[allow(clippy::missing_docs_in_private_items)]
struct MultiLocalizedUnicodeData {
    reserved: [u8; 4],
    number_of_records: u32,
    record_size: u32,
    values: Vec<LocalizedUnicodeData>,
}

impl MultiLocalizedUnicodeData {
    /// Try to parse
    fn try_parse(buffer: &[u8]) -> Result<(&[u8], Self), &'static str> {
        let (buffer, reserved) = read_buffer::<4>(buffer)?;
        if reserved != [0, 0, 0, 0] {
            return Err("Reserved is not empty");
        }
        let (buffer, number_of_records) = read_u32(buffer, false)?;
        let (mut buffer, record_size) = read_u32(buffer, false)?;
        if record_size != 12 {
            return Err("Invalid record size");
        }
        let mut values = Vec::new();
        for _ in 0..number_of_records {
            let (buff, first_record_lang_code) = read_u16(buffer, false)?;
            buffer = buff;
            let (buff, first_record_country_code) = read_u16(buffer, false)?;
            buffer = buff;
            let (buff, first_record_string_length) = read_u32(buffer, false)?;
            buffer = buff;
            let (buff, first_record_string_offset) = read_u32(buffer, false)?;
            buffer = buff;
            let (buff, value) = read_buffer_dyn(buffer, first_record_string_length as usize)?;
            buffer = buff;
            let value =
                String::from_utf8(value.to_vec()).map_err(|_e| "Failed to convert string")?;

            values.push(LocalizedUnicodeData {
                first_record_lang_code,
                first_record_country_code,
                first_record_string_length,
                first_record_string_offset,
                value,
            });
        }
        let mluc = Self {
            reserved,
            number_of_records,
            record_size,
            values,
        };
        Ok((buffer, mluc))
    }
}

#[derive(Debug)]
#[allow(clippy::missing_docs_in_private_items)]
enum InnerTag {
    MultiLocalizedUnicode(MultiLocalizedUnicodeData),
    Unknown,
    ParsingError(String),
}

impl InnerTag {
    /// ui
    fn ui(
        &self,
        ui: &mut egui::Ui,
        idx: usize,
        offset: u32,
        length: u32,
    ) -> Option<RangeInclusive<usize>> {
        let mut range = None;
        CollapsingState::load_with_default_open(
            ui.ctx(),
            Id::unique(("inner_tag", self.signature(), idx)),
            false,
        )
        .show_header(ui, |ui| {
            ui.label(self.signature());
            if ui.button("Show").clicked() {
                let start = offset as usize;
                let end = start.saturating_add(length as usize).saturating_sub(1);
                range = Some(start..=end);
            }
        })
        .body(|ui| {
            ui.label(format!("{self:?}"));
        });
        range
    }

    /// signature
    fn signature(&self) -> &str {
        match self {
            Self::MultiLocalizedUnicode(_) => "mluc",
            ParsingError(_) => "ERROR",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct IccProfileData {
    header: IccProfileHeader,
    tags: Vec<Tag>,
}

#[derive(Debug)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct IccProfileHeader {
    size: u32,
    preferred_cmm: [u8; 4],
    profile_version: u32,
    profile_class: [u8; 4],
    colour_space: ColourSpace,
    pcs: [u8; 4],
    date_created: DateTimeNumber,
    acsp_signature: TagSignature,
    primary_signature: [u8; 4],
    profile_flag: u32,
    device_manufacturer: [u8; 4],
    device_model: [u8; 4],
    device_attributes: u64,
    rendering_intent: u32,
    xyz_nums: [u8; 12],
    profile_creator: [u8; 4],
    profile_id: [u8; 16],
    spectral_pcs: [u8; 4],
    spectral_wavelength: [u8; 6],
    bi_spectral_wavelength: [u8; 6],
    mcs_signature: u32,
    profile_sub_class: [u8; 4],
    reserved: u32,
}

impl IccProfileHeader {
    /// Try to parse the profile
    pub(crate) fn try_parse(binary_file: &[u8]) -> Result<(&[u8], Self), String> {
        let (buffer, size) = read_u32(binary_file, false)?;
        if (size as usize) != binary_file.len() {
            return Err("Incorrect length of ICC profile".to_string());
        }
        let (buffer, preferred_cmm) = read_buffer::<4>(buffer)?;
        let (buffer, profile_version) = read_u32(buffer, false)?;
        let (buffer, profile_class) = read_buffer::<4>(buffer)?;
        let (buffer, colour_space) = read_buffer::<4>(buffer)?;
        let colour_space = ColourSpace::parse(colour_space);
        let (buffer, pcs) = read_buffer::<4>(buffer)?;
        let (buffer, date_created) = read_buffer::<12>(buffer)?;
        let date_created = DateTimeNumber::parse(date_created);
        let (buffer, acsp_signature) = read_buffer::<4>(buffer)?;
        if &acsp_signature != b"acsp" {
            return Err("Incorrect acsp signature".to_string());
        }
        let acsp_signature = TagSignature(acsp_signature);

        let (buffer, primary_signature) = read_buffer::<4>(buffer)?;
        let (buffer, profile_flag) = read_u32(buffer, false)?;
        let (buffer, device_manufacturer) = read_buffer::<4>(buffer)?;
        let (buffer, device_model) = read_buffer::<4>(buffer)?;
        let (buffer, device_attributes) = read_u64(buffer, false)?;
        let (buffer, rendering_intent) = read_u32(buffer, false)?;
        let (buffer, xyz_nums) = read_buffer::<12>(buffer)?;
        let (buffer, profile_creator) = read_buffer::<4>(buffer)?;
        let (buffer, profile_id) = read_buffer::<16>(buffer)?;
        let (buffer, spectral_pcs) = read_buffer::<4>(buffer)?;
        let (buffer, spectral_wavelength) = read_buffer::<6>(buffer)?;
        let (buffer, bi_spectral_wavelength) = read_buffer::<6>(buffer)?;
        let (buffer, mcs_signature) = read_u32(buffer, false)?;
        let (buffer, profile_sub_class) = read_buffer::<4>(buffer)?;
        let (buffer, reserved) = read_u32(buffer, false)?;
        if reserved != 0 {
            return Err("Reserved should be 0".to_string());
        }
        Ok((
            buffer,
            Self {
                size,
                preferred_cmm,
                profile_version,
                profile_class,
                colour_space,
                pcs,
                date_created,
                acsp_signature,
                primary_signature,
                profile_flag,
                device_manufacturer,
                device_model,
                device_attributes,
                rendering_intent,
                xyz_nums,
                profile_creator,
                profile_id,
                spectral_pcs,
                spectral_wavelength,
                bi_spectral_wavelength,
                mcs_signature,
                profile_sub_class,
                reserved,
            },
        ))
    }
}

impl IccProfileData {
    /// Try to parse the profile
    pub(crate) fn try_parse(binary_file: &[u8]) -> Result<Self, String> {
        let (buffer, header) = IccProfileHeader::try_parse(binary_file)?;
        let (mut buffer, tag_table_len) = read_u32(buffer, false)?;
        let mut tags = Vec::new();
        let tags_table_end = (tag_table_len as usize).saturating_mul(12);
        for _idx in (0..tags_table_end).step_by(12) {
            let (buff, signature) = read_buffer::<4>(buffer)?;
            let signature = TagSignature(signature);
            buffer = buff;
            let (buff, offset) = read_u32(buffer, false)?;
            buffer = buff;
            let (buff, length) = read_u32(buffer, false)?;
            buffer = buff;
            let start = offset as usize;
            let end = offset as usize + length as usize;
            let inner = match binary_file.get(start..end) {
                Some(tag_section) if &tag_section[0..4] == b"mluc" => {
                    match MultiLocalizedUnicodeData::try_parse(&tag_section[4..]) {
                        Ok((buff, mluc)) => InnerTag::MultiLocalizedUnicode(mluc),
                        Err(err) => InnerTag::ParsingError(err.to_string()),
                    }
                }
                _ => InnerTag::Unknown,
            };
            let tag = Tag {
                signature,
                offset,
                length,
                inner,
            };
            tags.push(tag);
        }
        Ok(Self { header, tags })
    }

    /// Show ui
    pub(crate) fn ui(&self, ui: &mut egui::Ui) -> Option<RangeInclusive<usize>> {
        ui.label(format!("{:#x?}", self.header));
        let mut range = None;
        if self.tags.is_empty() {
            ui.label("No tags");
        } else {
            ui.horizontal(|ui| {
                let len = self.tags.len();
                ui.label(format!("Tags: {len}"));
                if ui.button("Show").clicked() {
                    let start = 128_usize;
                    let real_len = len.saturating_mul(12);
                    let end = 4_usize;
                    let end = end.saturating_add(start).saturating_add(real_len);
                    let end = end.saturating_sub(1);
                    range = Some(start..=end);
                }
            });
            for (idx, one_tag) in self.tags.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}", one_tag.signature));
                    ui.label(format!("offset = {}", one_tag.offset));
                    ui.label(format!("length = {}", one_tag.length));
                    if ui.button("Show").clicked() {
                        let start = 128_usize.saturating_add(4).saturating_add(idx * 12);
                        let end = start.saturating_add(12).saturating_sub(1);
                        range = Some(start..=end);
                    }
                });
                ui.horizontal(|ui| {
                    if let Some(r) = one_tag.inner.ui(ui, idx, one_tag.offset, one_tag.length) {
                        range = Some(r);
                    }
                });
            }
        }
        range
    }
}
