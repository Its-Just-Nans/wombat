//! Import hex data from a string

/// Parse a hex string
pub fn parse_hex_string(input: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut buf = String::with_capacity(2);

    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // Separators: flush if needed
            ' ' | '\t' | '\n' | '\r' | ':' | '-' | ',' => {
                if buf.len() == 1 {
                    buf.insert(0, '0');
                }
                if buf.len() == 2 {
                    let byte = u8::from_str_radix(&buf, 16)
                        .map_err(|_| format!("invalid hex byte: {buf}"))?;
                    bytes.push(byte);
                    buf.clear();
                }
            }

            // 0x prefix
            '0' => {
                if let Some('x' | 'X') = chars.peek() {
                    chars.next();
                } else {
                    buf.push(c);
                }
            }

            // \x escape
            '\\' => {
                if let Some('x' | 'X') = chars.peek() {
                    chars.next();
                } else {
                    return Err(format!(
                        "Invalid escape: \\{}",
                        chars.peek().unwrap_or(&'?')
                    ));
                }
            }

            // hex digit
            '0'..='9' | 'a'..='f' | 'A'..='F' => {
                buf.push(c);

                if buf.len() == 2 {
                    let byte = u8::from_str_radix(&buf, 16)
                        .map_err(|_| format!("invalid hex byte: {buf}"))?;
                    bytes.push(byte);
                    buf.clear();
                }
            }

            _ => return Err(format!("Invalid character in input: {c}")),
        }
    }

    // Final flush
    if buf.len() == 1 {
        buf.insert(0, '0');
    }

    if buf.len() == 2 {
        let byte = u8::from_str_radix(&buf, 16).map_err(|_| format!("invalid hex byte: {buf}"))?;
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
    fn test_basic_space_separated_solo() {
        let s = "45 1 0";
        let expected = vec![0x45, 0x01, 0x00];
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
        let expected = vec![0x45, 0x89, 0x45, 0x12, 0x04];
        assert_eq!(parse_hex_string(s).unwrap(), expected);
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
}
