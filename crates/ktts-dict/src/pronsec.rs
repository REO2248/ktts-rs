use crate::common::{DictError, DictResult, Reader};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolUnit {
    Bytes,
    U16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordValue {
    PoolRef(u32),
    Code(u8),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record {
    pub key_ref: u32,
    pub value: RecordValue,
    pub next: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeNode {
    pub value: u32,
    pub left: i32,
    pub right: i32,
}

#[derive(Debug, Clone)]
pub struct SectionDict {
    pub header: Vec<u32>,
    pub key_pool: Vec<u8>,
    pub value_pool: Vec<u8>,
    pub key_unit: PoolUnit,
    pub value_unit: PoolUnit,
    pub records: Vec<Record>,
    pub tree: Vec<TreeNode>,
    pub tree_before_records: bool,
}

impl SectionDict {
    #[must_use]
    pub const fn num_records(&self) -> usize {
        self.records.len()
    }

    #[must_use]
    pub const fn num_tree_nodes(&self) -> usize {
        self.tree.len()
    }

    #[must_use]
    pub const fn root_index(&self) -> usize {
        self.tree.len().saturating_sub(1)
    }

    #[must_use]
    pub const fn key_pool_bytes(&self) -> usize {
        self.key_pool.len()
    }

    #[must_use]
    pub fn key_bytes(&self, rec: usize) -> Option<&[u8]> {
        let r = self.records.get(rec)?;
        let unit = match self.key_unit {
            PoolUnit::Bytes => 1usize,
            PoolUnit::U16 => 2usize,
        };
        let start = (r.key_ref as usize).checked_mul(unit)?;
        let pool = self.key_pool.get(start..)?;
        match self.key_unit {
            PoolUnit::Bytes => {
                let end = pool.iter().position(|&b| b == 0)?;
                Some(&pool[..end])
            }
            PoolUnit::U16 => {
                let mut end = 0usize;
                while end + 1 < pool.len() {
                    if pool[end] == 0 && pool[end + 1] == 0 {
                        return Some(&pool[..end]);
                    }
                    end += 2;
                }
                None
            }
        }
    }

    #[must_use]
    pub fn key_string(&self, rec: usize) -> Option<String> {
        let b = self.key_bytes(rec)?;
        Some(match self.key_unit {
            PoolUnit::U16 => String::from_utf16_lossy(
                &b.chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect::<Vec<_>>(),
            ),
            PoolUnit::Bytes => String::from_utf8_lossy(b).into_owned(),
        })
    }

    #[must_use]
    pub fn value_string(&self, rec: usize) -> Option<String> {
        let r = self.records.get(rec)?;
        let RecordValue::PoolRef(v) = r.value else {
            return None;
        };
        let unit = match self.value_unit {
            PoolUnit::Bytes => 1usize,
            PoolUnit::U16 => 2usize,
        };
        let start = (v as usize).checked_mul(unit)?;
        let pool = self.value_pool.get(start..)?;
        match self.value_unit {
            PoolUnit::Bytes => {
                let end = pool.iter().position(|&b| b == 0)?;
                Some(String::from_utf8_lossy(&pool[..end]).into_owned())
            }
            PoolUnit::U16 => {
                let mut end = 0usize;
                while end + 1 < pool.len() {
                    if pool[end] == 0 && pool[end + 1] == 0 {
                        return Some(String::from_utf16_lossy(
                            &pool[..end]
                                .chunks_exact(2)
                                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                                .collect::<Vec<_>>(),
                        ));
                    }
                    end += 2;
                }
                None
            }
        }
    }

    #[must_use]
    pub fn code(&self, rec: usize) -> Option<u8> {
        match self.records.get(rec)?.value {
            RecordValue::Code(c) => Some(c),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Spec {
    n_header: usize,
    key_unit: PoolUnit,
    value_unit: PoolUnit,
    has_value_pool: bool,
    tree_before_records: bool,
    validate_tree_ptrs: bool,
}

#[expect(
    clippy::too_many_lines,
    reason = "faithful C port of the section parser"
)]
fn parse_section(data: &[u8], spec: Spec) -> DictResult<SectionDict> {
    let mut r = Reader::new(data);
    let mut header = Vec::with_capacity(spec.n_header);
    for _ in 0..spec.n_header {
        header.push(r.u32()?);
    }
    let num = header[0] as usize;
    let ntree = header[1] as usize;
    let key_len = header[2] as usize;
    let key_bytes = key_len
        .checked_mul(match spec.key_unit {
            PoolUnit::Bytes => 1,
            PoolUnit::U16 => 2,
        })
        .ok_or_else(|| DictError::new("key pool length overflow", 0))?;
    let (value_bytes, value_len) = if spec.has_value_pool {
        let vlen = header[3] as usize;
        let vb = vlen
            .checked_mul(match spec.value_unit {
                PoolUnit::Bytes => 1,
                PoolUnit::U16 => 2,
            })
            .ok_or_else(|| DictError::new("value pool length overflow", 0))?;
        (vb, Some(vlen))
    } else {
        (0, None)
    };

    let expected = 4 * spec.n_header + key_bytes + value_bytes + num * 12 + ntree * 12;
    if expected != data.len() {
        return Err(DictError::new(
            format!(
                "size mismatch: computed {expected}B vs measured {}B (num={num}, ntree={ntree}, key={key_bytes}B, val={value_bytes}B)",
                data.len()
            ),
            0,
        ));
    }

    let key_pool = r.bytes(key_bytes)?.to_vec();
    let value_pool = if spec.has_value_pool {
        r.bytes(value_bytes)?.to_vec()
    } else {
        Vec::new()
    };
    let after_pools = r.pos;
    let (recs_off, tree_off) = if spec.tree_before_records {
        (after_pools + ntree * 12, after_pools)
    } else {
        (after_pools, after_pools + num * 12)
    };
    let _ = value_len;

    let mut records = Vec::with_capacity(num);
    for i in 0..num {
        let raw = data
            .get(recs_off + i * 12..recs_off + (i + 1) * 12)
            .ok_or_else(|| DictError::new("record beyond EOF", recs_off + i * 12))?;
        let key_ref = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
        let value = if spec.has_value_pool {
            let v = u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]);
            if v == u32::MAX {
                RecordValue::None
            } else {
                RecordValue::PoolRef(v)
            }
        } else {
            let code = raw[4];
            let pad = &raw[5..8];
            if pad != [0xCD, 0xCD, 0xCD] && !pad.iter().all(|&b| b == 0xCD) {
                return Err(DictError::new(
                    format!("record {i}: abnormal padding after code {pad:02x?}"),
                    recs_off + i * 12,
                ));
            }
            RecordValue::Code(code)
        };
        let next = i32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]);
        let unit = match spec.key_unit {
            PoolUnit::Bytes => 1usize,
            PoolUnit::U16 => 2usize,
        };
        let kstart = (key_ref as usize)
            .checked_mul(unit)
            .ok_or_else(|| DictError::new("key_ref overflow", recs_off + i * 12))?;
        if kstart >= key_pool.len() {
            return Err(DictError::new(
                format!("record {i}: key_ref {key_ref} exceeds key pool {key_bytes}B"),
                recs_off + i * 12,
            ));
        }
        records.push(Record {
            key_ref,
            value,
            next,
        });
    }

    r.pos = tree_off;
    let mut tree = Vec::with_capacity(ntree);
    for i in 0..ntree {
        let raw = r.bytes(12)?;
        let value = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
        let left = i32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]);
        let right = i32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]);
        for (name, p) in [("left", left), ("right", right)] {
            if spec.validate_tree_ptrs
                && p != -1
                && (p < 0 || usize::try_from(p).expect("p >= 0") >= ntree)
            {
                return Err(DictError::new(
                    format!("tree node {i}: {name}={p} out of range (ntree={ntree})"),
                    tree_off + i * 12,
                ));
            }
        }
        tree.push(TreeNode { value, left, right });
    }
    r.pos = after_pools + num * 12 + ntree * 12;
    if r.remaining() != 0 {
        return Err(DictError::new(
            format!("full consumption failed: {}B left", r.remaining()),
            r.pos,
        ));
    }
    Ok(SectionDict {
        header,
        key_pool,
        value_pool,
        key_unit: spec.key_unit,
        value_unit: spec.value_unit,
        records,
        tree,
        tree_before_records: spec.tree_before_records,
    })
}

/// Parses the `strpron` section dictionary.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_strpron(data: &[u8]) -> DictResult<SectionDict> {
    parse_section(
        data,
        Spec {
            n_header: 4,
            key_unit: PoolUnit::U16,
            value_unit: PoolUnit::Bytes,
            has_value_pool: true,
            tree_before_records: false,
            validate_tree_ptrs: true,
        },
    )
}

/// Parses the `prepron` section dictionary.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_prepron(data: &[u8]) -> DictResult<SectionDict> {
    parse_section(
        data,
        Spec {
            n_header: 3,
            key_unit: PoolUnit::U16,
            value_unit: PoolUnit::Bytes,
            has_value_pool: false,
            tree_before_records: false,
            validate_tree_ptrs: true,
        },
    )
}

/// Parses the `unipron` section dictionary.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_unipron(data: &[u8]) -> DictResult<SectionDict> {
    parse_section(
        data,
        Spec {
            n_header: 4,
            key_unit: PoolUnit::U16,
            value_unit: PoolUnit::Bytes,
            has_value_pool: true,
            tree_before_records: false,
            validate_tree_ptrs: true,
        },
    )
}

/// Parses the `morphmodify` section dictionary.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_morphmodify(data: &[u8]) -> DictResult<SectionDict> {
    parse_section(
        data,
        Spec {
            n_header: 3,
            key_unit: PoolUnit::Bytes,
            value_unit: PoolUnit::Bytes,
            has_value_pool: false,
            tree_before_records: false,
            validate_tree_ptrs: true,
        },
    )
}

/// Parses the `unienglishpron` section dictionary.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_unienglishpron(data: &[u8]) -> DictResult<SectionDict> {
    parse_section(
        data,
        Spec {
            n_header: 4,
            key_unit: PoolUnit::Bytes,
            value_unit: PoolUnit::Bytes,
            has_value_pool: true,
            tree_before_records: false,
            validate_tree_ptrs: true,
        },
    )
}

/// Parses the `engsym` section dictionary.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_engsym(data: &[u8]) -> DictResult<SectionDict> {
    parse_section(
        data,
        Spec {
            n_header: 3,
            key_unit: PoolUnit::Bytes,
            value_unit: PoolUnit::Bytes,
            has_value_pool: false,
            tree_before_records: false,
            validate_tree_ptrs: true,
        },
    )
}

/// Parses the `user` section dictionary.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_user(data: &[u8]) -> DictResult<SectionDict> {
    parse_section(
        data,
        Spec {
            n_header: 4,
            key_unit: PoolUnit::U16,
            value_unit: PoolUnit::U16,
            has_value_pool: true,
            tree_before_records: true,
            validate_tree_ptrs: false,
        },
    )
}

#[cfg(test)]
pub(crate) fn test_data_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(
        std::env::var("KTTSDB_DIR").expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
    )
    .join("KLangDic")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(rel: &str) -> Vec<u8> {
        std::fs::read(test_data_dir().join(rel))
            .unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
    }

    fn leaf_count(d: &SectionDict) -> usize {
        d.tree
            .iter()
            .filter(|n| n.left == -1 && n.right == -1)
            .count()
    }

    #[test]
    fn strpron_real() {
        let d = parse_strpron(&load("PronDict/strpron.bin")).unwrap();
        assert_eq!(d.num_records(), 121);
        assert_eq!(d.num_tree_nodes(), 241);
        assert_eq!(leaf_count(&d), 121);
        assert_eq!(d.root_index(), 240);
        for i in 0..120 {
            assert_eq!(
                d.records[i].next,
                i32::try_from(i).expect("index fits i32") + 1
            );
        }
        assert_eq!(d.records[120].next, -1);
        let idx = |k: &str| {
            (0..d.num_records())
                .find(|&i| d.key_string(i).as_deref() == Some(k))
                .unwrap_or_else(|| panic!("key {k} missing"))
        };
        assert_eq!(
            d.value_string(idx("1e/s")).as_deref(),
            Some("com8jeNjaboLt_")
        );
        assert_eq!(d.value_string(idx("1A/M")).as_deref(), Some("aMp9am8m9te"));
        assert_eq!(d.value_string(idx("1m3")).as_deref(), Some("liBba*m9te"));
        assert_eq!(d.value_string(idx("1%")).as_deref(), Some("p_lo"));
    }

    #[test]
    fn prepron_real() {
        let d = parse_prepron(&load("PronDict/prepron.bin")).unwrap();
        assert_eq!(d.num_records(), 383);
        assert_eq!(d.num_tree_nodes(), 765);
        assert_eq!(leaf_count(&d), 383);
        let mut dist = std::collections::BTreeMap::new();
        for i in 0..d.num_records() {
            *dist.entry(d.code(i).unwrap()).or_insert(0usize) += 1;
        }
        assert_eq!(dist.get(&6), Some(&4));
        assert_eq!(dist.get(&3), Some(&161));
        assert_eq!(dist.get(&2), Some(&210));
        for k in ["http", "ftp", "gopher", "mailto"] {
            let i = (0..d.num_records())
                .find(|&i| d.key_string(i).as_deref() == Some(k))
                .unwrap_or_else(|| panic!("key {k} missing"));
            assert_eq!(d.code(i), Some(6), "code of {k}");
        }
    }

    #[test]
    fn unipron_real() {
        let d = parse_unipron(&load("PronDict/unipron.bin")).unwrap();
        assert_eq!(d.num_records(), 558);
        assert_eq!(d.num_tree_nodes(), 1115);
        assert_eq!(leaf_count(&d), 558);
        let idx = |k: &str| {
            (0..d.num_records())
                .find(|&i| d.key_string(i).as_deref() == Some(k))
                .unwrap_or_else(|| panic!("key {k} missing"))
        };
        assert_eq!(d.value_string(idx("4ン")).as_deref(), Some("_*"));
        assert_eq!(d.value_string(idx("4ヲ")).as_deref(), Some("o"));
        assert_eq!(d.value_string(idx("2+")).as_deref(), Some("dehagi"));
        assert_eq!(d.value_string(idx("3@")).as_deref(), Some("9t_"));
        assert_eq!(d.value_string(idx("5ⅰ")).as_deref(), Some("iL"));
        assert_eq!(d.records[557].next, -1);
        assert_eq!(d.records[0].next, 1);
    }

    #[test]
    fn morphmodify_real() {
        let d = parse_morphmodify(&load("PronDict/UniMorphModify.bin")).unwrap();
        assert_eq!(d.num_records(), 90);
        assert_eq!(d.num_tree_nodes(), 179);
        assert_eq!(leaf_count(&d), 90);
        let mut dist = std::collections::BTreeMap::new();
        for i in 0..d.num_records() {
            *dist.entry(d.code(i).unwrap()).or_insert(0usize) += 1;
        }
        let expect: &[(u8, usize)] = &[
            (0x54, 1),
            (0x55, 6),
            (0x59, 3),
            (0x5A, 2),
            (0x5D, 52),
            (0x5F, 2),
            (0x67, 21),
            (0x68, 3),
        ];
        for (c, n) in expect {
            assert_eq!(dist.get(c), Some(n), "code {c:#x}");
        }
        assert!(d.key_string(0).is_some());
        assert!(
            d.key_bytes(0)
                .unwrap()
                .iter()
                .all(|b| b.is_ascii() || *b == 0)
        );
    }

    #[test]
    fn unienglishpron_real() {
        let d = parse_unienglishpron(&load("EngDict/unienglishpron.bin")).unwrap();
        assert_eq!(d.num_records(), 1061);
        assert_eq!(d.num_tree_nodes(), 2121);
        assert_eq!(leaf_count(&d), 1061);
        let keys: Vec<String> = (0..d.num_records())
            .map(|i| d.key_string(i).unwrap())
            .collect();
        assert_eq!(keys[0], "zero");
        assert_eq!(keys[1], "zealous");
        assert_eq!(keys[1060], "abide");
        let pool_keys: Vec<String> = {
            let mut out = Vec::new();
            let mut pos = 0usize;
            while pos < d.key_pool.len() {
                let end = d.key_pool[pos..]
                    .iter()
                    .position(|&b| b == 0)
                    .map_or(d.key_pool.len(), |z| pos + z);
                if end > pos {
                    out.push(String::from_utf8_lossy(&d.key_pool[pos..end]).into_owned());
                }
                pos = end + 1;
            }
            out
        };
        assert_eq!(pool_keys.len(), 1061);
        assert_eq!(keys, pool_keys);
        let mut sorted = keys.clone();
        sorted.sort();
        sorted.reverse();
        let inversions = keys
            .iter()
            .zip(sorted.iter())
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(inversions, 5);
        let idx = |k: &str| {
            (0..d.num_records())
                .find(|&i| d.key_string(i).as_deref() == Some(k))
                .unwrap_or_else(|| panic!("key {k} missing"))
        };
        assert_eq!(d.value_string(idx("zero")).as_deref(), Some("jilou"));
        assert_eq!(d.value_string(idx("zealous")).as_deref(), Some("j9Llev_"));
        assert_eq!(d.value_string(idx("womb")).as_deref(), Some("uM"));
    }

    #[test]
    fn engsym_real() {
        let d = parse_engsym(&load("EngDict/engsym.bin")).unwrap();
        assert_eq!(d.num_records(), 48);
        assert_eq!(d.num_tree_nodes(), 95);
        assert_eq!(leaf_count(&d), 48);
        let idx = |k: &str| {
            (0..d.num_records())
                .find(|&i| d.key_string(i).as_deref() == Some(k))
                .unwrap_or_else(|| panic!("key {k} missing"))
        };
        assert_eq!(d.code(idx("aa")), Some(1));
        assert_eq!(d.code(idx("h")), Some(16));
        assert_eq!(d.code(idx("hh")), Some(16));
        assert_eq!(d.code(idx("z")), Some(38));
        assert_eq!(d.code(idx("zh")), Some(39));
        assert_eq!(d.code(idx("ia")), Some(100));
        assert_eq!(d.code(idx("ea")), Some(101));
        assert_eq!(d.code(idx("ua")), Some(102));
        assert_eq!(d.code(idx("sil")), Some(103));
        for i in 0..d.num_records() {
            let k = d.key_string(i).unwrap();
            assert!(k.bytes().all(|b| b.is_ascii()), "key {k}");
        }
    }

    #[test]
    fn user_real() {
        let d = parse_user(&load("user.bin")).unwrap();
        assert_eq!(d.num_records(), 21);
        assert_eq!(d.num_tree_nodes(), 41);
        assert!(d.tree_before_records);
        assert_eq!(leaf_count(&d), 12);
        let pool_keys: Vec<String> = {
            let mut out = Vec::new();
            let mut pos = 0usize;
            while pos + 1 < d.key_pool.len() {
                let mut end = pos;
                while end + 1 < d.key_pool.len()
                    && !(d.key_pool[end] == 0 && d.key_pool[end + 1] == 0)
                {
                    end += 2;
                }
                if end > pos {
                    out.push(String::from_utf16_lossy(
                        &d.key_pool[pos..end]
                            .chunks_exact(2)
                            .map(|c| u16::from_le_bytes([c[0], c[1]]))
                            .collect::<Vec<_>>(),
                    ));
                }
                pos = end + 2;
            }
            out
        };
        assert_eq!(pool_keys[0], "D.P.R.K");
        assert_eq!(pool_keys.len(), 21);
        assert!(pool_keys.iter().any(|k| k == "MS-DOS"));
        assert!(pool_keys.iter().any(|k| k == "C++"));
        assert!(pool_keys.iter().any(|k| k == "FIFA"));
        assert!(pool_keys.iter().any(|k| k == "<5027>"));
        let keys: Vec<String> = (0..d.num_records())
            .map(|i| d.key_string(i).unwrap())
            .collect();
        assert_eq!(keys.len(), 21);
        assert_eq!(keys[0], "2-4분기");
        let pool_vals: Vec<String> = {
            let mut out = Vec::new();
            let mut pos = 0usize;
            while pos + 1 < d.value_pool.len() {
                let mut end = pos;
                while end + 1 < d.value_pool.len()
                    && !(d.value_pool[end] == 0 && d.value_pool[end + 1] == 0)
                {
                    end += 2;
                }
                if end > pos {
                    out.push(String::from_utf16_lossy(
                        &d.value_pool[pos..end]
                            .chunks_exact(2)
                            .map(|c| u16::from_le_bytes([c[0], c[1]]))
                            .collect::<Vec<_>>(),
                    ));
                }
                pos = end + 2;
            }
            out
        };
        assert_eq!(pool_vals.len(), 21);
        assert!(pool_vals.iter().any(|v| v == "조선민주주의인민공화국"));
        assert!(pool_vals.iter().any(|v| v == "엠에쓰도스"));
        assert!(pool_vals.iter().any(|v| v == "씨쁠라스 쁠라스"));
        let vals = (0..d.num_records())
            .filter_map(|i| d.value_string(i))
            .count();
        assert_eq!(vals, 12);
        assert_eq!(d.value_string(0).as_deref(), Some("사분기"));
        assert_eq!(
            d.records
                .iter()
                .filter(|r| r.value == RecordValue::None)
                .count(),
            9
        );
        assert_eq!(d.records[0].key_ref, 14);
        assert_eq!(d.records[0].value, RecordValue::PoolRef(18));
        assert_eq!(d.records[0].next, 19);
        assert_eq!(d.records[20].key_ref, 0);
        assert_eq!(d.records[20].value, RecordValue::PoolRef(38));
        assert_eq!(d.records[20].next, 39);
        for r in &d.records {
            assert!(r.next == -1 || (0..41).contains(&r.next));
        }
    }

    #[test]
    fn malformed_rejected() {
        assert!(
            parse_strpron(&[121, 0, 0, 0, 241, 0, 0, 0, 0x33, 2, 0, 0, 0xa3, 4, 0, 0]).is_err()
        );
        let mut d = load("EngDict/engsym.bin");
        let n = d.len();
        d[n - 8] = 0xFF;
        d[n - 7] = 0xFF;
        d[n - 6] = 0xFF;
        d[n - 5] = 0x7F;
        assert!(parse_engsym(&d).is_err());
        let mut d2 = load("EngDict/engsym.bin");
        let rec0 = 12 + 127;
        d2[rec0] = 0xFF;
        d2[rec0 + 1] = 0xFF;
        d2[rec0 + 2] = 0xFF;
        d2[rec0 + 3] = 0xFF;
        assert!(parse_engsym(&d2).is_err());
    }
}
