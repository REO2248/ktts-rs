use crate::common::{DictError, DictResult, Reader};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PronTable {
    data: Vec<u8>,
    entries: Vec<(u32, u8)>,
}

impl PronTable {
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn pron(&self, i: usize) -> Option<&[u8]> {
        let (off, len) = *self.entries.get(i)?;
        let s = off as usize;
        self.data.get(s + 1..s + 1 + len as usize)
    }

    #[must_use]
    pub fn index_of(&self, off: u32) -> Option<usize> {
        self.entries.binary_search_by_key(&off, |&(o, _)| o).ok()
    }
}

/// Parses the English pronunciation table.
///
/// # Errors
///
/// Returns an error if a pronunciation entry extends beyond EOF.
///
/// # Panics
///
/// Panics if a pronunciation offset does not fit in `u32`.
pub fn parse_pron_table(data: &[u8]) -> DictResult<PronTable> {
    let mut pos = 0usize;
    let mut entries = Vec::new();
    while pos < data.len() {
        let len = *data
            .get(pos)
            .ok_or_else(|| DictError::new("pron entry length beyond EOF", pos))?;
        let end = pos + 1 + len as usize;
        if end > data.len() {
            return Err(DictError::new(
                format!("pron entry length {len} beyond EOF (pos={pos})"),
                pos,
            ));
        }
        entries.push((u32::try_from(pos).expect("pron entry offset fits u32"), len));
        pos = end;
    }
    Ok(PronTable {
        data: data.to_vec(),
        entries,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub offset: usize,
    pub count: u8,
    pub words: Vec<Vec<u8>>,
    pub pron_offsets: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct HashDic {
    data: Vec<u8>,
    pub blocks: Vec<Block>,
}

impl HashDic {
    #[must_use]
    pub const fn len(&self) -> usize {
        self.blocks.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    #[must_use]
    pub fn word_count(&self) -> usize {
        self.blocks.iter().map(|b| b.count as usize).sum()
    }
}

/// Parses the English pronunciation hash dictionary.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_hash_dic(data: &[u8]) -> DictResult<HashDic> {
    let mut blocks = Vec::new();
    let mut pos = 0usize;
    while pos < data.len() {
        let cnt = *data
            .get(pos)
            .ok_or_else(|| DictError::new("block cnt beyond EOF", pos))?;
        let tot = *data
            .get(pos + 1)
            .ok_or_else(|| DictError::new("block tot beyond EOF", pos + 1))?;
        let count = usize::from(cnt);
        let offs_end = pos + 2 + count;
        let strs_end = offs_end + tot as usize;
        let tail_end = strs_end + 4 * count;
        if tail_end > data.len() {
            return Err(DictError::new(
                format!("block @{pos}: cnt={cnt} tot={tot} beyond EOF"),
                pos,
            ));
        }
        let mut words = Vec::with_capacity(count);
        let mut prev = 0usize;
        for i in 0..count {
            let off = data[pos + 2 + i] as usize;
            if (i == 0 && off != 0) || (i > 0 && off < prev) {
                return Err(DictError::new(
                    format!("block @{pos}: word {i} offset {off} is invalid (prev={prev})"),
                    pos + 2 + i,
                ));
            }
            if off >= tot as usize {
                return Err(DictError::new(
                    format!("block @{pos}: word {i} offset {off} exceeds string section {tot}B"),
                    pos + 2 + i,
                ));
            }
            prev = off;
        }
        for i in 0..count {
            let off = data[pos + 2 + i] as usize;
            let str_pos = offs_end + off;
            let body = data
                .get(str_pos..strs_end)
                .ok_or_else(|| DictError::new("word string beyond EOF", str_pos))?;
            let nul = body
                .iter()
                .position(|&b| b == 0)
                .ok_or_else(|| DictError::new("word without NUL terminator", str_pos))?;
            if nul == 0 {
                return Err(DictError::new("empty word", str_pos));
            }
            words.push(body[..nul].to_vec());
        }
        let mut pron_offsets = Vec::with_capacity(count);
        for i in 0..count {
            let off = u32::from_le_bytes([
                data[strs_end + 4 * i],
                data[strs_end + 4 * i + 1],
                data[strs_end + 4 * i + 2],
                data[strs_end + 4 * i + 3],
            ]);
            pron_offsets.push(off);
        }
        blocks.push(Block {
            offset: pos,
            count: cnt,
            words,
            pron_offsets,
        });
        pos = tail_end;
    }
    Ok(HashDic {
        data: data.to_vec(),
        blocks,
    })
}

#[derive(Debug, Clone)]
pub struct HashBin {
    pub buckets: Vec<u32>,
}

impl HashBin {
    #[must_use]
    pub const fn len(&self) -> usize {
        self.buckets.len()
    }
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.buckets.is_empty()
    }
}

/// Parses the English pronunciation hash bin.
///
/// # Errors
///
/// Returns an error if the size is not a multiple of 4.
pub fn parse_hash_bin(data: &[u8]) -> DictResult<HashBin> {
    if !data.len().is_multiple_of(4) {
        return Err(DictError::new(
            format!("hash table size {} is not a multiple of 4", data.len()),
            0,
        ));
    }
    let mut buckets = Vec::with_capacity(data.len() / 4);
    let mut r = Reader::new(data);
    for _ in 0..data.len() / 4 {
        buckets.push(r.u32()?);
    }
    Ok(HashBin { buckets })
}

#[must_use]
pub fn get_hash_value(word: &[u8]) -> u32 {
    const MOD: u32 = 16127;
    let mut h: u32 = 0;
    for &c in word {
        let sc = i32::from(i8::from_ne_bytes([c]));
        h = h
            .wrapping_mul(31)
            .wrapping_add(u32::from_ne_bytes(sc.to_ne_bytes()));
    }
    h % MOD
}

#[derive(Debug, Clone)]
pub struct EnglishPyogiSet {
    table: PronTable,
    hash_dic: HashDic,
    hash_bin: HashBin,
    off_to_idx: HashMap<u32, usize>,
}

impl EnglishPyogiSet {
    /// Builds the set, validating that every pronunciation offset is known.
    ///
    /// # Errors
    ///
    /// Returns an error if a pronunciation offset has no matching table entry.
    pub fn new(table: PronTable, hash_dic: HashDic, hash_bin: HashBin) -> DictResult<Self> {
        let mut off_to_idx = HashMap::with_capacity(table.len());
        for (i, &(off, _)) in table.entries.iter().enumerate() {
            off_to_idx.insert(off, i);
        }
        for b in &hash_dic.blocks {
            for &o in &b.pron_offsets {
                if !off_to_idx.contains_key(&o) {
                    return Err(DictError::new(
                        format!(
                            "block @{}: pron offset {o} does not exist in englishpyogi.dic",
                            b.offset
                        ),
                        b.offset,
                    ));
                }
            }
        }
        Ok(Self {
            table,
            hash_dic,
            hash_bin,
            off_to_idx,
        })
    }

    #[must_use]
    pub const fn table(&self) -> &PronTable {
        &self.table
    }
    #[must_use]
    pub const fn hash_dic(&self) -> &HashDic {
        &self.hash_dic
    }
    #[must_use]
    pub const fn hash_bin(&self) -> &HashBin {
        &self.hash_bin
    }

    #[must_use]
    pub fn lookup(&self, word: &[u8]) -> Option<&[u8]> {
        let hash = get_hash_value(word) as usize;
        let off = *self.hash_bin.buckets.get(hash)?;
        if off == 0 {
            return None;
        }
        let pos = (off - 1) as usize;
        let dic_data = &self.hash_dic.data;
        let cnt = *dic_data.get(pos)? as usize;
        let tot = *dic_data.get(pos + 1)? as usize;
        if cnt == 0 {
            return None;
        }
        let tail_end = pos + 2 + cnt + tot + 4 * cnt;
        if tail_end > dic_data.len() {
            return None;
        }
        let strs_off = pos + 2 + cnt;
        for i in 0..cnt {
            let off = dic_data[pos + 2 + i] as usize;
            let str_pos = strs_off + off;
            let body = &dic_data[str_pos..strs_off + tot];
            let nul = body.iter().position(|&b| b == 0)?;
            if &body[..nul] == word {
                let poff = u32::from_le_bytes([
                    dic_data[strs_off + tot + 4 * i],
                    dic_data[strs_off + tot + 4 * i + 1],
                    dic_data[strs_off + tot + 4 * i + 2],
                    dic_data[strs_off + tot + 4 * i + 3],
                ]);
                let idx = *self.off_to_idx.get(&poff)?;
                return self.table.pron(idx);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(rel: &str) -> Vec<u8> {
        std::fs::read(crate::pronsec::test_data_dir().join(rel))
            .unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
    }

    fn load_set() -> EnglishPyogiSet {
        let table = parse_pron_table(&load("EngDict/englishpyogi.dic")).expect("dic parse failed");
        let hash_dic =
            parse_hash_dic(&load("EngDict/englishpyogi_hash.dic")).expect("hash.dic parse failed");
        let hash_bin =
            parse_hash_bin(&load("EngDict/englishpyogi_hash.bin")).expect("hash.bin parse failed");
        EnglishPyogiSet::new(table, hash_dic, hash_bin).expect("3-file set construction failed")
    }

    #[test]
    fn counts_match_report() {
        let table = parse_pron_table(&load("EngDict/englishpyogi.dic")).unwrap();
        let hash_dic = parse_hash_dic(&load("EngDict/englishpyogi_hash.dic")).unwrap();
        let hash_bin = parse_hash_bin(&load("EngDict/englishpyogi_hash.bin")).unwrap();
        assert_eq!(table.len(), 105_160);
        assert_eq!(hash_dic.len(), 16_101);
        assert_eq!(hash_dic.word_count(), 105_160);
        assert_eq!(hash_bin.len(), 16_127);
        assert_eq!(hash_bin.buckets.iter().filter(|&&b| b == 0).count(), 26);
        assert_eq!(table.data.len(), 1_602_423);
        assert_eq!(hash_dic.data.len(), 1_434_493);
        assert_eq!(hash_bin.buckets.len() * 4, 64_508);
        let max_cnt = hash_dic.blocks.iter().map(|b| b.count).max().unwrap();
        assert_eq!(max_cnt, 20);
        for b in &hash_dic.blocks {
            assert!((1..=20).contains(&b.count), "block word count {}", b.count);
        }
        let _ = EnglishPyogiSet::new(table, hash_dic, hash_bin).unwrap();
    }

    #[test]
    fn hash_function_values() {
        assert_eq!(get_hash_value(b"degarmo"), 0);
        assert_eq!(get_hash_value(b"superman"), 4);
        assert_eq!(get_hash_value(b"absorbing"), 4);
        assert_eq!(get_hash_value(b"winners"), 5);
        assert_eq!(get_hash_value(b"zees"), get_hash_value(b"zees"));
        assert_eq!(get_hash_value(b""), 0);
    }

    #[test]
    fn full_scan_zero_miss() {
        let set = load_set();
        let mut tested = 0usize;
        for b in &set.hash_dic.blocks {
            for w in &b.words {
                let pron = set
                    .lookup(w)
                    .unwrap_or_else(|| panic!("{} not found", String::from_utf8_lossy(w)));
                assert!(
                    pron.iter().all(|c| c.is_ascii() && !c.is_ascii_control()),
                    "non-ASCII pron: {}",
                    String::from_utf8_lossy(w)
                );
                assert!(!pron.is_empty());
                tested += 1;
            }
        }
        assert_eq!(tested, 105_160);
        println!("englishpyogi full scan: {tested}/105,160 words 0 misses");
    }

    #[test]
    fn spot_checks() {
        let set = load_set();
        let cases: &[(&[u8], &str)] = &[
            (b"degarmo", "d ih g aa r m ow"),
            (b"superman", "s uw p er m ah n"),
            (b"absorbing", "ah b z ao r b ih ng"),
            (b"winners", "w ih n er z"),
            (b"zzzz", "z iy z"),
            (b"a", "ah"),
        ];
        for (w, expect) in cases {
            let pron = set
                .lookup(w)
                .unwrap_or_else(|| panic!("{} missing", String::from_utf8_lossy(w)));
            assert_eq!(
                std::str::from_utf8(pron).unwrap(),
                *expect,
                "pron of {}",
                String::from_utf8_lossy(w)
            );
        }
        assert!(set.lookup(b"zzzzzzzzzzzzzzzzzzzz").is_none());
        assert!(set.lookup(b"").is_none());
    }

    #[test]
    fn malformed_rejected() {
        let mut bad = vec![5u8, b'a', b'b'];
        bad.extend(std::iter::repeat_n(b'c', 10));
        bad.push(200);
        assert!(parse_pron_table(&bad).is_err());
        assert!(parse_hash_dic(&[8, 59, 0, 1, 2, 3]).is_err());
        assert!(parse_hash_bin(&[1, 2, 3]).is_err());
        assert!(parse_hash_dic(&[2, 4, 5, 0, b'a', 0, b'b', 0, 0, 0, 0, 0, 0, 0, 0]).is_err());
    }
}
