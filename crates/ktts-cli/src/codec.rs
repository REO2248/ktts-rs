#[must_use]
pub fn utf8_to_u16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

#[must_use]
pub fn u16_to_utf8(units: &[u16]) -> String {
    String::from_utf16_lossy(units)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hangul_roundtrip() {
        let text = "안녕하세요";
        let units = utf8_to_u16(text);
        assert_eq!(units.len(), 5);
        assert_eq!(units[0], 0xC548);
        assert_eq!(u16_to_utf8(&units), text);
    }

    #[test]
    fn surrogate_pair_roundtrip() {
        let text = "a😀b";
        let units = utf8_to_u16(text);
        assert_eq!(units.len(), 4);
        assert_eq!(units[1], 0xD83D);
        assert_eq!(units[2], 0xDE00);
        assert_eq!(u16_to_utf8(&units), text);
    }

    #[test]
    fn mixed_content() {
        let text = "Hello 안녕 123!";
        let units = utf8_to_u16(text);
        assert_eq!(u16_to_utf8(&units), text);
    }
}
