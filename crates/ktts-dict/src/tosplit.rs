use crate::common::{DictError, DictResult, Reader};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToSplitEntry {
    pub name: String,
    pub pos: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToSplitDic {
    pub sections: Vec<Vec<ToSplitEntry>>,
}

impl ToSplitDic {
    #[must_use]
    pub fn total_entries(&self) -> usize {
        self.sections.iter().map(Vec::len).sum()
    }
    #[must_use]
    pub fn section(&self, i: usize) -> &[ToSplitEntry] {
        self.sections.get(i).map_or(&[], Vec::as_slice)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrrPredEntry {
    pub stem: String,
    pub cond: [u8; 3],
}

impl IrrPredEntry {
    #[must_use]
    pub fn conditions(&self) -> &[u8] {
        match self.cond.iter().position(|&b| b == 0xFF) {
            Some(i) => &self.cond[..i],
            None => &self.cond,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrregulePredDic {
    pub sections: Vec<Vec<IrrPredEntry>>,
}

impl IrregulePredDic {
    #[must_use]
    pub fn total_entries(&self) -> usize {
        self.sections.iter().map(Vec::len).sum()
    }
    #[must_use]
    pub fn section(&self, i: usize) -> &[IrrPredEntry] {
        self.sections.get(i).map_or(&[], Vec::as_slice)
    }
}

pub const TO_SPLIT_SECTIONS: usize = 12;
pub const IRR_PRED_SECTIONS: usize = 5;

const TO_SPLIT_NAME_CAP: usize = 15;
const IRR_NAME_CAP: usize = 5;

fn check_name(name: &[u8], what: &str, off: usize) -> DictResult<()> {
    if name.is_empty() {
        return Err(DictError::new(format!("{what}: name is empty"), off));
    }
    if let Some(&b) = name.iter().find(|&&b| !b.is_ascii_graphic()) {
        return Err(DictError::new(
            format!("{what}: non-ASCII graphic char 0x{b:02x}"),
            off,
        ));
    }
    Ok(())
}

/// Parses the `ToSplit` dictionary.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_to_split(data: &[u8]) -> DictResult<ToSplitDic> {
    let mut r = Reader::new(data);
    let mut sections: Vec<Vec<ToSplitEntry>> = Vec::with_capacity(TO_SPLIT_SECTIONS);
    for si in 0..TO_SPLIT_SECTIONS {
        let n = r.u32().map_err(|e| {
            DictError::new(
                format!("ToSplit section {si} count read failed: {e}"),
                e.offset,
            )
        })?;
        let expected_len = si + 1;
        let mut entries: Vec<ToSplitEntry> = Vec::with_capacity((n as usize).min(1 << 16));
        for ei in 0..n {
            let slot_off = r.pos;
            let slot = r.bytes(TO_SPLIT_NAME_CAP + 1)?;
            let name_len = slot[..TO_SPLIT_NAME_CAP]
                .iter()
                .position(|&b| b == 0)
                .ok_or_else(|| {
                    DictError::new(
                        format!("ToSplit section {si} entry {ei}: schName has no NUL"),
                        slot_off,
                    )
                })?;
            check_name(
                &slot[..name_len],
                &format!("ToSplit section {si} entry {ei}"),
                slot_off,
            )?;
            if name_len != expected_len {
                return Err(DictError::new(
                    format!(
                        "ToSplit section {si} entry {ei}: name length {name_len} (expected {expected_len})"
                    ),
                    slot_off,
                ));
            }
            if let Some(&b) = slot[name_len + 1..TO_SPLIT_NAME_CAP]
                .iter()
                .find(|&&b| b != 0xCD)
            {
                return Err(DictError::new(
                    format!(
                        "ToSplit section {si} entry {ei}: padding byte 0x{b:02x} (expected 0xCD)"
                    ),
                    slot_off,
                ));
            }
            entries.push(ToSplitEntry {
                name: String::from_utf8_lossy(&slot[..name_len]).into_owned(),
                pos: slot[TO_SPLIT_NAME_CAP],
            });
        }
        sections.push(entries);
    }
    if r.remaining() != 0 {
        return Err(DictError::new(
            format!("ToSplit: {} bytes trailing after parse", r.remaining()),
            r.pos,
        ));
    }
    Ok(ToSplitDic { sections })
}

/// Parses the irregular-predicate dictionary.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_irregule_pred(data: &[u8]) -> DictResult<IrregulePredDic> {
    let mut r = Reader::new(data);
    let mut sections: Vec<Vec<IrrPredEntry>> = Vec::with_capacity(IRR_PRED_SECTIONS);
    for si in 0..IRR_PRED_SECTIONS {
        let n = r.u32().map_err(|e| {
            DictError::new(
                format!("IrregulePred section {si} count read failed: {e}"),
                e.offset,
            )
        })?;
        let expected_len = si + 1;
        let mut entries: Vec<IrrPredEntry> = Vec::with_capacity((n as usize).min(1 << 16));
        for ei in 0..n {
            let rec_off = r.pos;
            let rec = r.bytes(8)?;
            let raw = &rec[..IRR_NAME_CAP];
            let name_len = raw.iter().position(|&b| b == 0).unwrap_or(IRR_NAME_CAP);
            check_name(
                &raw[..name_len],
                &format!("IrregulePred section {si} entry {ei}"),
                rec_off,
            )?;
            if name_len != expected_len {
                return Err(DictError::new(
                    format!(
                        "IrregulePred section {si} entry {ei}: stem length {name_len} (expected {expected_len})"
                    ),
                    rec_off,
                ));
            }
            entries.push(IrrPredEntry {
                stem: String::from_utf8_lossy(&raw[..name_len]).into_owned(),
                cond: [rec[5], rec[6], rec[7]],
            });
        }
        sections.push(entries);
    }
    if r.remaining() != 0 {
        return Err(DictError::new(
            format!("IrregulePred: {} bytes trailing after parse", r.remaining()),
            r.pos,
        ));
    }
    Ok(IrregulePredDic { sections })
}

/// Parses both `ToSplit` dictionaries.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse(tosplit: &[u8], irregule_pred: &[u8]) -> DictResult<(ToSplitDic, IrregulePredDic)> {
    Ok((
        parse_to_split(tosplit)?,
        parse_irregule_pred(irregule_pred)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(rel: &str) -> Vec<u8> {
        let dir = crate::pronsec::test_data_dir().join("KMPADict");
        std::fs::read(dir.join(rel)).unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
    }

    const TOSPLIT_COUNTS: [usize; 12] = [6, 35, 47, 123, 109, 71, 93, 38, 25, 17, 6, 2];
    const IRR_COUNTS: [usize; 5] = [4, 113, 187, 29, 10];

    #[test]
    fn tosplit_real_file() {
        let data = load("ToSplit.bin");
        let d = parse_to_split(&data).unwrap();
        assert_eq!(d.sections.len(), 12);
        let counts: Vec<usize> = d.sections.iter().map(Vec::len).collect();
        assert_eq!(counts, TOSPLIT_COUNTS);
        assert_eq!(d.total_entries(), 572);
        let names: Vec<&str> = d.section(0).iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["8", "9", "e", "i", "o", "u"]);
        assert_eq!(d.section(0)[0].pos, b'^');
        assert_eq!(d.section(0)[1].pos, b'W');
        assert_eq!(d.section(0)[3].pos, b'T');
        let names2: Vec<&str> = d
            .section(1)
            .iter()
            .take(4)
            .map(|e| e.name.as_str())
            .collect();
        assert_eq!(names2, ["_L", "_M", "_N", "9N"]);
        assert_eq!(d.section(2)[0].name, "_l_");
        assert_eq!(d.section(3)[0].name, "_Lga");
        assert_eq!(d.section(11)[0].name, "ilaNd9seqaji");
        assert_eq!(d.section(11)[0].pos, b'X');
        for (i, sec) in d.sections.iter().enumerate() {
            for e in sec {
                assert_eq!(e.name.len(), i + 1, "section {i} name length");
                assert!((0x54..=0x5F).contains(&e.pos), "chPOS 0x{:02x}", e.pos);
            }
        }
    }

    #[test]
    fn irregule_pred_real_file() {
        let data = load("IrregulePred.bin");
        let d = parse_irregule_pred(&data).unwrap();
        assert_eq!(d.sections.len(), 5);
        let counts: Vec<usize> = d.sections.iter().map(Vec::len).collect();
        assert_eq!(counts, IRR_COUNTS);
        assert_eq!(d.total_entries(), 343);
        let stems: Vec<&str> = d.section(0).iter().map(|e| e.stem.as_str()).collect();
        assert_eq!(stems, ["_", "a", "i", "u"]);
        for e in d.section(0) {
            assert_eq!(e.cond, [0x01, 0xFF, 0xFF]);
            assert_eq!(e.conditions(), &[0x01]);
        }
        let s5: Vec<&str> = d.section(4).iter().map(|e| e.stem.as_str()).collect();
        assert_eq!(
            s5,
            [
                "bogel", "gagel", "gyeL_", "gyeLe", "iVgel", "il_le", "jagel", "nagel", "onela",
                "ul_le"
            ]
        );
        assert_eq!(d.section(4)[0].cond, [0x09, 0xFF, 0xFF]);
        let ga = d.section(1).iter().find(|e| e.stem == "ga").unwrap();
        assert_eq!(ga.cond, [0x00, 0x01, 0x11]);
        assert_eq!(ga.conditions(), &[0x00, 0x01, 0x11]);
        let d9 = d.section(1).iter().find(|e| e.stem == "d9").unwrap();
        assert_eq!(d9.cond, [0x0D, 0x13, 0xFF]);
        assert_eq!(d9.conditions(), &[0x0D, 0x13]);
        for (i, sec) in d.sections.iter().enumerate() {
            for e in sec {
                assert_eq!(e.stem.len(), i + 1, "section {i} stem length");
                for &b in &e.cond {
                    assert!(b <= 0x13 || b == 0xFF, "condition code 0x{b:02x}");
                }
                assert!(
                    !e.conditions().is_empty(),
                    "section {i} {} has no conditions",
                    e.stem
                );
            }
        }
    }

    #[test]
    fn combined_parse() {
        let (ts, irr) = parse(&load("ToSplit.bin"), &load("IrregulePred.bin")).unwrap();
        assert_eq!(ts.total_entries(), 572);
        assert_eq!(irr.total_entries(), 343);
    }

    #[test]
    fn tosplit_synthetic() {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&2u32.to_le_bytes());
        buf.extend_from_slice(b"a\0");
        buf.extend(std::iter::repeat_n(0xCD, 13));
        buf.push(b'X');
        buf.extend_from_slice(b"b\0");
        buf.extend(std::iter::repeat_n(0xCD, 13));
        buf.push(b'^');
        buf.extend_from_slice(&1u32.to_le_bytes());
        buf.extend_from_slice(b"cd\0");
        buf.extend(std::iter::repeat_n(0xCD, 12));
        buf.push(b'Z');
        for _ in 2..12 {
            buf.extend_from_slice(&0u32.to_le_bytes());
        }
        let d = parse_to_split(&buf).unwrap();
        assert_eq!(d.sections.len(), 12);
        assert_eq!(d.section(2).len(), 0);
        assert_eq!(d.total_entries(), 3);
        assert_eq!(
            d.section(0)[0],
            ToSplitEntry {
                name: "a".into(),
                pos: b'X'
            }
        );
        assert_eq!(
            d.section(0)[1],
            ToSplitEntry {
                name: "b".into(),
                pos: b'^'
            }
        );
        assert_eq!(
            d.section(1)[0],
            ToSplitEntry {
                name: "cd".into(),
                pos: b'Z'
            }
        );
    }

    #[test]
    fn tosplit_truncated() {
        let data = load("ToSplit.bin");
        assert!(parse_to_split(&data[..data.len() - 1]).is_err());
        assert!(parse_to_split(&data[..4]).is_err());
        assert!(parse_to_split(&[]).is_err());
    }

    #[test]
    fn tosplit_bad_name_len() {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&1u32.to_le_bytes());
        buf.extend_from_slice(b"xy\0");
        buf.extend(std::iter::repeat_n(0xCD, 12));
        buf.push(b'^');
        assert!(parse_to_split(&buf).is_err());
    }

    #[test]
    fn tosplit_missing_nul() {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&1u32.to_le_bytes());
        buf.extend(std::iter::repeat_n(b'z', 16));
        assert!(parse_to_split(&buf).is_err());
    }

    #[test]
    fn irregule_pred_truncated() {
        let data = load("IrregulePred.bin");
        assert!(parse_irregule_pred(&data[..data.len() - 1]).is_err());
        assert!(parse_irregule_pred(&[]).is_err());
    }
}
