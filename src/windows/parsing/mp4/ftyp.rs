//! ftyp atom

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
