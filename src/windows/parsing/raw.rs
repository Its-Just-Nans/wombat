//! Raw String detection

use std::ops::RangeInclusive;

use bladvak::eframe::egui;
use uuid::Uuid;

/// String Detection
#[derive(Debug)]
#[allow(dead_code)]
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
    Uuid(uuid::Uuid),
    /// ipv4
    Ipv4,
    /// ipv6
    Ipv6,
    /// int
    Integer(i64),
    /// float
    Float(f64),
    /// url
    Url(url::Url),
    /// json
    Json,
    /// base 64
    Base64,
    /// base 64 url
    Base64Url,
    /// Gedcom
    Ged,
    /// unknown str
    Unknown,
}

impl std::fmt::Display for StringType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// check if hex and len
fn is_hex(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// check if base 64
fn is_base64(s: &str) -> bool {
    use base64::{Engine, engine::general_purpose};

    general_purpose::STANDARD.decode(s).is_ok()
}

/// check if base 64 url
fn is_base64url(s: &str) -> bool {
    use base64::{Engine, engine::general_purpose};

    general_purpose::URL_SAFE_NO_PAD.decode(s).is_ok()
}

/// check if ged
fn is_ged(bin: &[u8]) -> bool {
    bin.starts_with("0 HEAD\r\n1 SOUR".as_bytes())
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
            } else if let Ok(uuid) = Uuid::parse_str(s) {
                StringType::Uuid(uuid)
            } else if s.parse::<std::net::Ipv4Addr>().is_ok() {
                StringType::Ipv4
            } else if s.parse::<std::net::Ipv6Addr>().is_ok() {
                StringType::Ipv6
            } else if let Ok(int) = s.parse::<i64>() {
                StringType::Integer(int)
            } else if let Ok(float) = s.parse::<f64>() {
                StringType::Float(float)
            } else if let Ok(url) = url::Url::parse(s) {
                StringType::Url(url)
            } else if serde_json::from_str::<serde_json::Value>(s).is_ok() {
                StringType::Json
            } else if is_base64(s) {
                StringType::Base64
            } else if is_base64url(s) {
                StringType::Base64Url
            } else if is_ged(s.as_bytes()) {
                StringType::Ged
            } else {
                StringType::Unknown
            };
            return Some(str_type);
        } else if is_ged(bin) {
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
    ui.label(format!("Could be: {data}"));
    None
}
