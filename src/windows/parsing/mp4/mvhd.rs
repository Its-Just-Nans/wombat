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
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        let Some(version) = data.first() else {
            return Err("Cannot get version of mvhd".to_string());
        };
        let version = *version;
        let flags = (u32::from(
            data.get(1)
                .copied()
                .ok_or_else(|| "Cannot get data for mvhd".to_string())?,
        ) << 16)
            | (u32::from(
                data.get(2)
                    .copied()
                    .ok_or_else(|| "Cannot get data for mvhd".to_string())?,
            ) << 8)
            | u32::from(
                data.get(3)
                    .copied()
                    .ok_or_else(|| "Cannot get data for mvhd".to_string())?,
            );

        match version {
            0 => {
                if data.len() < 20 {
                    return Err("Invalid len: mvhd < 20".to_string());
                }

                Ok(Self {
                    version,
                    flags,
                    creation_time: u64::from(u32::from_be_bytes(
                        data[4..8]
                            .try_into()
                            .map_err(|_e| "Cannot convert bytes for mvhd".to_string())?,
                    )),
                    modification_time: u64::from(u32::from_be_bytes(
                        data[8..12]
                            .try_into()
                            .map_err(|_e| "Cannot convert bytes for mvhd".to_string())?,
                    )),
                    timescale: u32::from_be_bytes(
                        data[12..16]
                            .try_into()
                            .map_err(|_e| "Cannot convert bytes for mvhd".to_string())?,
                    ),
                    duration: u64::from(u32::from_be_bytes(
                        data[16..20]
                            .try_into()
                            .map_err(|_e| "Cannot convert bytes for mvhd".to_string())?,
                    )),
                })
            }

            1 => {
                if data.len() < 32 {
                    return Err("Invalid len: mvhd < 32".to_string());
                }

                Ok(Self {
                    version,
                    flags,
                    creation_time: u64::from_be_bytes(
                        data[4..12]
                            .try_into()
                            .map_err(|_e| "Cannot convert bytes for mvhd".to_string())?,
                    ),
                    modification_time: u64::from_be_bytes(
                        data[12..20]
                            .try_into()
                            .map_err(|_e| "Cannot convert bytes for mvhd".to_string())?,
                    ),
                    timescale: u32::from_be_bytes(
                        data[20..24]
                            .try_into()
                            .map_err(|_e| "Cannot convert bytes for mvhd".to_string())?,
                    ),
                    duration: u64::from_be_bytes(
                        data[24..32]
                            .try_into()
                            .map_err(|_e| "Cannot convert bytes for mvhd".to_string())?,
                    ),
                })
            }

            _ => Err("Version of mvhd not handled".to_string()),
        }
    }
}
