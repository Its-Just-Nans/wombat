//! mdat atom

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
