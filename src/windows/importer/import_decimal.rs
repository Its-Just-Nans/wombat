//! Import decimal data from a string

/// Parse a decimal number
fn parse_buffer(buf: &str, big_endian: bool, bytes: &mut Vec<u8>) -> Result<(), String> {
    // 1. Parse decimal
    let value: u64 = buf
        .parse()
        .map_err(|_| format!("invalid decimal number: {buf}"))?;

    // 2. Convert to minimal bytes
    let mut v = value;
    let mut temp = Vec::new();

    if v == 0 {
        temp.push(0u8);
    } else {
        while v > 0 {
            #[allow(clippy::cast_sign_loss)]
            temp.push((v & 0xFF) as u8);
            v >>= 8;
        }

        if big_endian {
            temp.reverse();
        }
    }

    // 3. Append to output
    bytes.extend_from_slice(&temp);
    Ok(())
}

/// Parse a decimal string
pub fn parse_decimal_string(input: &str, big_endian: bool) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut buf = String::new();

    for c in input.chars() {
        match c {
            // Separators
            ' ' | '\t' | '\n' | '\r' | '-' | ':' | ',' => {
                if !buf.is_empty() {
                    parse_buffer(&buf, big_endian, &mut bytes)?;
                    buf.clear();
                }
            }

            // Decimal digit
            '0'..='9' => buf.push(c),

            _ => return Err(format!("Invalid character in input: {c}")),
        }
    }

    // Final flush
    if !buf.is_empty() {
        parse_buffer(&buf, big_endian, &mut bytes)?;
    }

    Ok(bytes)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::windows::importer::parse_decimal_string;

    #[test]
    fn test_decimal_simple() {
        let s = "1 2 3";
        let expected = vec![1, 2, 3];
        assert_eq!(parse_decimal_string(s, true).unwrap(), expected);
    }

    #[test]
    fn test_decimal_simple_no_spaces() {
        let s = "123";
        let expected = vec![123];
        assert_eq!(parse_decimal_string(s, true).unwrap(), expected);
    }

    #[test]
    fn test_decimal_with_separators() {
        let s = "1:2-3,4\t5\n6";
        let expected = vec![1, 2, 3, 4, 5, 6];
        assert_eq!(parse_decimal_string(s, true).unwrap(), expected);
    }

    #[test]
    fn test_decimal_max_value() {
        let s = "255";
        let expected = vec![255];
        assert_eq!(parse_decimal_string(s, true).unwrap(), expected);
    }

    #[test]
    fn test_decimal_zero() {
        let s = "0";
        let expected = vec![0];
        assert_eq!(parse_decimal_string(s, true).unwrap(), expected);
    }

    #[test]
    fn test_decimal_multiple_bytes() {
        let s = "256";
        let expected = vec![0x01, 0x00];
        assert_eq!(parse_decimal_string(s, true).unwrap(), expected);

        let expected = vec![0x00, 0x01];
        assert_eq!(parse_decimal_string(s, false).unwrap(), expected);

        let s2 = "999";
        let expected = vec![0x03, 0xE7];
        assert_eq!(parse_decimal_string(s2, true).unwrap(), expected);

        let expected = vec![0xE7, 0x03];
        assert_eq!(parse_decimal_string(s2, false).unwrap(), expected);
    }

    #[test]
    fn test_decimal_invalid_char() {
        let s = "12a";
        assert!(parse_decimal_string(s, true).is_err());
    }

    #[test]
    fn test_decimal_empty() {
        let s = "";
        let expected: Vec<u8> = vec![];
        assert_eq!(parse_decimal_string(s, true).unwrap(), expected);
    }
}
