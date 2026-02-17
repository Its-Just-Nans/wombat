//! Import binary data from a string

/// Parse a binary string like "0b00000001 0b00000010" into Vec<u8>
pub fn parse_binary_string(input: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut buf = String::with_capacity(8);

    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // Separators
            ' ' | '\t' | '\n' | '\r' | ':' | '-' | ',' => {
                if !buf.is_empty() {
                    while buf.len() < 8 {
                        buf.insert(0, '0');
                    }

                    let byte = u8::from_str_radix(&buf, 2)
                        .map_err(|_| format!("invalid binary byte: {buf}"))?;
                    bytes.push(byte);
                    buf.clear();
                }
            }

            '0' => {
                if let Some('b' | 'B') = chars.peek() {
                    // 0b prefix
                    chars.next();

                    if !buf.is_empty() {
                        return Err("prefix inside byte".into());
                    }
                } else {
                    buf.push(c);

                    if buf.len() == 8 {
                        let byte = u8::from_str_radix(&buf, 2)
                            .map_err(|_| format!("invalid binary byte: {buf}"))?;
                        bytes.push(byte);
                        buf.clear();
                    }
                }
            }

            // \b escape
            '\\' => {
                if let Some('b' | 'B') = chars.peek() {
                    chars.next();

                    if !buf.is_empty() {
                        return Err("prefix inside byte".into());
                    }
                } else {
                    return Err(format!(
                        "Invalid escape: \\{}",
                        chars.peek().unwrap_or(&'?')
                    ));
                }
            }

            // Binary digit
            '1' => {
                buf.push(c);

                if buf.len() == 8 {
                    let byte = u8::from_str_radix(&buf, 2)
                        .map_err(|_| format!("invalid binary byte: {buf}"))?;
                    bytes.push(byte);
                    buf.clear();
                }
            }

            _ => return Err(format!("Invalid character in input: {c}")),
        }
    }

    // Final flush
    if !buf.is_empty() {
        while buf.len() < 8 {
            buf.insert(0, '0');
        }

        let byte =
            u8::from_str_radix(&buf, 2).map_err(|_| format!("invalid binary byte: {buf}"))?;
        bytes.push(byte);
    }

    Ok(bytes)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::windows::importer::parse_binary_string;

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
    fn test_binary_simple_solo() {
        let s = "1 1 0";
        let expected = vec![0x01, 0x01, 0x00];
        assert_eq!(parse_binary_string(s).unwrap(), expected);
    }

    #[test]
    fn test_binary_no_spaces() {
        let s = "000000010000001000000011";
        let expected = vec![1, 2, 3];
        assert_eq!(parse_binary_string(s).unwrap(), expected);
    }

    #[test]
    fn test_binary_no_spaces_with_1() {
        let s = "000000011000001000000011";
        let expected = vec![1, 130, 3];
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
}
