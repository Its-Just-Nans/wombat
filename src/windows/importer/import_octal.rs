//! Import octal data from a string

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
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::windows::importer::parse_octal_string;

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
    fn test_octal_simple_solo() {
        let s = "1 1 0";
        let expected = vec![0o001, 0o001, 0o000];
        assert_eq!(parse_octal_string(s).unwrap(), expected);
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
