//! Raw String detection

use std::ops::RangeInclusive;

use bladvak::eframe::egui;
use uuid::Uuid;

/// String Detection
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum StringType {
    /// Md5
    Md5,
    /// sha 1
    Sha1,
    /// sha 256
    Sha256,
    /// sha 512
    Sha512,
    /// uuid
    Uuid,
    /// Gedcom
    Ged,
    /// unknown str
    Unknown,
}

/// check if hex and len
fn is_hex(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|b| b.is_ascii_hexdigit())
}

impl StringType {
    /// parse
    pub fn parse(bin: &[u8]) -> Option<StringType> {
        if let Ok(s) = std::str::from_utf8(bin) {
            let str_type = if is_hex(s, 32) {
                StringType::Md5
            } else if is_hex(s, 40) {
                StringType::Sha1
            } else if is_hex(s, 64) {
                StringType::Sha256
            } else if is_hex(s, 128) {
                StringType::Sha512
            } else if Uuid::parse_str(s).is_ok() {
                StringType::Uuid
            } else {
                StringType::Unknown
            };
            return Some(str_type);
        } else if bin.starts_with("0 HEAD\r\n1 SOUR".as_bytes()) {
            return Some(StringType::Ged);
        }
        None
    }
}

/// Show the ui
pub(crate) fn show_raw_string_data(
    ui: &mut egui::Ui,
    data: Option<&StringType>,
) -> Option<RangeInclusive<usize>> {
    let Some(data) = data else {
        ui.label("Unknown bin");
        return None;
    };
    ui.label(format!("Could be: {data:?}"));
    None
}
