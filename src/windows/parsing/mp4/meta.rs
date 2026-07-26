//! meta atom

use crate::windows::parsing::mp4::BOX_HEADER_SIZE;
use crate::windows::parsing::mp4::Mp4Box;

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
    pub(crate) fn parse(data: &[u8], offset: usize) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }

        let version = data[0];
        let flags = (u32::from(data[1]) << 16) | (u32::from(data[2]) << 8) | u32::from(data[3]);

        let children = Mp4Box::parse_all(&data[4..], offset + BOX_HEADER_SIZE + 4)?;

        Some(Self {
            version,
            flags,
            children,
        })
    }
}
