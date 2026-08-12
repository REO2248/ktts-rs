use crate::common::{DictError, DictResult, Reader};

#[derive(Debug, Clone, Copy)]
pub struct AptreeInfo {
    pub key_offset: u32,
    pub value: u32,
    pub next: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct AptreeNode {
    pub value: u32,
    pub left: i32,
    pub right: i32,
}

#[derive(Debug)]
pub struct Aptree<'a> {
    pub n_info: usize,
    pub n_tree: usize,
    pub key_len: usize,
    pub keys: &'a [u8],
    pub wide: bool,
    pub info: Vec<AptreeInfo>,
    pub nodes: Vec<AptreeNode>,
}

impl Aptree<'_> {
    #[must_use]
    pub fn key_at(&self, idx: usize) -> &[u8] {
        let info = &self.info[idx];
        let start = info.key_offset as usize;
        let end = self.keys[start..]
            .iter()
            .position(|&b| b == 0)
            .map_or(self.keys.len(), |p| start + p);
        &self.keys[start..end]
    }

    #[must_use]
    pub fn key_w_at(&self, idx: usize) -> &[u8] {
        let info = &self.info[idx];
        let mut end = info.key_offset as usize * 2;
        while end + 1 < self.keys.len() && !(self.keys[end] == 0 && self.keys[end + 1] == 0) {
            end += 2;
        }
        &self.keys[info.key_offset as usize * 2..end]
    }

    #[must_use]
    /// Searches the tree for a byte key.
    ///
    /// # Panics
    ///
    /// Panics if a tree index is negative.
    pub fn search(&self, key: &[u8]) -> Option<usize> {
        if self.n_tree == 0 {
            return None;
        }
        let mut cur = self.n_tree - 1;
        loop {
            let n = &self.nodes[cur];
            if n.left == -1 || n.right == -1 {
                return Some(usize::try_from(n.value).expect("leaf value is non-negative"));
            }
            cur = if test_bit(key, n.value) {
                usize::try_from(n.right).expect("node index is non-negative")
            } else {
                usize::try_from(n.left).expect("node index is non-negative")
            };
        }
    }

    #[must_use]
    /// Searches the tree for a wide (UTF-16) key.
    ///
    /// # Panics
    ///
    /// Panics if a tree index is negative.
    pub fn search_w(&self, key: &[u16]) -> Option<usize> {
        if self.n_tree == 0 {
            return None;
        }
        let mut cur = self.n_tree - 1;
        loop {
            let n = &self.nodes[cur];
            if n.left == -1 || n.right == -1 {
                return Some(usize::try_from(n.value).expect("leaf value is non-negative"));
            }
            cur = if test_bit_w(key, n.value) {
                usize::try_from(n.right).expect("node index is non-negative")
            } else {
                usize::try_from(n.left).expect("node index is non-negative")
            };
        }
    }
}

#[must_use]
pub fn test_bit(key: &[u8], bit: u32) -> bool {
    let byte = key.get((bit >> 3) as usize).copied().unwrap_or(0);
    (byte & (0x80 >> (bit & 7))) != 0
}

#[must_use]
pub fn test_bit_w(key: &[u16], bit: u32) -> bool {
    let w = key.get((bit >> 4) as usize).copied().unwrap_or(0);
    (w & (0x8000 >> (bit & 15))) != 0
}

pub(crate) fn parse_aptree(data: &[u8], info_size: usize, wide: bool) -> DictResult<Aptree<'_>> {
    if info_size != 12 && info_size != 8 {
        return Err(DictError::new("info_size must be 8 or 12", 0));
    }
    let mut r = Reader::new(data);
    let n_info = r.u32()? as usize;
    let n_tree = r.u32()? as usize;
    let key_len = r.u32()? as usize;
    let key_bytes = key_len * if wide { 2 } else { 1 };
    let keys = r.bytes(key_bytes)?;
    let mut info = Vec::with_capacity(n_info);
    for _ in 0..n_info {
        let b = r.bytes(info_size)?;
        if info_size == 12 {
            info.push(AptreeInfo {
                key_offset: u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
                value: u32::from_le_bytes([b[4], b[5], b[6], b[7]]),
                next: i32::from_le_bytes([b[8], b[9], b[10], b[11]]),
            });
        } else {
            info.push(AptreeInfo {
                key_offset: u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
                value: 0,
                next: i32::from_le_bytes([b[4], b[5], b[6], b[7]]),
            });
        }
    }
    let mut nodes = Vec::with_capacity(n_tree);
    for _ in 0..n_tree {
        let b = r.bytes(12)?;
        nodes.push(AptreeNode {
            value: u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            left: i32::from_le_bytes([b[4], b[5], b[6], b[7]]),
            right: i32::from_le_bytes([b[8], b[9], b[10], b[11]]),
        });
    }
    if r.pos != data.len() {
        return Err(DictError::new(
            format!("trailing excess {} bytes", data.len() - r.pos),
            r.pos,
        ));
    }
    for (i, e) in info.iter().enumerate() {
        let start = e.key_offset as usize * if wide { 2 } else { 1 };
        if start >= keys.len() {
            return Err(DictError::new(
                format!(
                    "info[{i}] key_offset {} exceeds key array length {}",
                    e.key_offset,
                    keys.len()
                ),
                0,
            ));
        }
        if wide {
            let mut end = start;
            while end + 1 < keys.len() && !(keys[end] == 0 && keys[end + 1] == 0) {
                end += 2;
            }
            if end + 1 >= keys.len() {
                return Err(DictError::new(
                    format!("info[{i}] wide key is not 0x0000-terminated"),
                    0,
                ));
            }
        } else if !keys[start..].contains(&0) {
            return Err(DictError::new(
                format!("info[{i}] key is not NUL-terminated"),
                0,
            ));
        }
    }
    Ok(Aptree {
        n_info,
        n_tree,
        key_len,
        keys,
        wide,
        info,
        nodes,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordPattern {
    pub prob: f32,
    pub pyogi: [Vec<u8>; 3],
    pub pos: [u8; 3],
}

#[derive(Debug)]
pub struct WordTriGram<'a> {
    pub aptree: Aptree<'a>,
    pub dic_offsets: Vec<u32>,
    pub patterns: Vec<Vec<WordPattern>>,
}

impl WordTriGram<'_> {
    #[must_use]
    pub fn lookup(&self, key: &[u8]) -> Option<usize> {
        let i = self.aptree.search(key)?;
        if self.aptree.key_at(i) == key {
            Some(i)
        } else {
            None
        }
    }
}

/// Parses the word trigram dictionary.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse<'a>(bin: &'a [u8], dic: &'a [u8]) -> DictResult<WordTriGram<'a>> {
    let aptree = parse_aptree(bin, 12, false)?;
    let dic_offsets: Vec<u32> = aptree.info.iter().map(|e| e.value).collect();
    let mut patterns = Vec::with_capacity(dic_offsets.len());
    for &o in &dic_offsets {
        patterns.push(parse_patterns(dic, o as usize)?);
    }
    Ok(WordTriGram {
        aptree,
        dic_offsets,
        patterns,
    })
}

#[allow(clippy::needless_range_loop)]
/// Parses the word patterns starting at the given offset.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_patterns(dic: &[u8], offset: usize) -> DictResult<Vec<WordPattern>> {
    if offset >= dic.len() {
        return Err(DictError::new("offset exceeds file size", offset));
    }
    let mut r = Reader::new(&dic[offset..]);
    let n = r.u8()? as usize;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let prob = r.f32()?;
        let mut pyogi = [Vec::new(), Vec::new(), Vec::new()];
        for s in 0..3 {
            let len = r.u8()? as usize;
            pyogi[s] = r.bytes(len)?.to_vec();
        }
        let mut pos = [0u8; 3];
        for s in 0..3 {
            pos[s] = r.u8()?;
        }
        out.push(WordPattern { prob, pyogi, pos });
    }
    if offset + r.pos > dic.len() {
        return Err(DictError::new("record exceeds file size", offset));
    }
    Ok(out)
}

#[derive(Debug)]
pub struct CharGram<'a> {
    pub aptree: Aptree<'a>,
}

impl CharGram<'_> {
    #[must_use]
    pub fn lookup(&self, key: &[u16]) -> Option<f32> {
        let i = self.aptree.search_w(key)?;
        if self.aptree.key_w_at(i) == u16_slice_bytes(key) {
            Some(f32::from_bits(self.aptree.info[i].value))
        } else {
            None
        }
    }
}

/// Parses the hanja character bigram tree.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_hc(data: &[u8]) -> DictResult<CharGram<'_>> {
    let aptree = parse_aptree(data, 12, true)?;
    Ok(CharGram { aptree })
}

#[derive(Debug)]
pub struct NameGram<'a> {
    pub aptree: Aptree<'a>,
}

impl NameGram<'_> {
    #[must_use]
    pub fn contains(&self, key: &[u16]) -> bool {
        self.aptree
            .search_w(key)
            .is_some_and(|i| self.aptree.key_w_at(i) == u16_slice_bytes(key))
    }
}

/// Parses the hanja name trigram tree.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_hkname(data: &[u8]) -> DictResult<NameGram<'_>> {
    let aptree = parse_aptree(data, 8, true)?;
    Ok(NameGram { aptree })
}

fn u16_slice_bytes(key: &[u16]) -> Vec<u8> {
    let mut v = Vec::with_capacity(key.len() * 2);
    for &w in key {
        v.extend_from_slice(&w.to_le_bytes());
    }
    v
}

#[cfg(test)]
mod tests {
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

    fn u16s(hex_pairs: &[u16]) -> Vec<u16> {
        hex_pairs.to_vec()
    }

    #[test]
    fn word_trigram_bin_sizes() {
        let data = load("WordTriGram.bin");
        let t = parse_aptree(&data, 12, false).unwrap();
        assert_eq!(t.n_info, 126);
        assert_eq!(t.n_tree, 251);
        assert_eq!(t.key_len, 731);
        let keys: Vec<&[u8]> = (0..t.n_info).map(|i| t.key_at(i)).collect();
        assert_eq!(keys.len(), 126);
        for k in [
            b"3ya".as_slice(),
            b"3wi",
            b"3t8",
            b"3sudo",
            b"3seaN",
            b"2yuyeN",
        ] {
            assert!(
                keys.contains(&k),
                "key {} does not exist",
                String::from_utf8_lossy(k)
            );
        }
        for k in &keys {
            assert!(
                k.first().is_some_and(|c| c.is_ascii_digit() && *c != b'0'),
                "key {} does not start with a digit",
                String::from_utf8_lossy(k)
            );
        }
    }

    #[test]
    fn word_trigram_lookup_and_dic() {
        let bin = load("WordTriGram.bin");
        let dic = load("WordTriGram.dic");
        let t = parse(&bin, &dic).unwrap();
        let i = t.lookup(b"3ya").expect("3ya not found");
        assert_eq!(t.dic_offsets[i], 2306);
        let pats = &t.patterns[i];
        assert_eq!(pats.len(), 1);
        assert!((pats[0].prob - (-10.0)).abs() < 1e-4);
        assert_eq!(pats[0].pyogi[0], b"");
        assert_eq!(pats[0].pyogi[1], b"");
        assert_eq!(pats[0].pyogi[2], b"ya");
        assert_eq!(pats[0].pos, [0, b'g', b'^']);

        let i = t.lookup(b"3sudo").expect("3sudo not found");
        assert_eq!(t.dic_offsets[i], 2263);
        assert!((t.patterns[i][0].prob - (-20.0)).abs() < 1e-4);
        assert_eq!(t.patterns[i][0].pyogi[1], b"iL");

        let i = t.lookup(b"2yuyeN").expect("2yuyeN not found");
        assert_eq!(t.dic_offsets[i], 1279);

        assert!(t.lookup(b"3zzzz").is_none());

        for (i, &o) in t.dic_offsets.iter().enumerate() {
            let k = t.aptree.key_at(i);
            let j = t
                .lookup(k)
                .unwrap_or_else(|| panic!("key {} does not roundtrip", String::from_utf8_lossy(k)));
            assert_eq!(j, i);
            let _ = o;
        }
    }

    #[test]
    fn word_trigram_dic_records_contiguous() {
        let bin = load("WordTriGram.bin");
        let dic = load("WordTriGram.dic");
        let t = parse(&bin, &dic).unwrap();
        let mut offs: Vec<usize> = t.dic_offsets.iter().map(|&o| o as usize).collect();
        offs.sort_unstable();
        let mut prev_end = None;
        for &o in &offs {
            if let Some(pe) = prev_end {
                assert_eq!(pe, o, "gap between records @{pe:#x} → @{o:#x}");
            }
            let mut r = Reader::new(&dic[o..]);
            let n = r.u8().unwrap() as usize;
            for _ in 0..n {
                r.f32().unwrap();
                for _ in 0..3 {
                    let len = r.u8().unwrap() as usize;
                    r.bytes(len).unwrap();
                }
                r.bytes(3).unwrap();
            }
            prev_end = Some(o + r.pos);
        }
        assert_eq!(prev_end, Some(2319));
    }

    #[test]
    fn hc_bigram_sizes_and_roundtrip() {
        let data = load("HCBigram.bin");
        let h = parse_hc(&data).unwrap();
        assert_eq!(h.aptree.n_info, 11529);
        assert_eq!(h.aptree.n_tree, 23057);
        assert_eq!(h.aptree.key_len, 34587);
        for i in 0..h.aptree.n_info {
            let k = h.aptree.key_w_at(i);
            assert_eq!(k.len(), 4);
        }
        let mut minp = f32::MAX;
        let mut maxp = f32::MIN;
        let mut zeros = 0;
        for i in 0..h.aptree.n_info {
            let k = h.aptree.key_w_at(i);
            let ks: Vec<u16> = k
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            let p = h
                .lookup(&ks)
                .unwrap_or_else(|| panic!("key {k:02x?} does not roundtrip"));
            minp = minp.min(p);
            maxp = maxp.max(p);
            if p == 0.0 {
                zeros += 1;
            }
        }
        assert!(minp < -4.9 && maxp > 1.9, "prob range {minp}..{maxp}");
        assert_eq!(zeros, 260);
    }

    #[test]
    fn hk_namegram_sizes_and_roundtrip() {
        let data = load("HKNamegram.bin");
        let h = parse_hkname(&data).unwrap();
        assert_eq!(h.aptree.n_info, 54078);
        assert_eq!(h.aptree.n_tree, 108_155);
        assert_eq!(h.aptree.key_len, 215_334);
        let mut dist = [0usize; 5];
        for i in 0..h.aptree.n_info {
            let k = h.aptree.key_w_at(i);
            dist[k.len() / 2] += 1;
        }
        assert_eq!(&dist[..], &[0, 0, 979, 53098, 1]);
        for i in 0..h.aptree.n_info {
            let k = h.aptree.key_w_at(i);
            let ks: Vec<u16> = k
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            assert!(h.contains(&ks), "key {k:02x?} does not roundtrip");
        }
        assert!(!h.contains(&u16s(&[0xFFFF, 0xFFFF])));
        let k0 = h.aptree.key_w_at(0);
        assert!(k0[0] == b'N' || k0[0] == b'F' || k0[0] == b'R' || k0[0] == b'L');
    }
}
