use crate::common::{DictError, DictResult, Reader};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PronRule {
    pub cond: [u8; 12],
    pub apply: [u8; 8],
}

impl PronRule {
    #[must_use]
    pub fn cond_sequences(&self) -> Vec<&[u8]> {
        let mut seqs = Vec::new();
        let mut pos = 0usize;
        while pos < 12 {
            let rest = &self.cond[pos..];
            if let Some(z) = rest.iter().position(|&b| b == 0) {
                seqs.push(&rest[..z]);
                pos += z + 1;
            } else {
                seqs.push(rest);
                pos = 12;
            }
            if pos < 12 && self.cond[pos..].iter().all(|&b| b == 0xCC) {
                break;
            }
        }
        seqs
    }

    #[allow(clippy::needless_range_loop)]
    #[must_use]
    pub fn apply_pairs(&self) -> [(u8, u8); 4] {
        let mut out = [(0u8, 0u8); 4];
        for i in 0..4 {
            out[i] = (self.apply[2 * i], self.apply[2 * i + 1]);
        }
        out
    }

    #[must_use]
    pub const fn apply_source(&self) -> (u8, u8) {
        (self.apply[0], self.apply[1])
    }

    #[must_use]
    pub fn apply_rest(&self) -> &[u8] {
        &self.apply[2..]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronRuleDict {
    pub count: u32,
    pub rules: Vec<PronRule>,
}

/// Parses the pronunciation rule dictionary.
///
/// # Errors
///
/// Returns an error if the data is malformed or the size does not match.
pub fn parse(data: &[u8]) -> DictResult<PronRuleDict> {
    let mut r = Reader::new(data);
    let count = r.u32()?;
    let expected = 4usize
        .checked_add(count as usize * 20)
        .ok_or_else(|| DictError::new("count overflow", 0))?;
    if expected != data.len() {
        return Err(DictError::new(
            format!(
                "size mismatch: header count={count} should give {expected}B but measured {}B",
                data.len()
            ),
            0,
        ));
    }
    let mut rules = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let raw = r.bytes(20)?;
        let mut cond = [0u8; 12];
        let mut apply = [0u8; 8];
        cond.copy_from_slice(&raw[..12]);
        apply.copy_from_slice(&raw[12..]);
        if !cond.contains(&0) {
            return Err(DictError::new(
                format!("rule {i}: condition has no NUL terminator ({cond:02x?})"),
                4 + i * 20,
            ));
        }
        rules.push(PronRule { cond, apply });
    }
    if r.remaining() != 0 {
        return Err(DictError::new(
            format!("full consumption failed: {}B left", r.remaining()),
            r.pos,
        ));
    }
    Ok(PronRuleDict { count, rules })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pronrule_path() -> std::path::PathBuf {
        crate::pronsec::test_data_dir().join("PronDict/pronrule.bin")
    }

    #[test]
    fn parse_real_pronrule() {
        let data = std::fs::read(pronrule_path()).expect("cannot read pronrule.bin");
        let dict = parse(&data).expect("parse failed");
        assert_eq!(dict.count, 43);
        assert_eq!(dict.rules.len(), 43);
        assert_eq!(data.len(), 864);
    }

    #[test]
    fn cond_sequences_structure() {
        let data = std::fs::read(pronrule_path()).expect("cannot read pronrule.bin");
        let dict = parse(&data).unwrap();
        let mut one = 0;
        let mut two = 0;
        for (i, rule) in dict.rules.iter().enumerate() {
            let seqs = rule.cond_sequences();
            assert!(
                (1..=3).contains(&seqs.len()),
                "rule {i}: column count {}",
                seqs.len()
            );
            for s in &seqs {
                assert!(!s.is_empty(), "rule {i}: empty column");
                for &b in *s {
                    assert!(
                        b != 0 && b != 0xCC,
                        "rule {i}: terminator/padding byte in column {b:#x}"
                    );
                }
            }
            let mut pos = 0usize;
            for s in &seqs {
                assert_eq!(&rule.cond[pos..pos + s.len()], *s);
                pos += s.len();
                assert_eq!(rule.cond[pos], 0);
                pos += 1;
            }
            assert!(rule.cond[pos..].iter().all(|&b| b == 0xCC));
            match seqs.len() {
                1 => one += 1,
                2 => two += 1,
                3 => {}
                _ => unreachable!(),
            }
        }
        assert_eq!(one, 6);
        assert_eq!(two, 15);
        assert_eq!(
            dict.rules[0].cond_sequences(),
            vec![&[0x0b, 0x0d, 0x01, 0x07, 0x1d, 0x01][..]]
        );
        assert_eq!(
            dict.rules[42].cond_sequences(),
            vec![&[0x0b, 0x03, 0x01, 0x07, 0x03, 0x11][..], &[0x1b, 0x01][..]]
        );
        println!("condition 1 column: {one} rules / 2 columns: {two} rules (43 total)");
    }

    #[test]
    fn apply_structure() {
        let data = std::fs::read(pronrule_path()).expect("cannot read pronrule.bin");
        let dict = parse(&data).unwrap();
        assert_eq!(
            dict.rules[0].apply,
            [0x30, 0x03, 0x00, 0x01, 0x30, 0x02, 0x33, 0x01]
        );
        assert_eq!(dict.rules[0].apply_source(), (0x30, 0x03));
        assert_eq!(
            dict.rules[2].apply,
            [0x30, 0x03, 0x00, 0x01, 0x30, 0x01, 0x31, 0x01]
        );
        assert_eq!(
            dict.rules[42].apply,
            [0x30, 0x03, 0x00, 0x01, 0x39, 0x03, 0x00, 0x02]
        );
        for (i, rule) in dict.rules.iter().enumerate() {
            assert!(!rule.apply.contains(&0xCC), "rule {i}: 0xCC in apply part");
        }
        for (i, rule) in dict.rules.iter().enumerate() {
            let (_, flag) = rule.apply_source();
            assert!([1, 2, 3].contains(&flag), "rule {i}: kind {flag:#x}");
        }
    }

    #[test]
    fn malformed_inputs_rejected() {
        assert!(parse(&[]).is_err());
        assert!(parse(&[0x2b, 0, 0, 0]).is_err());
        let mut bad = vec![43u8, 0, 0, 0];
        bad.extend(std::iter::repeat_n(0u8, 43 * 20 - 20));
        assert!(parse(&bad).is_err());
        let mut bad2 = vec![1u8, 0, 0, 0];
        bad2.extend(std::iter::repeat_n(0x11u8, 20));
        assert!(parse(&bad2).is_err());
    }
}
