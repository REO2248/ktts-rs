use crate::common::{DictError, DictResult, Reader};
use std::collections::HashMap;

pub const KMORPH_HASH_BUCKETS: usize = 16127;

/// Computes the bucket index for a key.
///
/// # Panics
///
/// Panics if `KMORPH_HASH_BUCKETS` does not fit in `u32`.
#[must_use]
pub fn get_hash_value(key: &[u8]) -> usize {
    let mut h: u32 = 0;
    for &c in key {
        let c = u32::from_ne_bytes(i32::from(i8::from_ne_bytes([c])).to_ne_bytes());
        h = h.wrapping_mul(31).wrapping_add(c);
    }
    (h % u32::try_from(KMORPH_HASH_BUCKETS).expect("KMORPH_HASH_BUCKETS fits u32")) as usize
}

#[derive(Debug)]
pub struct KmorphHashRecord<'a> {
    pub offset: usize,
    pub keys: Vec<&'a [u8]>,
    pub dic_offsets: Vec<u32>,
}

#[derive(Debug)]
pub struct KmorphHashDic<'a> {
    pub buckets: Vec<u32>,
    pub records: Vec<KmorphHashRecord<'a>>,
    offset_to_record: HashMap<u32, usize>,
}

impl KmorphHashDic<'_> {
    #[must_use]
    pub fn lookup(&self, key: &[u8]) -> Option<u32> {
        let h = get_hash_value(key);
        let v = *self.buckets.get(h)?;
        if v == 0 {
            return None;
        }
        let rec = &self.records[*self.offset_to_record.get(&(v - 1))?];
        for (i, k) in rec.keys.iter().enumerate() {
            if *k == key {
                return Some(rec.dic_offsets[i]);
            }
        }
        None
    }
}

/// Parses the kmorph hash bin.
///
/// # Errors
///
/// Returns an error if the size does not match `KMORPH_HASH_BUCKETS * 4`.
pub fn parse_hash_bin(data: &[u8]) -> DictResult<Vec<u32>> {
    if data.len() != KMORPH_HASH_BUCKETS * 4 {
        return Err(DictError::new(
            format!(
                "kmorph_hash.bin size mismatch: {} (expected {})",
                data.len(),
                KMORPH_HASH_BUCKETS * 4
            ),
            0,
        ));
    }
    let mut r = Reader::new(data);
    let mut buckets = Vec::with_capacity(KMORPH_HASH_BUCKETS);
    for _ in 0..KMORPH_HASH_BUCKETS {
        buckets.push(r.u32()?);
    }
    Ok(buckets)
}

/// Parses the kmorph hash dictionary.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
///
/// # Panics
///
/// Panics if a record offset does not fit in `u32`.
pub fn parse_hash_dic(data: &[u8]) -> DictResult<KmorphHashDic<'_>> {
    let mut r = Reader::new(data);
    let mut records = Vec::new();
    let mut offset_to_record = HashMap::new();
    while r.remaining() > 0 {
        let rec_start = r.pos;
        let n_addr = r.u8()? as usize;
        let str_len = r.u8()? as usize;
        let addrs = r.bytes(n_addr)?.to_vec();
        let buf = r.bytes(str_len)?;
        let mut keys = Vec::with_capacity(n_addr);
        for &a in &addrs {
            let a = a as usize;
            if a >= buf.len() {
                return Err(DictError::new(
                    format!("addr[{a}] exceeds buf length {}", buf.len()),
                    rec_start,
                ));
            }
            let end = buf[a..]
                .iter()
                .position(|&b| b == 0)
                .map_or(buf.len(), |p| a + p);
            keys.push(&buf[a..end]);
        }
        let mut dic_offsets = Vec::with_capacity(n_addr);
        for _ in 0..n_addr {
            dic_offsets.push(r.u32()?);
        }
        offset_to_record.insert(
            u32::try_from(rec_start).expect("record offset fits u32"),
            records.len(),
        );
        records.push(KmorphHashRecord {
            offset: rec_start,
            keys,
            dic_offsets,
        });
    }
    if r.pos != data.len() {
        return Err(DictError::new(
            format!("trailing excess {} bytes", data.len() - r.pos),
            r.pos,
        ));
    }
    Ok(KmorphHashDic {
        buckets: Vec::new(),
        records,
        offset_to_record,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SKeyItem {
    pub ch_pos: u8,
    pub n_freq: u32,
    pub ch_irr_pred: u8,
    pub un_ui_link: u32,
}

#[derive(Debug)]
pub struct KmorphDicRecord<'a> {
    pub offset: usize,
    pub text: &'a [u8],
    pub items: Vec<SKeyItem>,
}

#[derive(Debug)]
pub struct KmorphDic<'a> {
    pub records: Vec<KmorphDicRecord<'a>>,
}

impl<'a> KmorphDic<'a> {
    /// Looks up a record by offset.
    ///
    /// # Panics
    ///
    /// Panics if a record offset does not fit in `u32`.
    #[must_use]
    pub fn record_at(&self, offset: u32) -> Option<&KmorphDicRecord<'a>> {
        self.records
            .binary_search_by_key(&offset, |r| {
                u32::try_from(r.offset).expect("record offset fits u32")
            })
            .ok()
            .map(|i| &self.records[i])
    }
}

/// Parses the kmorph dictionary records for the given offsets.
///
/// # Errors
///
/// Returns an error if an offset exceeds the file size or the data is truncated.
pub fn parse_dic<'a>(data: &'a [u8], offsets: &[u32]) -> DictResult<KmorphDic<'a>> {
    let mut offs: Vec<u32> = offsets.to_vec();
    offs.sort_unstable();
    offs.dedup();
    let mut records = Vec::with_capacity(offs.len());
    let mut last_end = 0usize;
    for &o in &offs {
        let o = o as usize;
        if o >= data.len() {
            return Err(DictError::new("dic offset exceeds file size", o));
        }
        let mut r = Reader::new(&data[o..]);
        let len = r.u8()? as usize;
        let text = r.bytes(len)?;
        let n_item = r.u8()? as usize;
        let mut items = Vec::with_capacity(n_item);
        for _ in 0..n_item {
            let b = r.bytes(10)?;
            items.push(SKeyItem {
                ch_pos: b[0],
                n_freq: u32::from_le_bytes([b[1], b[2], b[3], b[4]]),
                ch_irr_pred: b[5],
                un_ui_link: u32::from_le_bytes([b[6], b[7], b[8], b[9]]),
            });
        }
        last_end = o + r.pos;
        records.push(KmorphDicRecord {
            offset: o,
            text,
            items,
        });
    }
    if last_end != data.len() {
        return Err(DictError::new(
            format!(
                "kmorph.dic full consumption failed: last record end {} != file size {}",
                last_end,
                data.len()
            ),
            last_end,
        ));
    }
    Ok(KmorphDic { records })
}

#[derive(Debug)]
pub struct KmorphDict<'a> {
    pub hash: KmorphHashDic<'a>,
    pub dic: KmorphDic<'a>,
}

/// Parses the combined kmorph dictionary.
///
/// # Errors
///
/// Returns an error if any part is malformed or a bucket does not match a record.
pub fn parse<'a>(
    hash_bin: &'a [u8],
    hash_dic: &'a [u8],
    dic: &'a [u8],
) -> DictResult<KmorphDict<'a>> {
    let buckets = parse_hash_bin(hash_bin)?;
    let mut hash = parse_hash_dic(hash_dic)?;
    hash.buckets = buckets;
    for (h, &v) in hash.buckets.iter().enumerate() {
        if v != 0 && !hash.offset_to_record.contains_key(&(v - 1)) {
            return Err(DictError::new(
                format!("bucket {h} value {v} does not match record start+1"),
                h * 4,
            ));
        }
    }
    for rec in &hash.records {
        let h = get_hash_value(rec.keys[0]);
        if hash.buckets[h] as usize != rec.offset + 1 {
            return Err(DictError::new(
                format!(
                    "key {} hash {h} does not match record @{:#x} (bucket={})",
                    String::from_utf8_lossy(rec.keys[0]),
                    rec.offset,
                    hash.buckets[h]
                ),
                rec.offset,
            ));
        }
    }
    let offsets: Vec<u32> = hash
        .records
        .iter()
        .flat_map(|r| r.dic_offsets.iter().copied())
        .collect();
    let dic = parse_dic(dic, &offsets)?;
    Ok(KmorphDict { hash, dic })
}

impl<'a> KmorphDict<'a> {
    #[must_use]
    pub fn lookup(&self, key: &[u8]) -> Option<&KmorphDicRecord<'a>> {
        let o = self.hash.lookup(key)?;
        self.dic.record_at(o)
    }
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

    fn full() -> KmorphDict<'static> {
        let leak = |v: Vec<u8>| -> &'static [u8] { Box::leak(v.into_boxed_slice()) };
        parse(
            leak(load("kmorph_hash.bin")),
            leak(load("kmorph_hash.dic")),
            leak(load("kmorph.dic")),
        )
        .expect("failed to parse kmorph 3-file set")
    }

    #[test]
    fn hash_bin_bucket_array() {
        let buckets = parse_hash_bin(&load("kmorph_hash.bin")).unwrap();
        assert_eq!(buckets.len(), 16127);
        assert_eq!(&buckets[..4], &[1, 39, 162, 246]);
        assert_eq!(buckets[16_126], 0x18_3A86);
        let empty: Vec<usize> = (0..16127).filter(|&i| buckets[i] == 0).collect();
        assert_eq!(
            empty,
            vec![
                2406, 2945, 4172, 8161, 9826, 10051, 10476, 11296, 12932, 13687, 14015
            ]
        );
        assert_eq!(16127 - empty.len(), 16116);
    }

    #[test]
    fn hash_dic_records_full_consumption() {
        let data = load("kmorph_hash.dic");
        let hash = parse_hash_dic(&data).unwrap();
        assert_eq!(hash.records.len(), 16116);
        let r0 = &hash.records[0];
        assert_eq!(r0.offset, 0);
        assert_eq!(
            r0.keys,
            vec![&b"g8yodo"[..], &b"daLnai"[..], &b"baNjeL"[..]]
        );
        assert_eq!(r0.dic_offsets, vec![0xA937, 0x44F2C, 0x86A1B]);
        let n_keys: usize = hash.records.iter().map(|r| r.keys.len()).sum();
        assert_eq!(n_keys, 116_655);
        let rec = hash
            .records
            .iter()
            .find(|r| r.keys.iter().any(|k| *k == b"hi*hi*"))
            .expect("no record containing hi*hi* found");
        assert_eq!(rec.offset, 0x15_A9DC);
        let hih = rec.keys.iter().position(|k| *k == b"hi*hi*").unwrap();
        assert_eq!(rec.dic_offsets[hih], 0x17_7B6F);
        let last = hash.records.last().unwrap();
        assert_eq!(last.offset, 0x18_3A85);
    }

    #[test]
    fn hash_value_formula() {
        assert_eq!(get_hash_value(b"baNjeL"), 0);
        assert_eq!(get_hash_value(b"useNju"), 1);
        assert_eq!(get_hash_value(b"g8yodo"), 0);
        assert_eq!(get_hash_value(b"daLnai"), 0);
        assert_eq!(get_hash_value(b"jagisu"), 1);
        assert_eq!(get_hash_value(b"d_LmeGd_LmeG"), 1);
        assert_eq!(get_hash_value(b"hi*hi*"), 14432);
        assert_eq!(get_hash_value(b""), 0);
    }

    #[test]
    fn hash_zero_miss_all_keys() {
        let hdata = load("kmorph_hash.dic");
        let hash = parse_hash_dic(&hdata).unwrap();
        let buckets = parse_hash_bin(&load("kmorph_hash.bin")).unwrap();
        let mut checked = 0;
        for rec in &hash.records {
            for k in &rec.keys {
                let h = get_hash_value(k);
                assert_ne!(
                    buckets[h],
                    0,
                    "key {} in empty bucket",
                    String::from_utf8_lossy(k)
                );
                assert_eq!(
                    buckets[h] as usize - 1,
                    rec.offset,
                    "key {} hash {h} does not match record @{:#x}",
                    String::from_utf8_lossy(k),
                    rec.offset
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 116_655);
    }

    #[test]
    fn lookup_known_keys() {
        let d = full();
        let r = d.lookup(b"baNjeL").expect("baNjeL not found");
        assert_eq!(r.offset, 0x86A1B);
        assert_eq!(r.text, b"");
        assert_eq!(r.items.len(), 2);
        assert_eq!(
            r.items[0],
            SKeyItem {
                ch_pos: b'0',
                n_freq: 14,
                ch_irr_pred: 0,
                un_ui_link: 0
            }
        );
        assert_eq!(
            r.items[1],
            SKeyItem {
                ch_pos: b'1',
                n_freq: 1,
                ch_irr_pred: 0,
                un_ui_link: 0
            }
        );

        let r = d.lookup(b"useNju").expect("useNju not found");
        assert_eq!(r.offset, 0xF66F1);
        assert_eq!(
            r.items[0],
            SKeyItem {
                ch_pos: b'0',
                n_freq: 1,
                ch_irr_pred: 0,
                un_ui_link: 0
            }
        );

        let r = d.lookup(b"g8yodo").expect("g8yodo not found");
        assert_eq!(r.offset, 0xA937);
        let r = d.lookup(b"daLnai").expect("daLnai not found");
        assert_eq!(r.offset, 0x44F2C);

        assert!(d.lookup(b"zzzz").is_none());
        assert!(d.lookup(b"").is_none());
    }

    #[test]
    fn dic_records_full_consumption() {
        let d = full();
        assert_eq!(d.dic.records.len(), 116_655);
        let n_items: usize = d.dic.records.iter().map(|r| r.items.len()).sum();
        let n_text: usize = d.dic.records.iter().filter(|r| !r.text.is_empty()).count();
        assert_eq!(n_items, 123_263);
        assert_eq!(n_text, 8599);
        for w in d.dic.records.windows(2) {
            let (r1, r2) = (&w[0], &w[1]);
            let end1 = r1.offset + 2 + r1.text.len() + 10 * r1.items.len();
            assert_eq!(
                end1, r2.offset,
                "gap between @{:#x} and @{:#x}",
                r1.offset, r2.offset
            );
        }
        let last = d.dic.records.last().unwrap();
        let end = last.offset + 2 + last.text.len() + 10 * last.items.len();
        assert_eq!(end, 1_538_939);
        let r = d.lookup(b"hi*hi*").unwrap();
        assert_eq!(r.offset, 0x17_7B6F);
        assert_eq!(r.items.len(), 1);
        assert_eq!(r.items[0].ch_pos, b'K');
        assert_eq!(r.items[0].n_freq, 1);
        for rec in &d.hash.records {
            for (i, k) in rec.keys.iter().enumerate() {
                let r = d
                    .lookup(k)
                    .unwrap_or_else(|| panic!("{} not found", String::from_utf8_lossy(k)));
                assert_eq!(
                    u32::try_from(r.offset).expect("offset fits u32"),
                    rec.dic_offsets[i]
                );
            }
        }
    }
}
