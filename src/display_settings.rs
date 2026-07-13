//! Display settings

use std::ops::RangeInclusive;

/// Display setting
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct DisplaySettings {
    /// Least Significant Bit
    pub(crate) display_lsb: bool,
    /// limit ascii
    pub(crate) limit_to_base_ascii: bool,
    /// show color picket
    pub(crate) show_color_picker: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            display_lsb: false,
            limit_to_base_ascii: true,
            show_color_picker: true,
        }
    }
}

/// Accent type
#[derive(Debug, PartialEq)]
pub(crate) enum Accent {
    /// decimal
    Decimal,
    /// hexe
    Hex,
    /// octal
    Octal,
    /// binary
    Binary,
    /// ascii
    Ascii,
}

impl DisplaySettings {
    /// start ASCII printable char (after space)
    pub(crate) const RANGE_ASCII_PRINTABLE: RangeInclusive<u8> = 0x21_u8..=0x7E;

    /// ascii u8 to string
    pub(crate) fn ascii_to_string(&self, c: u8) -> String {
        match c {
            0x0 => "NUL (Null character)".to_string(),
            0x01 => "SOH (Start of Heading)".to_string(),
            0x02 => "STX (Start of Text)".to_string(),
            0x03 => "ETX (End of Text)".to_string(),
            0x04 => "EOT (End of Transmission)".to_string(),
            0x05 => "ENQ (Enquiry)".to_string(),
            0x06 => "ACK (Acknowledge)".to_string(),
            0x07 => "BEL (Bell, Alert)".to_string(),
            0x08 => "BS (Backspace)".to_string(),
            0x09 => "HT (Horizontal Tab)".to_string(),
            0x0A => "LF (Line Feed)".to_string(),
            0x0B => "VT (Vertical Tabulation)".to_string(),
            0x0C => "FF (Form Feed)".to_string(),
            0x0D => "CR (Carriage Return)".to_string(),
            0x0E => "SO (Shift Out)".to_string(),
            0x0F => "SI (Shift In)".to_string(),
            0x10 => "DLE (Data Link Escape)".to_string(),
            0x11 => "DC1 (Device Control One (XON))".to_string(),
            0x12 => "DC2 (Device Control Two)".to_string(),
            0x13 => "DC3 (Device Control Three (XOFF))".to_string(),
            0x14 => "DC4 (Device Control Four)".to_string(),
            0x15 => "NAK (Negative Acknowledge)".to_string(),
            0x16 => "SYN (Synchronous Idle)".to_string(),
            0x17 => "ETB (End of Transmission Block)".to_string(),
            0x18 => "CAN (Cancel)".to_string(),
            0x19 => "EM (End of medium)".to_string(),
            0x1A => "SUB (Substitute)".to_string(),
            0x1B => "ESC (Escape)".to_string(),
            0x1C => "FS (File Separator)".to_string(),
            0x1D => "GS (Group Separator)".to_string(),
            0x1E => "RS (Record Separator)".to_string(),
            0x1F => "US (Unit Separator)".to_string(),
            0x20 => "SP (Space)".to_string(),
            x if Self::RANGE_ASCII_PRINTABLE.contains(&x) => (c as char).to_string(),
            0x7F => "DEL (Delete)".to_string(),
            c => {
                if self.limit_to_base_ascii {
                    "extended ASCII".to_string()
                } else {
                    format!("{} (extended ASCII)", c as char)
                }
            }
        }
    }
}
