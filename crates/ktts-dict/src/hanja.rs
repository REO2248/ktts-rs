use crate::common::{DictError, DictResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hanja2Korea {
    pub table: Vec<u16>,
}

impl Hanja2Korea {
    pub const SIZE: usize = 0x10000;

    #[must_use]
    pub fn get(&self, cp: u16) -> u16 {
        self.table[cp as usize]
    }

    #[must_use]
    pub fn is_korea_hanja(&self, cp: u16) -> bool {
        self.table[cp as usize] != cp
    }

    /// Counts CJK code points whose Korean reading differs.
    ///
    /// # Panics
    ///
    /// Panics if a CJK code point does not fit in `u16` (impossible by range).
    #[must_use]
    pub fn non_identity_cjk_count(&self) -> usize {
        (0x4E00..=0x9FFF)
            .filter(|&cp| self.table[cp] != u16::try_from(cp).expect("CJK code point fits u16"))
            .count()
    }
}

/// Parses the hanja-to-Korean mapping table.
///
/// # Errors
///
/// Returns an error if the size does not match the fixed table size.
pub fn parse(data: &[u8]) -> DictResult<Hanja2Korea> {
    let expected = Hanja2Korea::SIZE * 2;
    if data.len() != expected {
        return Err(DictError::new(
            format!(
                "size mismatch: fixed {expected}B expected but measured {}B",
                data.len()
            ),
            0,
        ));
    }
    let mut table = Vec::with_capacity(Hanja2Korea::SIZE);
    for i in 0..Hanja2Korea::SIZE {
        table.push(u16::from_le_bytes([data[2 * i], data[2 * i + 1]]));
    }
    Ok(Hanja2Korea { table })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load() -> Vec<u8> {
        std::fs::read(crate::pronsec::test_data_dir().join("PronDict/unihanja2korea.bin"))
            .expect("cannot read unihanja2korea.bin")
    }

    #[test]
    fn parse_real() {
        let data = load();
        assert_eq!(data.len(), 131_072);
        let h = parse(&data).unwrap();
        assert_eq!(h.table.len(), 65_536);
        assert_eq!(h.get(0x0000), 0x0000);
        assert_eq!(h.get(0x0041), 0x0041);
        assert_eq!(h.get(0xAC00), 0xAC00);
        assert_eq!(h.get(0x4E00), 0xC77C);
        assert_eq!(h.get(0x4E8C), 0xC774);
        assert_eq!(h.get(0x4E09), 0xC0BC);
        assert_eq!(h.get(0x56D7), 0xAD6D);
        assert_eq!(h.non_identity_cjk_count(), 20_902);
        assert_eq!(h.get(0x4E01), 0xC815);
        assert!(h.table.contains(&65_535));
        let distinct: std::collections::HashSet<u16> = h.table.iter().copied().collect();
        assert_eq!(distinct.len(), 37_750);
        for cp in 0..128u16 {
            assert_eq!(h.get(cp), cp);
        }
    }

    #[test]
    fn is_korea_hanja_smoke() {
        let h = parse(&load()).unwrap();
        assert!(!h.is_korea_hanja('A' as u16));
        assert!(!h.is_korea_hanja(0xAC00));
        assert!(h.is_korea_hanja(0x4E00));
    }

    #[test]
    fn malformed_rejected() {
        assert!(parse(&[]).is_err());
        assert!(parse(&vec![0u8; 131_071]).is_err());
        assert!(parse(&vec![0u8; 131_073]).is_err());
    }
}
