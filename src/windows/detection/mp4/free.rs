//! free atom

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
