//! mvhd atom

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
