//! utils functions for parsing

/// read a u16
pub(crate) fn read_u16(buffer: &[u8], little_endian: bool) -> Result<(&[u8], u16), &'static str> {
    if buffer.len() < 2 {
        return Err("Buffer is not big enough to read u16");
    }
    let bytes = [buffer[0], buffer[1]];
    let num = if little_endian {
        u16::from_le_bytes(bytes)
    } else {
        u16::from_be_bytes(bytes)
    };
    Ok((&buffer[2..], num))
}

/// read a u32
pub(crate) fn read_u32(buffer: &[u8], little_endian: bool) -> Result<(&[u8], u32), &'static str> {
    if buffer.len() < 4 {
        return Err("Buffer is not big enough to read u32");
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
        return Err("Buffer is not big enough to read u64");
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
        return Err("Buffer is not big enough to read buffer");
    }
    let bytes = buffer[0..N]
        .try_into()
        .map_err(|_e| "Cannot convert buffer size")?;
    Ok((&buffer[N..], bytes))
}

/// Read a buffer of dynamic size
pub(crate) fn read_buffer_dyn(buffer: &[u8], size: usize) -> Result<(&[u8], &[u8]), &'static str> {
    if buffer.len() < size {
        return Err("Buffer is not big enough to read dynamic buffer");
    }

    Ok((&buffer[size..], &buffer[..size]))
}
