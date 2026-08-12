use crate::common::{DictError, DictResult, Reader};
use crate::wordgram::{Aptree, parse_aptree};

pub const POS_COUNT: usize = 64;
pub const POS_UNI_FLOOR: f64 = -66.774_967_696_827;

#[derive(Debug, Clone, Copy)]
pub struct PosUniGram {
    pub freqs: [u32; POS_COUNT],
}

impl PosUniGram {
    #[must_use]
    pub fn freq(&self, tag: u8) -> u32 {
        if (b'0'..=b'o').contains(&tag) {
            self.freqs[(tag - b'0') as usize]
        } else {
            0
        }
    }
}

/// Parses the POS unigram frequency table.
///
/// # Errors
///
/// Returns an error if the size does not match `POS_COUNT * 4`.
pub fn parse_uni(data: &[u8]) -> DictResult<PosUniGram> {
    if data.len() != POS_COUNT * 4 {
        return Err(DictError::new(
            format!(
                "POSUniGram.bin size mismatch: {} (expected {})",
                data.len(),
                POS_COUNT * 4
            ),
            0,
        ));
    }
    let mut r = Reader::new(data);
    let mut freqs = [0u32; POS_COUNT];
    for f in &mut freqs {
        *f = r.u32()?;
    }
    Ok(PosUniGram { freqs })
}

#[derive(Debug, Clone)]
pub struct PosBigram {
    pub pairs: Vec<(u8, u8, f64)>,
    matrix: Vec<[f64; POS_COUNT]>,
}

impl PosBigram {
    /// Returns the bigram probability, or 0.0 for out-of-range tags.
    ///
    /// # Panics
    ///
    /// Panics if `POS_COUNT` does not fit in `u8`.
    #[must_use]
    pub fn get(&self, tag1: u8, tag2: u8) -> f64 {
        let pos_count = u8::try_from(POS_COUNT).expect("POS_COUNT fits u8");
        if tag1 < pos_count && tag2 < pos_count {
            self.matrix[tag1 as usize][tag2 as usize]
        } else {
            0.0
        }
    }
}

/// Parses the POS bigram table.
///
/// # Errors
///
/// Returns an error if the data is malformed or has trailing bytes.
pub fn parse_bi(data: &[u8]) -> DictResult<PosBigram> {
    let mut r = Reader::new(data);
    let n = r.u32()? as usize;
    let mut pairs = Vec::with_capacity(n);
    let mut matrix = vec![[0.0f64; POS_COUNT]; POS_COUNT];
    for _ in 0..n {
        let t1 = r.u8()?;
        let t2 = r.u8()?;
        let p = r.f64()?;
        if (t1 as usize) < POS_COUNT && (t2 as usize) < POS_COUNT {
            matrix[t1 as usize][t2 as usize] = p;
        }
        pairs.push((t1, t2, p));
    }
    if r.pos != data.len() {
        return Err(DictError::new(
            format!("POSBigram.bin trailing excess {} bytes", data.len() - r.pos),
            r.pos,
        ));
    }
    Ok(PosBigram { pairs, matrix })
}

#[derive(Debug)]
pub struct PosTrigram<'a> {
    pub aptree: Aptree<'a>,
}

impl PosTrigram<'_> {
    #[must_use]
    pub fn lookup(&self, key: &[u8]) -> Option<f32> {
        let i = self.aptree.search(key)?;
        if self.aptree.key_at(i) == key {
            Some(f32::from_bits(self.aptree.info[i].value))
        } else {
            None
        }
    }
}

/// Parses the POS trigram tree.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_tri(data: &[u8]) -> DictResult<PosTrigram<'_>> {
    let aptree = parse_aptree(data, 12, false)?;
    Ok(PosTrigram { aptree })
}

#[derive(Debug)]
pub struct PosNGram<'a> {
    pub uni: PosUniGram,
    pub bi: PosBigram,
    pub tri: PosTrigram<'a>,
}

/// Parses the combined POS n-gram dictionary.
///
/// # Errors
///
/// Returns an error if any part is malformed.
pub fn parse<'a>(uni: &'a [u8], bi: &'a [u8], tri: &'a [u8]) -> DictResult<PosNGram<'a>> {
    Ok(PosNGram {
        uni: parse_uni(uni)?,
        bi: parse_bi(bi)?,
        tri: parse_tri(tri)?,
    })
}

#[must_use]
pub fn get_kchar_count(s: &[u8]) -> usize {
    let mut n = 0;
    let mut i = 0;
    while i < s.len() {
        let c = s[i];
        if b"aeoui_89".contains(&c) {
            n += 1;
            i += 1;
        } else if b"yw".contains(&c) {
            n += 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    n
}

#[must_use]
pub fn get_word_prob(uni: &PosUniGram, word: &[u8], tag: u8, n_bindo: u32) -> f64 {
    let freq = uni.freq(tag);
    if freq == 0 {
        return POS_UNI_FLOOR;
    }
    if tag == b'k' {
        return 0.0;
    }
    let prob: f64 = if n_bindo >= 10 {
        if freq < n_bindo {
            0.98
        } else {
            f64::from(n_bindo) / f64::from(freq)
        }
    } else {
        let kchar = get_kchar_count(word);
        if is_k_symbol_pumsa(tag) {
            match tag {
                b'J' => 1.0 / 52.0,
                b'L' => 0.25,
                b'M' => 0.5,
                b'N' => 0.0,
                b'O' => 0.2,
                b'Q' => 1.0,
                _ => 0.1,
            }
        } else if tag.wrapping_add(0x96) > 2 {
            if kchar > 2 && tag == b'E' {
                1000.0 / f64::from(freq)
            } else {
                let nb = if n_bindo == 0 { 1 } else { n_bindo };
                f64::from(nb) / f64::from(freq)
            }
        } else {
            1.0
        }
    };
    prob.ln()
}

const fn is_k_symbol_pumsa(tag: u8) -> bool {
    matches!(
        tag,
        b'k' | b'I' | b'J' | b'L' | b'M' | b'N' | b'O' | b'P' | b'Q' | b'R' | b'S'
    )
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::float_cmp,
        reason = "oracle assertions use exact float equality"
    )]
    use super::*;
    use std::path::PathBuf;

    fn data_dir() -> PathBuf {
        PathBuf::from(
            std::env::var("KTTSDB_DIR")
                .expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
        )
        .join("KLangDic")
        .join("KMPADict")
    }

    fn load(name: &str) -> Vec<u8> {
        std::fs::read(data_dir().join(name)).unwrap_or_else(|e| panic!("{name}: {e}"))
    }

    #[test]
    fn uni_gram_counts() {
        let u = parse_uni(&load("POSUniGram.bin")).unwrap();
        assert_eq!(u.freqs.len(), 64);
        assert_eq!(u.freqs[0], 12_135_387);
        assert_eq!(u.freqs[1], 7_714_961);
        assert_eq!(u.freqs[2], 8_161_492);
        assert_eq!(u.freqs[9], 12_815_641);
        assert_eq!(u.freqs[12], 13_796_061);
        assert_eq!(u.freqs[16], 15_293_870);
        assert_eq!(u.freqs[47], 7_978_588);
        assert_eq!(u.freqs[58], 4_401_000);
        assert_eq!(u.freqs[59], 47_232_442);
        assert_eq!(u.freqs[60], 4_401_000);
        assert_eq!(u.freqs[62], 0);
        assert_eq!(u.freqs[63], 0);
        let sum: u64 = u.freqs.iter().map(|&f| u64::from(f)).sum();
        assert_eq!(sum, 203_586_131);
        assert_eq!(u.freqs.iter().filter(|&&f| f != 0).count(), 62);
    }

    #[test]
    fn bi_gram_pairs() {
        let b = parse_bi(&load("POSBigram.bin")).unwrap();
        assert_eq!(b.pairs.len(), 1198);
        let expected = [
            (0u8, 0u8, -2.1348f64),
            (0, 1, -2.6052),
            (0, 2, -7.8608),
            (0, 3, -7.4742),
            (0, 4, -5.4399),
        ];
        for (i, (t1, t2, p)) in expected.iter().enumerate() {
            let (a1, a2, ap) = b.pairs[i];
            assert_eq!((a1, a2), (*t1, *t2));
            assert!((ap - *p).abs() < 1e-3, "pairs[{i}] = {ap} (expected {p})");
        }
        assert!((b.get(0, 0) - (-2.1348)).abs() < 1e-3);
        assert_eq!(b.get(b'0', b'0'), 0.0);
        assert_eq!(b.get(62, 62), 0.0);
    }

    #[test]
    fn tri_gram_sizes() {
        let data = load("POSTrigram.bin");
        let t = parse_tri(&data).unwrap();
        assert_eq!(t.aptree.n_info, 9091);
        assert_eq!(t.aptree.n_tree, 18181);
        assert_eq!(t.aptree.key_len, 36364);
        for i in 0..t.aptree.n_info {
            let k = t.aptree.key_at(i);
            assert_eq!(k.len(), 3, "trigram key should be 3 chars: {k:?}");
        }
    }

    #[test]
    fn tri_gram_lookup() {
        let data = load("POSTrigram.bin");
        let t = parse_tri(&data).unwrap();
        let cases = [
            (b"j52".as_slice(), -3.8592f32),
            (b"6IX", -1.3706),
            (b"jIX", -1.2359),
            (b"kIT", -1.8086),
            (b"2>l", -9.7955),
        ];
        for (k, exp) in cases {
            let p = t
                .lookup(k)
                .unwrap_or_else(|| panic!("{} not found", String::from_utf8_lossy(k)));
            assert!(
                (p - exp).abs() < 1e-3,
                "{} = {p} (expected {exp})",
                String::from_utf8_lossy(k)
            );
        }
        let mut minp = f32::MAX;
        let mut zeros = 0;
        for e in &t.aptree.info {
            let p = f32::from_bits(e.value);
            minp = minp.min(p);
            if p == 0.0 {
                zeros += 1;
            }
        }
        assert!((minp - (-13.087)).abs() < 1e-2, "min = {minp}");
        assert_eq!(zeros, 28);
        assert!(t.lookup(b"zzz").is_none());
        for i in 0..t.aptree.n_info {
            let k = t.aptree.key_at(i);
            assert!(
                t.lookup(k).is_some(),
                "key {} does not roundtrip",
                String::from_utf8_lossy(k)
            );
        }
    }

    #[test]
    fn kchar_count() {
        assert_eq!(get_kchar_count(b"baNjeL"), 2);
        assert_eq!(get_kchar_count(b"useNju"), 3);
        assert_eq!(get_kchar_count(b"hye*gwa*c9"), 3);
        assert_eq!(get_kchar_count(b"daMvoGdaMvoG"), 4);
        assert_eq!(get_kchar_count(b""), 0);
        assert_eq!(get_kchar_count(b"kkkk"), 0);
    }

    #[test]
    fn word_prob_formula() {
        let u = parse_uni(&load("POSUniGram.bin")).unwrap();
        assert_eq!(get_word_prob(&u, b"", b'k', 0), 0.0);
        assert!((get_word_prob(&u, b"x", b'n', 2) - POS_UNI_FLOOR).abs() < 1e-9);
        let p = get_word_prob(&u, b"baNjeL", b'0', 2);
        assert!((p - (2.0f64 / 12_135_387.0).ln()).abs() < 1e-12);
        let p = get_word_prob(&u, b"x", b'0', 0);
        assert!((p - (1.0f64 / 12_135_387.0).ln()).abs() < 1e-12);
        let p = get_word_prob(&u, b"ga*seNlu", b'E', 1);
        assert!((p - (1000.0f64 / 3_501_716.0).ln()).abs() < 1e-12);
        let p = get_word_prob(&u, b"ga*", b'E', 1);
        assert!((p - (1.0f64 / 3_501_716.0).ln()).abs() < 1e-12);
        assert!((get_word_prob(&u, b"x", b'J', 1) - (1.0f64 / 52.0).ln()).abs() < 1e-12);
        assert!((get_word_prob(&u, b"x", b'I', 1) - 0.1f64.ln()).abs() < 1e-12);
        assert_eq!(get_word_prob(&u, b"x", b'N', 1), f64::NEG_INFINITY);
        assert!((get_word_prob(&u, b"x", b'P', 200) - 0.98f64.ln()).abs() < 1e-12);
        let p = get_word_prob(&u, b"x", b'0', 10);
        assert!((p - (10.0f64 / 12_135_387.0).ln()).abs() < 1e-12);
        assert_eq!(get_word_prob(&u, b"x", b'j', 1), 0.0);
        assert_eq!(get_word_prob(&u, b"x", b'l', 1), 0.0);
    }

    #[test]
    fn full_parse() {
        let u = load("POSUniGram.bin");
        let b = load("POSBigram.bin");
        let t = load("POSTrigram.bin");
        let n = parse(&u, &b, &t).unwrap();
        assert_eq!(n.uni.freqs.len(), 64);
        assert_eq!(n.bi.pairs.len(), 1198);
        assert_eq!(n.tri.aptree.n_info, 9091);
    }
}
