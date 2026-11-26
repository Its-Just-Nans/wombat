//! utils functions for parsing

/// read a u32
pub(crate) fn read_u32(buffer: &[u8], little_endian: bool) -> Result<(&[u8], u32), &'static str> {
    if buffer.len() < 4 {
        return Err("Buffer is not big enough");
    }
    let bytes = [buffer[0], buffer[1], buffer[2], buffer[3]];
    let num = if little_endian {
        u32::from_le_bytes(bytes)
    } else {
        u32::from_be_bytes(bytes)
    };
    Ok((&buffer[4..], num))
}

/// read a u64
pub(crate) fn read_u64(buffer: &[u8], little_endian: bool) -> Result<(&[u8], u64), &'static str> {
    if buffer.len() < 8 {
        return Err("Buffer is not big enough");
    }
    let bytes = [
        buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
    ];
    let num = if little_endian {
        u64::from_le_bytes(bytes)
    } else {
        u64::from_be_bytes(bytes)
    };
    Ok((&buffer[8..], num))
}

/// read a fixed buffer
pub(crate) fn read_buffer<const N: usize>(buffer: &[u8]) -> Result<(&[u8], [u8; N]), &'static str> {
    if buffer.len() < N {
        return Err("Buffer is not big enough");
    }
    let bytes = buffer[0..N]
        .try_into()
        .map_err(|_e| "Cannot convert buffer size")?;
    Ok((&buffer[N..], bytes))
}
