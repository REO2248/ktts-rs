use crate::common::{DictError, DictResult, Reader};

pub const SRULEMORPH_SIZE: usize = 96;
pub const BIPROB_BUCKETS: usize = 16127;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleMorph {
    pub ch_root_pos_tag: i8,
    pub b_root_not_flag: u8,
    pub ch_end_pos_tag: i8,
    pub b_end_not_flag: u8,
    pub sw_string: [u16; 32],
    pub b_string_not_flag: u8,
    pub b_attrib_exist: u8,
    pub w_first_attrib_idx: i16,
    pub b_first_not_flag: u8,
    pub ch_first_cmp_str_idx: i8,
    pub w_second_attrib_idx: i16,
    pub b_second_not_flag: u8,
    pub ch_second_cmp_str_idx: i8,
    pub b_str_pos: u8,
    pub b_start_flag: u8,
    pub b_str_len: u8,
    pub b_over_flag: u8,
    pub b_bi_idx: u8,
    pub b_pos_not_flag: u8,
    pub ch_pos_array: [u8; 12],
}

impl RuleMorph {
    #[must_use]
    pub fn string(&self) -> Vec<u16> {
        self.sw_string
            .iter()
            .take_while(|&&c| c != 0)
            .copied()
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttribRecord {
    pub items: Vec<Vec<u16>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleRecord {
    pub morphs: Vec<RuleMorph>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BIRuleFile {
    pub w_rule_num: u16,
    pub w_attrib_num: u16,
    pub w_last: u16,
    pub w_begin: u16,
    pub attributes: Vec<AttribRecord>,
    pub rules: Vec<RuleRecord>,
}

impl BIRuleFile {
    #[must_use]
    pub fn attrib_item_count(&self) -> usize {
        self.attributes.iter().map(|a| a.items.len()).sum()
    }
    #[must_use]
    pub fn morph_count(&self) -> usize {
        self.rules.iter().map(|r| r.morphs.len()).sum()
    }
    #[must_use]
    pub fn bi_idx_histogram(&self) -> std::collections::BTreeMap<u8, usize> {
        let mut m = std::collections::BTreeMap::new();
        for r in &self.rules {
            for mo in &r.morphs {
                *m.entry(mo.b_bi_idx).or_insert(0) += 1;
            }
        }
        m
    }
}

/// Parses the BI rule dictionary.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_rule(data: &[u8]) -> DictResult<BIRuleFile> {
    let mut r = Reader::new(data);
    let _pad0 = r.u32()?;
    let _pad1 = r.u32()?;
    let w_rule_num = r.u16()?;
    let w_attrib_num = r.u16()?;
    let w_last = r.u16()?;
    let w_begin = r.u16()?;
    let mut attributes = Vec::with_capacity(w_attrib_num as usize);
    for _ in 0..w_attrib_num {
        let n = r.u32()?;
        let mut items = Vec::with_capacity(n as usize);
        for _ in 0..n {
            let len = r.u8()? as usize;
            let mut s = Vec::with_capacity(len);
            for _ in 0..len {
                s.push(r.u16()?);
            }
            items.push(s);
        }
        attributes.push(AttribRecord { items });
    }
    let mut rules = Vec::with_capacity(w_rule_num as usize);
    for _ in 0..w_rule_num {
        let n = r.u32()?;
        if n == 0 || n > 16 {
            return Err(DictError::new(
                format!("nRuleMorphNum abnormal value: {n} (expected 1..5)"),
                r.pos - 4,
            ));
        }
        let mut morphs = Vec::with_capacity(n as usize);
        for _ in 0..n {
            morphs.push(parse_rule_morph(&mut r)?);
        }
        rules.push(RuleRecord { morphs });
    }
    if r.remaining() != 0 {
        return Err(DictError::new(
            format!(
                "BIRule.bin full consumption failed: {}B left",
                r.remaining()
            ),
            r.pos,
        ));
    }
    Ok(BIRuleFile {
        w_rule_num,
        w_attrib_num,
        w_last,
        w_begin,
        attributes,
        rules,
    })
}

fn parse_rule_morph(r: &mut Reader<'_>) -> DictResult<RuleMorph> {
    let ch_root_pos_tag = i8::from_ne_bytes([r.u8()?]);
    let b_root_not_flag = r.u8()?;
    let ch_end_pos_tag = i8::from_ne_bytes([r.u8()?]);
    let b_end_not_flag = r.u8()?;
    let mut sw_string = [0u16; 32];
    for v in &mut sw_string {
        *v = r.u16()?;
    }
    let b_string_not_flag = r.u8()?;
    let b_attrib_exist = r.u8()?;
    let w_first_attrib_idx = i16::from_ne_bytes(r.u16()?.to_ne_bytes());
    let b_first_not_flag = r.u8()?;
    let ch_first_cmp_str_idx = i8::from_ne_bytes([r.u8()?]);
    let w_second_attrib_idx = i16::from_ne_bytes(r.u16()?.to_ne_bytes());
    let b_second_not_flag = r.u8()?;
    let ch_second_cmp_str_idx = i8::from_ne_bytes([r.u8()?]);
    let b_str_pos = r.u8()?;
    let b_start_flag = r.u8()?;
    let b_str_len = r.u8()?;
    let b_over_flag = r.u8()?;
    let b_bi_idx = r.u8()?;
    let b_pos_not_flag = r.u8()?;
    let mut ch_pos_array = [0u8; 12];
    for v in &mut ch_pos_array {
        *v = r.u8()?;
    }
    Ok(RuleMorph {
        ch_root_pos_tag,
        b_root_not_flag,
        ch_end_pos_tag,
        b_end_not_flag,
        sw_string,
        b_string_not_flag,
        b_attrib_exist,
        w_first_attrib_idx,
        b_first_not_flag,
        ch_first_cmp_str_idx,
        w_second_attrib_idx,
        b_second_not_flag,
        ch_second_cmp_str_idx,
        b_str_pos,
        b_start_flag,
        b_str_len,
        b_over_flag,
        b_bi_idx,
        b_pos_not_flag,
        ch_pos_array,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BIProbHashBin {
    pub buckets: Vec<u32>,
}

impl BIProbHashBin {
    #[must_use]
    pub fn non_zero_count(&self) -> usize {
        self.buckets.iter().filter(|&&b| b != 0).count()
    }
    #[must_use]
    pub fn max_offset(&self) -> u32 {
        self.buckets.iter().copied().max().unwrap_or(0)
    }
}

/// Parses the BI probability hash bin.
///
/// # Errors
///
/// Returns an error if the size does not match `BIPROB_BUCKETS * 4`.
pub fn parse_prob_bin(data: &[u8]) -> DictResult<BIProbHashBin> {
    if data.len() != BIPROB_BUCKETS * 4 {
        return Err(DictError::new(
            format!(
                "BIProb_hash.bin size mismatch: {} (expected {})",
                data.len(),
                BIPROB_BUCKETS * 4
            ),
            0,
        ));
    }
    let mut r = Reader::new(data);
    let mut buckets = Vec::with_capacity(BIPROB_BUCKETS);
    for _ in 0..BIPROB_BUCKETS {
        buckets.push(r.u32()?);
    }
    Ok(BIProbHashBin { buckets })
}

/// Computes the bucket index for a key.
///
/// # Panics
///
/// Panics if `BIPROB_BUCKETS` does not fit in `u32`.
#[must_use]
pub fn hash_value(key: &[u8]) -> u32 {
    let mut h = 0u32;
    for &c in key {
        h = h.wrapping_mul(31).wrapping_add(u32::from(c));
    }
    h % u32::try_from(BIPROB_BUCKETS).expect("BIPROB_BUCKETS fits u32")
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbRecord {
    pub bucket: usize,
    pub offset: usize,
    pub keys: Vec<Vec<u8>>,
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BIProbDic {
    pub records: Vec<ProbRecord>,
    pub key_count: usize,
}

impl BIProbDic {
    #[must_use]
    pub fn lookup(&self, key: &[u8]) -> Option<f32> {
        let b = hash_value(key) as usize;
        for rec in &self.records {
            if rec.bucket == b {
                for (k, v) in rec.keys.iter().zip(rec.values.iter()) {
                    if k == key {
                        return Some(*v);
                    }
                }
                return None;
            }
        }
        None
    }
}

/// Parses the BI probability dictionary using the hash bin.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
///
/// # Panics
///
/// Panics if a bucket offset does not fit in `usize`.
pub fn parse_prob_dic(data: &[u8], buckets: &[u32]) -> DictResult<BIProbDic> {
    let mut non_zero: Vec<(usize, usize)> = buckets
        .iter()
        .enumerate()
        .filter(|&(_, &b)| b != 0)
        .map(|(i, &b)| (i, b as usize - 1))
        .collect();
    non_zero.sort_by_key(|&(_, off)| off);
    let mut pos = 0usize;
    let mut records = Vec::with_capacity(non_zero.len());
    let mut key_count = 0usize;
    for (bucket_idx, off) in non_zero {
        if off != pos {
            return Err(DictError::new(
                format!(
                    "dic records not contiguous: expected offset {pos}, got {off} (bucket {bucket_idx})"
                ),
                off,
            ));
        }
        let b_str_count = *data
            .get(pos)
            .ok_or_else(|| DictError::new("bStrCount EOF", pos))?;
        pos += 1;
        let b_len = *data
            .get(pos)
            .ok_or_else(|| DictError::new("bLen EOF", pos))? as usize;
        pos += 1;
        let mut key_offs = Vec::with_capacity(b_str_count as usize);
        for _ in 0..b_str_count {
            key_offs.push(
                *data
                    .get(pos)
                    .ok_or_else(|| DictError::new("keyOff EOF", pos))?,
            );
            pos += 1;
        }
        let buf = data
            .get(pos..pos + b_len)
            .ok_or_else(|| DictError::new("schBuf EOF", pos))?;
        pos += b_len;
        let mut keys = Vec::with_capacity(b_str_count as usize);
        for &ko in &key_offs {
            if ko as usize >= buf.len() {
                return Err(DictError::new(
                    format!("keyOff {ko} exceeds schBuf (len {b_len}) (bucket {bucket_idx})"),
                    off,
                ));
            }
            let end = buf[ko as usize..]
                .iter()
                .position(|&b| b == 0)
                .map_or(buf.len(), |e| ko as usize + e);
            keys.push(buf[ko as usize..end].to_vec());
        }
        let mut values = Vec::with_capacity(b_str_count as usize);
        for _ in 0..b_str_count {
            let v = data
                .get(pos..pos + 4)
                .ok_or_else(|| DictError::new("float EOF", pos))?;
            pos += 4;
            values.push(f32::from_le_bytes([v[0], v[1], v[2], v[3]]));
        }
        for k in &keys {
            if hash_value(k) != u32::try_from(bucket_idx).expect("bucket_idx < 2^32") {
                return Err(DictError::new(
                    format!(
                        "hash mismatch: bucket {bucket_idx} has key {:?} (hash {})",
                        String::from_utf8_lossy(k),
                        hash_value(k)
                    ),
                    off,
                ));
            }
        }
        key_count += keys.len();
        records.push(ProbRecord {
            bucket: bucket_idx,
            offset: off,
            keys,
            values,
        });
    }
    if pos != data.len() {
        return Err(DictError::new(
            format!(
                "BIProb_hash.dic full consumption failed: {pos} != {}B",
                data.len()
            ),
            pos,
        ));
    }
    Ok(BIProbDic { records, key_count })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn break_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(
            std::env::var("KTTSDB_DIR")
                .expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
        )
        .join("KSpeechDic")
        .join("woman")
        .join("Break")
    }

    fn read(p: &str) -> Vec<u8> {
        std::fs::read(p).unwrap_or_else(|e| panic!("read failed {p}: {e}"))
    }

    #[test]
    fn birule_full_parse() {
        let data = read(&format!("{}/BIRule.bin", break_dir().display()));
        assert_eq!(data.len(), 33_451);
        let rule = parse_rule(&data).expect("BIRule.bin parse");
        assert_eq!(rule.w_rule_num, 132);
        assert_eq!(rule.w_attrib_num, 79);
        assert_eq!(rule.w_last, 130);
        assert_eq!(rule.w_begin, 3);
        assert_eq!(rule.attributes.len(), 79);
        assert_eq!(rule.attrib_item_count(), 453);
        assert_eq!(rule.rules.len(), 132);
        assert_eq!(rule.morph_count(), 304);
        let tagset: Vec<u16> = rule.attributes[64]
            .items
            .iter()
            .flatten()
            .copied()
            .collect();
        assert!(tagset.contains(&('@' as u16)));
        assert!(tagset.contains(&('C' as u16)));
        assert_eq!(rule.rules[0].morphs[0].ch_pos_array.len(), 12);
        let hist = rule.bi_idx_histogram();
        let expected: std::collections::BTreeMap<u8, usize> =
            [(0, 158), (1, 55), (2, 1), (3, 30), (4, 2), (6, 39), (7, 19)]
                .into_iter()
                .collect();
        assert_eq!(hist, expected);
    }

    #[test]
    fn birule_morph_fields() {
        let data = read(&format!("{}/BIRule.bin", break_dir().display()));
        let rule = parse_rule(&data).unwrap();
        let used: Vec<(u8, u8)> = rule
            .rules
            .iter()
            .flat_map(|r| r.morphs.iter())
            .filter(|m| m.b_str_len != 0)
            .map(|m| (m.b_str_len, m.b_over_flag))
            .collect();
        assert!(!used.is_empty());
        assert!(used.iter().all(|&(l, o)| (l == 4 || l == 5) && o == 1));
        let any_wild = rule.rules.iter().flat_map(|r| r.morphs.iter()).any(|m| {
            m.ch_root_pos_tag == i8::try_from(b'*').expect("ASCII fits i8")
                || m.ch_end_pos_tag == i8::try_from(b'*').expect("ASCII fits i8")
        });
        assert!(any_wild);
    }

    #[test]
    fn biprob_bin_parse() {
        let data = read(&format!("{}/BIProb_hash.bin", break_dir().display()));
        assert_eq!(data.len(), 64_508);
        let bin = parse_prob_bin(&data).expect("BIProb_hash.bin parse");
        assert_eq!(bin.buckets.len(), 16_127);
        assert_eq!(bin.non_zero_count(), 15_488);
        assert_eq!(bin.max_offset(), 651_339);
        assert_eq!(bin.buckets[0], 1);
        assert_eq!(hash_value(b""), 0);
        assert_eq!(hash_value(b"a"), 97);
        assert_eq!(hash_value(b"ab"), (97 * 31 + 98));
    }

    #[test]
    fn biprob_dic_full_parse() {
        let bin_data = read(&format!("{}/BIProb_hash.bin", break_dir().display()));
        let bin = parse_prob_bin(&bin_data).unwrap();
        let dic_data = read(&format!("{}/BIProb_hash.dic", break_dir().display()));
        assert_eq!(dic_data.len(), 651_362);
        let dic = parse_prob_dic(&dic_data, &bin.buckets).expect("BIProb_hash.dic parse");
        assert_eq!(dic.key_count, 56_153);
        assert_eq!(dic.records.len(), 15_488);
        for rec in &dic.records {
            for &v in &rec.values {
                assert!(
                    (0.0..=1.0).contains(&v),
                    "out of range: {v} (bucket {})",
                    rec.bucket
                );
            }
        }
        let mut transition = 0usize;
        let mut emission = 0usize;
        for rec in &dic.records {
            for k in &rec.keys {
                assert!(
                    matches!(k.first(), Some(b'p' | b'q' | b'r')),
                    "key does not start with level char: {:?}",
                    String::from_utf8_lossy(k)
                );
                match k.last() {
                    Some(b'p' | b'q' | b'r') => transition += 1,
                    _ => emission += 1,
                }
            }
        }
        assert_eq!(transition, 34_125);
        assert_eq!(emission, 22_028);
        let max_sz = dic.records.iter().map(|r| r.keys.len()).max().unwrap();
        assert_eq!(max_sz, 12);
        let rec0 = &dic.records[0];
        let k0 = &rec0.keys[0];
        assert_eq!(hash_value(k0) as usize, rec0.bucket);
        let v = dic.lookup(k0);
        assert_eq!(v, Some(rec0.values[0]));
        assert_eq!(dic.lookup(b"zzz-no-such-key"), None);
    }
}
