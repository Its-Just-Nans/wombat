//! MP4 parser

mod co64;
mod free;
mod ftyp;
mod mdat;
mod meta;
mod mvhd;
mod stco;
mod stsc;
mod stsd;
mod stsz;
mod stts;
pub(crate) mod ui;

use free::Free;
use ftyp::Ftyp;
use mdat::MDat;
use meta::MetaBox;
use mvhd::Mvhd;
use stsd::StsdBox;
use stsz::StszBox;

use crate::windows::parsing::mp4::{co64::Co64Box, stco::StcoBox, stsc::StscBox, stts::SttsBox};

/// Box header size
const BOX_HEADER_SIZE: usize = 8;

/// Different Mp4 box data
#[derive(Debug)]
pub(crate) enum Mp4BoxData {
    /// Ftyp
    Ftyp(Ftyp),
    /// mvhd
    Mvhd(Mvhd),
    /// meta
    Meta(MetaBox),
    /// stsd
    Stsd(StsdBox),
    /// stzs
    Stzs(StszBox),
    /// stts
    Stts(SttsBox),
    /// stco
    Stco(StcoBox),
    /// stsc
    Stsc(StscBox),
    /// co64
    Co64(Co64Box),
    /// Generic Container
    Container(Vec<Mp4Box>),
    /// mdat
    MDat(MDat),
    /// free
    Free(Free),
    /// Unknown box
    Unknown(Box<[u8]>),
}

/// A MP4 Box
#[derive(Debug)]
pub(crate) struct Mp4Box {
    /// offset
    pub(crate) offset: u64,
    /// size
    pub(crate) size: u64,
    /// name
    pub(crate) name: String,
    /// data
    pub(crate) data: Mp4BoxData,
}

impl Mp4Box {
    /// Parse
    fn parse(data: &[u8], offset: usize) -> Option<Self> {
        let header = data.get(..BOX_HEADER_SIZE)?;

        let size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;

        if size < BOX_HEADER_SIZE || size > data.len() {
            return None;
        }

        let name = String::from_utf8_lossy(&header[4..8]).to_string();

        let raw = &data[BOX_HEADER_SIZE..size];

        let data = match name.as_str() {
            "ftyp" => Mp4BoxData::Ftyp(Ftyp::parse(raw)?),
            "mdat" => Mp4BoxData::MDat(MDat::parse(raw)),
            "free" => Mp4BoxData::Free(Free::parse(raw)),

            // containers
            "moov" | "udta" | "trak" | "mdia" | "minf" | "stbl" | "edts" | "ilst" => {
                Mp4BoxData::Container(Mp4Box::parse_all(raw, offset + BOX_HEADER_SIZE)?)
            }

            // content
            "mvhd" => Mp4BoxData::Mvhd(Mvhd::parse(raw)?),
            "meta" => Mp4BoxData::Meta(MetaBox::parse(raw, offset)?),
            "stsd" => Mp4BoxData::Stsd(StsdBox::parse(raw)?),
            "stsz" => Mp4BoxData::Stzs(StszBox::parse(raw)?),
            "stts" => Mp4BoxData::Stts(SttsBox::parse(raw)?),
            "stco" => Mp4BoxData::Stco(StcoBox::parse(raw)?),
            "stsc" => Mp4BoxData::Stsc(StscBox::parse(raw)?),
            "co64" => Mp4BoxData::Co64(Co64Box::parse(raw)?),

            _ => Mp4BoxData::Unknown(raw.into()),
        };

        Some(Self {
            offset: offset as u64,
            size: size as u64,
            name,
            data,
        })
    }

    /// Parse all
    fn parse_all(data: &[u8], base_offset: usize) -> Option<Vec<Self>> {
        let mut boxes = Vec::new();
        let mut offset = 0;

        while offset + BOX_HEADER_SIZE <= data.len() {
            let b = Self::parse(&data[offset..], base_offset + offset)?;
            let size = usize::try_from(b.size).ok()?;

            if size == 0 {
                break;
            }

            boxes.push(b);
            offset += size;
        }

        Some(boxes)
    }
}

/// Mp4 Data
#[derive(Debug)]
pub(crate) struct Mp4Data {
    /// Boxes
    pub(crate) boxes: Vec<Mp4Box>,
}

impl Mp4Data {
    /// parse
    pub(crate) fn parse(binary_data: &[u8]) -> Option<Self> {
        Some(Self {
            boxes: Mp4Box::parse_all(binary_data, 0)?,
        })
    }
}
