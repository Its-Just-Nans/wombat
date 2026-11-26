//! ICC profile
//!
//! <https://www.color.org/icc_specs2/>

use std::fmt::Display;

use bladvak::eframe::egui;

use crate::windows::parsing::utils::{read_buffer, read_u32, read_u64};

#[derive(Debug)]
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
    Unkown([u8; 4]),
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
            _ => Self::Unkown(buffer),
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
            Self::Unkown(d) => write!(f, "unkown: {d:?}"),
        }
    }
}

#[derive(Debug)]
#[allow(clippy::missing_docs_in_private_items)]
pub(crate) struct IccProfileData {
    size: u32,
    preferred_cmm: [u8; 4],
    profile_version: u32,
    profile_class: [u8; 4],
    colour_space: ColourSpace,
    pcs: [u8; 4],
    date_created: DateTimeNumber,
    acsp_signature: [u8; 4],
    primary_signature: [u8; 4],
    profile_flag: u32,
    device_manufacturer: [u8; 4],
    device_model: [u8; 4],
    device_attributes: u64,
    rendering_intent: u32,
    xyz_nums: [u8; 12],
    profile_creator: [u8; 4],
    profile_id: u64,
    spectral_pcs: [u8; 4],
    spectral_wavelength: [u8; 6],
    bi_spectral_wavelength: [u8; 6],
    mcs_signature: u32,
    profile_sub_class: [u8; 4],
    reserved: u32,
}

impl IccProfileData {
    /// Try to parse the profile
    pub(crate) fn try_parse(binary_file: &[u8]) -> Result<Self, String> {
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

        let (buffer, primary_signature) = read_buffer::<4>(buffer)?;
        let (buffer, profile_flag) = read_u32(buffer, false)?;
        let (buffer, device_manufacturer) = read_buffer::<4>(buffer)?;
        let (buffer, device_model) = read_buffer::<4>(buffer)?;
        let (buffer, device_attributes) = read_u64(buffer, false)?;
        let (buffer, rendering_intent) = read_u32(buffer, false)?;
        let (buffer, xyz_nums) = read_buffer::<12>(buffer)?;
        let (buffer, profile_creator) = read_buffer::<4>(buffer)?;
        let (buffer, profile_id) = read_u64(buffer, false)?;
        let (buffer, spectral_pcs) = read_buffer::<4>(buffer)?;
        let (buffer, spectral_wavelength) = read_buffer::<6>(buffer)?;
        let (buffer, bi_spectral_wavelength) = read_buffer::<6>(buffer)?;
        let (buffer, mcs_signature) = read_u32(buffer, false)?;
        let (buffer, profile_sub_class) = read_buffer::<4>(buffer)?;
        let (buffer, reserved) = read_u32(buffer, false)?;
        if reserved != 0 {
            return Err("Reserved should be 0".to_string());
        }
        Ok(Self {
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
        })
    }

    /// Show ui
    pub(crate) fn ui(&self, ui: &mut egui::Ui) {
        ui.label(format!("{self:#x?}"));
    }
}
