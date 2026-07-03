//! MP4 parser

/// Box header size
const BOX_HEADER_SIZE: usize = 8;

/// Ftyp atom
#[derive(Debug)]
pub(crate) struct Ftyp {
    /// Brand
    pub(crate) major_brand: String,
    /// Minor version
    pub(crate) minor_version: u32,
    /// Compatible brands
    pub(crate) compatible_brands: Vec<String>,
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

/// mdat atom
#[derive(Debug)]
pub(crate) struct MDat {
    /// Size
    pub(crate) size: usize,
}

impl MDat {
    /// Parse mdat atom
    pub(crate) fn parse(data: &[u8]) -> Self {
        Self { size: data.len() }
    }
}

/// free atom
#[derive(Debug)]
pub(crate) struct Free {
    /// Size
    pub(crate) size: usize,
}

impl Free {
    /// Parse free atom
    pub(crate) fn parse(data: &[u8]) -> Self {
        Self { size: data.len() }
    }
}

/// Generic Container
#[derive(Debug)]
pub(crate) struct ContainerBox {
    /// children
    pub(crate) children: Vec<Mp4Box>,
}

impl ContainerBox {
    /// Parse the generic
    fn parse(data: &[u8]) -> Option<Self> {
        Some(Self {
            children: Mp4Box::parse_all(data, 0)?,
        })
    }
}

/// Different Mp4 box data
#[derive(Debug)]
pub(crate) enum Mp4BoxData {
    /// Ftyp
    Ftyp(Ftyp),
    /// mvhd
    Mvhd(Mvhd),
    /// meta
    Meta(MetaBox),
    /// Generic Container
    Container(ContainerBox),
    /// mdat
    MDat(MDat),
    /// free
    Free(Free),
    /// Unknown box
    Unknown,
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
            "moov" | "udta" | "trak" | "mdia" | "minf" | "stbl" => {
                Mp4BoxData::Container(ContainerBox::parse(raw)?)
            }

            // content
            "mvhd" => Mp4BoxData::Mvhd(Mvhd::parse(raw)?),
            "meta" => Mp4BoxData::Meta(MetaBox::parse(raw)?),

            _ => Mp4BoxData::Unknown,
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

/// mvhd atom
#[derive(Debug)]
pub(crate) struct Mvhd {
    /// version
    pub(crate) version: u8,
    /// flags
    pub(crate) flags: u32,
    /// creation time
    pub(crate) creation_time: u64,
    /// modification time
    pub(crate) modification_time: u64,
    /// timescale
    pub(crate) timescale: u32,
    /// duration
    pub(crate) duration: u64,
}

impl Mvhd {
    /// Parse mvhd
    pub fn parse(data: &[u8]) -> Option<Self> {
        let version = *data.first()?;
        let flags = (u32::from(data.get(1).copied()?) << 16)
            | (u32::from(data.get(2).copied()?) << 8)
            | u32::from(data.get(3).copied()?);

        match version {
            0 => {
                if data.len() < 20 {
                    return None;
                }

                Some(Self {
                    version,
                    flags,
                    creation_time: u64::from(u32::from_be_bytes(data[4..8].try_into().ok()?)),
                    modification_time: u64::from(u32::from_be_bytes(data[8..12].try_into().ok()?)),
                    timescale: u32::from_be_bytes(data[12..16].try_into().ok()?),
                    duration: u64::from(u32::from_be_bytes(data[16..20].try_into().ok()?)),
                })
            }

            1 => {
                if data.len() < 32 {
                    return None;
                }

                Some(Self {
                    version,
                    flags,
                    creation_time: u64::from_be_bytes(data[4..12].try_into().ok()?),
                    modification_time: u64::from_be_bytes(data[12..20].try_into().ok()?),
                    timescale: u32::from_be_bytes(data[20..24].try_into().ok()?),
                    duration: u64::from_be_bytes(data[24..32].try_into().ok()?),
                })
            }

            _ => None,
        }
    }
}

/// Meta
#[derive(Debug)]
pub(crate) struct MetaBox {
    /// Version
    pub(crate) version: u8,
    /// Flags
    pub(crate) flags: u32,
    /// Children
    pub(crate) children: Vec<Mp4Box>,
}

impl MetaBox {
    /// Parse the meta
    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }

        let version = data[0];
        let flags = (u32::from(data[1]) << 16) | (u32::from(data[2]) << 8) | u32::from(data[3]);

        let children = Mp4Box::parse_all(&data[4..], 4)?;

        Some(Self {
            version,
            flags,
            children,
        })
    }
}
