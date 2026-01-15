//! Windows

use bladvak::eframe::egui::ahash::{HashMap, HashMapExt};
use bladvak::eframe::egui::{self, RichText, TextEdit};
use bladvak::eframe::egui::{Color32, Widget};
use bladvak::errors::ErrorManager;
use bladvak::{BladvakApp, File};
use egui_plot::{Bar, BarChart, Legend, Plot};
use std::path::PathBuf;

use crate::WombatApp;

/// Histogram data
#[derive(Debug)]
pub(crate) struct Histogram {
    /// is open
    pub(crate) is_open: bool,
    /// histogram values
    data: Option<HashMap<u8, usize>>,
    /// vertical or horizontal
    vertical: bool,
}

impl Histogram {
    /// Create empty histogram
    pub(crate) fn new() -> Self {
        Self {
            is_open: false,
            data: None,
            vertical: false,
        }
    }

    /// Calculate the histogram
    fn calculate_histogram(binary_data: &[u8]) -> HashMap<u8, usize> {
        let mut map = binary_data.iter().fold(HashMap::new(), |mut map, &byte| {
            *map.entry(byte).or_insert(0) += 1;
            map
        });
        for idx in 0u8..=255 {
            map.entry(idx).or_insert(0);
        }
        map
    }

    /// Show the histogram ui
    fn ui(&mut self, binary_data: &[u8], ui: &mut egui::Ui, _error_manager: &mut ErrorManager) {
        if self.is_open {
            if self.data.is_some() {
                let mut is_open = self.is_open;
                egui::Window::new("Histogram")
                    .open(&mut is_open)
                    .vscroll(true)
                    .show(ui.ctx(), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Orientation:");
                            ui.selectable_value(&mut self.vertical, true, "Vertical");
                            ui.selectable_value(&mut self.vertical, false, "Horizontal");
                        });
                        self.show_plot(ui);
                    });
                self.is_open = is_open;
            } else {
                self.data = Some(Self::calculate_histogram(binary_data));
            }
        }
    }

    /// Show the plot
    pub fn show_plot(&self, ui: &mut egui::Ui) {
        let Some(data) = &self.data else {
            return;
        };
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut chart = BarChart::new(
            "Histogram",
            data.iter()
                .map(|(byte, value)| {
                    #[allow(clippy::cast_precision_loss)]
                    let value = *value as f64;
                    min = value.min(min);
                    max = value.max(max);
                    Bar::new(f64::from(*byte), value)
                })
                .collect(),
        )
        .color(egui::Color32::LIGHT_BLUE);

        if !self.vertical {
            chart = chart.horizontal();
        }

        let x_label = "bytes (0 to 255)";
        let y_label = format!("count (min:{min} max:{max})");
        Plot::new("Distribution")
            .legend(Legend::default())
            .x_axis_label(if self.vertical { x_label } else { &y_label })
            .y_axis_label(if self.vertical { &y_label } else { x_label })
            .clamp_grid(true)
            .allow_zoom(egui::Vec2b::new(true, true))
            // .allow_drag(egui::Vec2b::new(true, true))
            // .allow_scroll(egui::Vec2b::new(true, true))
            .show(ui, |plot_ui| plot_ui.bar_chart(chart));
    }
}

/// File info
#[derive(Debug)]
pub struct WindowsData {
    /// Histogram info
    pub(crate) histogram: Histogram,
    /// importer
    pub(crate) importer: Importer,
}

impl WindowsData {
    /// Create a new empty window data
    pub(crate) fn new() -> Self {
        Self {
            histogram: Histogram::new(),
            importer: Importer {
                is_open: false,
                value: String::new(),
                value_type: ImportType::String,
                import_error: None,
            },
        }
    }
}

/// import type
#[derive(Debug, PartialEq)]
pub(crate) enum ImportType {
    /// hex import
    Hex,
    /// string import
    String,
    /// binary import
    Binary,
    /// octal import
    Octal,
}

/// Histogram data
#[derive(Debug)]
pub(crate) struct Importer {
    /// is open
    pub(crate) is_open: bool,
    /// current value
    value: String,
    /// value type
    value_type: ImportType,
    /// import error
    import_error: Option<String>,
}

impl Importer {
    /// Import
    /// # Errors
    /// return error if fails to parse the `value`
    fn import(value: &str, value_type: &ImportType) -> Result<Vec<u8>, String> {
        match value_type {
            ImportType::String => Ok(value.as_bytes().to_vec()),
            ImportType::Hex => parse_hex_string(value),
            ImportType::Binary => parse_binary_string(value),
            ImportType::Octal => parse_octal_string(value),
        }
    }
    /// Show the importer ui
    fn ui(&mut self, ui: &mut egui::Ui, _error_manager: &mut ErrorManager) -> Option<Vec<u8>> {
        if self.is_open {
            let mut is_open = self.is_open;
            let mut ret = None;
            egui::Window::new("Import")
                .open(&mut is_open)
                .vscroll(true)
                .show(ui.ctx(), |ui| {
                    let previous_import_type = &self.value_type;
                    ui.horizontal(|ui| {
                        ui.label("Import from:");
                        ui.selectable_value(&mut self.value_type, ImportType::String, "String");
                        ui.selectable_value(&mut self.value_type, ImportType::Hex, "Hex");
                        ui.selectable_value(&mut self.value_type, ImportType::Binary, "Binary");
                        ui.selectable_value(&mut self.value_type, ImportType::Octal, "Octal");
                    });
                    if previous_import_type != self.value_type {
                        self.import_error = None;
                    }
                    if ui.button("Import").clicked() {
                        ret = Some(Self::import(&self.value, &self.value_type));
                    }
                    if let Some(err) = &self.import_error {
                        ui.label(RichText::new(err).color(Color32::LIGHT_RED));
                    }
                    if TextEdit::multiline(&mut self.value)
                        .min_size(ui.available_size())
                        .ui(ui)
                        .changed()
                    {
                        self.import_error = None;
                    }
                });
            self.is_open = is_open;
            if let Some(import_result) = ret {
                match import_result {
                    Ok(res) => return Some(res),
                    Err(import_err) => self.import_error = Some(import_err),
                }
            }
        }
        None
    }
}

impl WombatApp {
    /// Display windows
    pub(crate) fn ui_windows(&mut self, ui: &mut egui::Ui, error_manager: &mut ErrorManager) {
        self.windows_data
            .histogram
            .ui(&self.binary_file, ui, error_manager);
        if let Some(data) = self.windows_data.importer.ui(ui, error_manager)
            && let Err(e) = self.handle_file(File {
                data,
                path: PathBuf::from("imported.bin"),
            })
        {
            error_manager.add_error(e);
        }
    }
}

/// Parse a hex string
fn parse_hex_string(input: &str) -> Result<Vec<u8>, String> {
    // 1. Normalize input into a contiguous string of hex digits
    let mut hex_digits = String::with_capacity(input.len());

    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            // Skip common separators and whitespace
            ' ' | '\t' | '\n' | '\r' | ':' | '-' | ',' => {}

            // Handle 0x / 0X prefix
            '0' => {
                if let Some('x' | 'X') = chars.peek() {
                    chars.next(); // consume 'x' or 'X'
                } else {
                    hex_digits.push(c);
                }
            }

            // Handle \x escape
            '\\' => {
                if let Some('x' | 'X') = chars.peek() {
                    chars.next(); // consume 'x' or 'X'
                } else {
                    return Err(format!(
                        "Invalid escape: \\{}",
                        chars.peek().unwrap_or(&'?')
                    ));
                }
            }

            // Valid hex digit
            '0'..='9' | 'a'..='f' | 'A'..='F' => hex_digits.push(c),

            _ => return Err(format!("Invalid character in input: {c}")),
        }
    }

    // 2. Ensure even number of hex digits
    if !hex_digits.len().is_multiple_of(2) {
        return Err("hex string has odd number of digits".into());
    }

    // 3. Parse pairs into u8
    let bytes = (0..hex_digits.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex_digits[i..i + 2], 16)
                .map_err(|_| format!("invalid hex byte: {}", &hex_digits[i..i + 2]))
        })
        .collect::<Result<Vec<u8>, String>>()?;

    Ok(bytes)
}

/// Parse a binary string like "0b00000001 0b00000010" into Vec<u8>
pub fn parse_binary_string(input: &str) -> Result<Vec<u8>, String> {
    let mut digits = String::new();

    let mut chars = input.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | '\r' | ':' | '-' | ',' => {
                chars.next(); // skip separators
            }
            '0' => {
                chars.next();
                if let Some(&next) = chars.peek() {
                    if next == 'b' || next == 'B' {
                        chars.next(); // skip 'b' prefix
                    } else {
                        digits.push('0'); // standalone 0
                    }
                } else {
                    digits.push('0'); // last char
                }
            }
            '1' => {
                digits.push(c);
                chars.next();
            }
            _ => return Err(format!("invalid character in binary input: {c}")),
        }
    }

    if digits.is_empty() {
        return Ok(vec![]);
    }

    // Pad to multiple of 8 bits
    let pad = (8 - digits.len() % 8) % 8;
    digits = "0".repeat(pad) + &digits;

    // Convert each 8-bit chunk to u8
    let mut bytes = Vec::new();
    for i in (0..digits.len()).step_by(8) {
        let byte = u8::from_str_radix(&digits[i..i + 8], 2)
            .map_err(|_| format!("invalid binary byte: {}", &digits[i..i + 8]))?;
        bytes.push(byte);
    }

    Ok(bytes)
}

/// Parse an octal string like "0o44 0o77" into Vec<u8>
pub fn parse_octal_string(input: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();

    // separators: whitespace + :, -, ,
    let separators = |c: char| c.is_whitespace() || c == ':' || c == '-' || c == ',';

    for token in input.split(separators).filter(|t| !t.is_empty()) {
        // Remove optional prefix
        let token = token
            .strip_prefix("0o")
            .or_else(|| token.strip_prefix("0O"))
            .unwrap_or(token);

        // Parse the full octal number
        let byte =
            u8::from_str_radix(token, 8).map_err(|_| format!("invalid octal number: {token}"))?;
        bytes.push(byte);
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::parse_hex_string;

    #[test]
    fn test_basic_space_separated() {
        let s = "45 89 45 12 45";
        let expected = vec![0x45, 0x89, 0x45, 0x12, 0x45];
        assert_eq!(parse_hex_string(s).unwrap(), expected);
    }

    #[test]
    fn test_no_separators() {
        let s = "4589451245";
        let expected = vec![0x45, 0x89, 0x45, 0x12, 0x45];
        assert_eq!(parse_hex_string(s).unwrap(), expected);
    }

    #[test]
    fn test_0x_prefix() {
        let s = "0x45 0x84";
        let expected = vec![0x45, 0x84];
        assert_eq!(parse_hex_string(s).unwrap(), expected);

        let s2 = "0x450x84";
        assert_eq!(parse_hex_string(s2).unwrap(), expected);
    }

    #[test]
    fn test_colon_and_dash_separators() {
        let s = "45:89:45:12:45";
        let expected = vec![0x45, 0x89, 0x45, 0x12, 0x45];
        assert_eq!(parse_hex_string(s).unwrap(), expected);

        let s2 = "45-89-45-12-45";
        assert_eq!(parse_hex_string(s2).unwrap(), expected);
    }

    #[test]
    fn test_comma_separators() {
        let s = "45,89,45,12,45";
        let expected = vec![0x45, 0x89, 0x45, 0x12, 0x45];
        assert_eq!(parse_hex_string(s).unwrap(), expected);
    }

    #[test]
    fn test_backslash_x_escape() {
        let s = r"\x45\x89\x45\x12\x45";
        let expected = vec![0x45, 0x89, 0x45, 0x12, 0x45];
        assert_eq!(parse_hex_string(s).unwrap(), expected);
    }

    #[test]
    fn test_mixed_case_hex() {
        let s = "45 89 aF 12 45";
        let expected = vec![0x45, 0x89, 0xAF, 0x12, 0x45];
        assert_eq!(parse_hex_string(s).unwrap(), expected);
    }

    #[test]
    fn test_invalid_characters() {
        let s = "45 89 GG 12";
        assert!(parse_hex_string(s).is_err());

        let s2 = "45 89 4z";
        assert!(parse_hex_string(s2).is_err());
    }

    #[test]
    fn test_odd_number_of_digits() {
        let s = "458945124";
        assert!(parse_hex_string(s).is_err());
    }

    #[test]
    fn test_empty_string() {
        let s = "";
        let expected: Vec<u8> = vec![];
        assert_eq!(parse_hex_string(s).unwrap(), expected);
    }

    #[test]
    fn test_only_separators() {
        let s = " ,:- \t\n";
        let expected: Vec<u8> = vec![];
        assert_eq!(parse_hex_string(s).unwrap(), expected);
    }
    use super::{parse_binary_string, parse_octal_string};

    // =====================
    // Binary parser tests
    // =====================

    #[test]
    fn test_binary_simple_prefixed() {
        // Binary
        let s = "0b00000001 0b00000010 00000011";
        let expected = vec![1, 2, 3];
        assert_eq!(parse_binary_string(s).unwrap(), expected);
    }

    #[test]
    fn test_binary_simple_prefixed_concat() {
        // Binary
        let s = "0b000000010b000000100b00000011";
        let expected = vec![1, 2, 3];
        assert_eq!(parse_binary_string(s).unwrap(), expected);
    }

    #[test]
    fn test_binary_simple() {
        let s = "00000001 00000010 00000011";
        let expected = vec![1, 2, 3];
        assert_eq!(parse_binary_string(s).unwrap(), expected);
    }

    #[test]
    fn test_binary_no_spaces() {
        let s = "000000010000001000000011";
        let expected = vec![1, 2, 3];
        assert_eq!(parse_binary_string(s).unwrap(), expected);
    }

    #[test]
    fn test_binary_with_separators() {
        let s = "00000001:00000010-00000011,00000100\t00000101\n00000110";
        let expected = vec![1, 2, 3, 4, 5, 6];
        assert_eq!(parse_binary_string(s).unwrap(), expected);
    }

    #[test]
    fn test_binary_odd_padding() {
        let s = "1"; // single bit → padded to 8 bits
        let expected = vec![1];
        assert_eq!(parse_binary_string(s).unwrap(), expected);
    }

    #[test]
    fn test_binary_empty() {
        let s = "";
        let expected: Vec<u8> = vec![];
        assert_eq!(parse_binary_string(s).unwrap(), expected);
    }

    #[test]
    fn test_binary_invalid_char() {
        let s = "00000001 00000002";
        assert!(parse_binary_string(s).is_err());

        let s2 = "0000000a";
        assert!(parse_binary_string(s2).is_err());
    }

    // =====================
    // Octal parser tests
    // =====================

    #[test]
    fn test_octal_simple_prefixed() {
        // octal
        let s = "0o44 0o7 0o12";
        let expected = vec![36, 7, 10];
        assert_eq!(parse_octal_string(s).unwrap(), expected);
    }

    #[test]
    fn test_octal_simple() {
        let s = "123 77 7";
        let expected = vec![83, 63, 7];
        let bytes = parse_octal_string(s).unwrap();
        assert_eq!(bytes, expected); // just ensure it parses without panic
    }

    #[test]
    fn test_octal_with_separators() {
        let s = "123:77-7,0 1\t2\n3";
        let expected = vec![83, 63, 7, 0, 1, 2, 3];
        let bytes = parse_octal_string(s).unwrap();
        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_octal_empty() {
        let s = "";
        let expected: Vec<u8> = vec![];
        assert_eq!(parse_octal_string(s).unwrap(), expected);
    }

    #[test]
    fn test_octal_invalid_char() {
        let s = "128 456"; // 8 is invalid in octal
        assert!(parse_octal_string(s).is_err());

        let s2 = "123a"; // a is invalid
        assert!(parse_octal_string(s2).is_err());
    }

    #[test]
    fn test_octal_padding() {
        let s = "7"; // single octal digit -> 3 bits, padded to 8 bits
        let bytes = parse_octal_string(s).unwrap();
        assert_eq!(bytes.len(), 1);
    }
}
