use crate::common::{DictError, DictResult, Reader};

pub const SPHONEDICT_SIZE: usize = 120;
pub const PHONE_NUM: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhoneDict {
    pub ch_dict_file_id: i8,
    pub ch_phone_mode: i8,
    pub w_pcm_size: u16,
    pub n_pcm_start: u32,
    pub phones: [u8; PHONE_NUM],
    pub energy_pitch: [u8; PHONE_NUM],
    pub w_upm_size: u16,
    pub n_upm_start: u32,
    pub sr_start_cepstrum: [f32; 12],
    pub sr_end_cepstrum: [f32; 12],
}

impl PhoneDict {
    #[must_use]
    pub const fn pcm_range(&self) -> std::ops::Range<usize> {
        self.n_pcm_start as usize..(self.n_pcm_start as usize + self.w_pcm_size as usize)
    }
    #[must_use]
    pub const fn upm_range(&self) -> std::ops::Range<usize> {
        self.n_upm_start as usize..(self.n_upm_start as usize + self.w_upm_size as usize)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SynthUnit {
    pub unit_no: u16,
    pub records: Vec<PhoneDict>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SynthIdx {
    pub units: Vec<SynthUnit>,
    pub record_count: usize,
}

impl SynthIdx {
    pub fn records(&self) -> impl Iterator<Item = &PhoneDict> {
        self.units.iter().flat_map(|u| u.records.iter())
    }
}

/// Parses the synthesis index.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_idx(data: &[u8]) -> DictResult<SynthIdx> {
    let mut r = Reader::new(data);
    let n = r.u16()? as usize;
    let mut units = Vec::with_capacity(n);
    let mut record_count = 0usize;
    for i in 0..n {
        let no = r.u16()?;
        if no as usize != i {
            return Err(DictError::new(
                format!("unitNo sequence violation: expected {i}, got {no}"),
                r.pos - 2,
            ));
        }
        let cnt = r.u16()? as usize;
        let mut records = Vec::with_capacity(cnt);
        for _ in 0..cnt {
            records.push(parse_phonedict(&mut r)?);
        }
        record_count += cnt;
        units.push(SynthUnit {
            unit_no: no,
            records,
        });
    }
    if r.remaining() != 0 {
        return Err(DictError::new(
            format!("synth.idx full consumption failed: {}B left", r.remaining()),
            r.pos,
        ));
    }
    Ok(SynthIdx {
        units,
        record_count,
    })
}

fn parse_phonedict(r: &mut Reader<'_>) -> DictResult<PhoneDict> {
    let ch_dict_file_id = i8::from_ne_bytes([r.u8()?]);
    let ch_phone_mode = i8::from_ne_bytes([r.u8()?]);
    let w_pcm_size = r.u16()?;
    let n_pcm_start = r.u32()?;
    let mut phones = [0u8; PHONE_NUM];
    for v in &mut phones {
        *v = r.u8()?;
    }
    let mut energy_pitch = [0u8; PHONE_NUM];
    for v in &mut energy_pitch {
        *v = r.u8()?;
    }
    let w_upm_size = r.u16()?;
    let n_upm_start = r.u32()?;
    let mut sr_start_cepstrum = [0f32; 12];
    for v in &mut sr_start_cepstrum {
        *v = r.f32()?;
    }
    let mut sr_end_cepstrum = [0f32; 12];
    for v in &mut sr_end_cepstrum {
        *v = r.f32()?;
    }
    Ok(PhoneDict {
        ch_dict_file_id,
        ch_phone_mode,
        w_pcm_size,
        n_pcm_start,
        phones,
        energy_pitch,
        w_upm_size,
        n_upm_start,
        sr_start_cepstrum,
        sr_end_cepstrum,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcmFile<'a> {
    pub data: &'a [u8],
}

impl<'a> PcmFile<'a> {
    #[must_use]
    pub const fn byte_len(&self) -> usize {
        self.data.len()
    }
    /// Returns the PCM segment for a phone record.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is malformed or truncated.
    pub fn segment(&self, rec: &PhoneDict) -> DictResult<&'a [u8]> {
        let rng = rec.pcm_range();
        self.data.get(rng.clone()).ok_or_else(|| {
            DictError::new(
                format!("PCM out of range: {:?} (file {}B)", rng, self.data.len()),
                rng.start,
            )
        })
    }
}

/// Parses the PCM data file.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub const fn parse_pcm(data: &[u8]) -> DictResult<PcmFile<'_>> {
    Ok(PcmFile { data })
}

#[must_use]
pub fn uraw_to_pcm(n: i8) -> i16 {
    let inv = !i32::from(n);
    let mantissa = (inv & 0xf) * 8 + 0x84;
    let shift = (u32::from_ne_bytes(inv.to_ne_bytes()) >> 4) & 7;
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let s = i16::from_ne_bytes(((mantissa << shift) as u16).to_ne_bytes());
    if n < 0 {
        s.wrapping_sub(0x84)
    } else {
        0x84i16.wrapping_sub(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpmMark {
    pub addr: u16,
    pub cur_pitch: u8,
    pub next_pitch: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpmFile<'a> {
    pub data: &'a [u8],
}

impl<'a> UpmFile<'a> {
    #[must_use]
    pub const fn byte_len(&self) -> usize {
        self.data.len()
    }
    /// Returns the UPM segment for a phone record.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is malformed or truncated.
    pub fn segment(&self, rec: &PhoneDict) -> DictResult<&'a [u8]> {
        let rng = rec.upm_range();
        self.data.get(rng.clone()).ok_or_else(|| {
            DictError::new(
                format!("UPM out of range: {:?} (file {}B)", rng, self.data.len()),
                rng.start,
            )
        })
    }
    /// Computes cumulative mark positions within a record.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is malformed or truncated.
    pub fn mark_positions(&self, rec: &PhoneDict) -> DictResult<Vec<u32>> {
        let seg = self.segment(rec)?;
        let mut pos = 0u32;
        let mut out = Vec::with_capacity(seg.len());
        for &d in seg {
            pos += u32::from(d);
            out.push(pos);
        }
        Ok(out)
    }
    /// Decodes the marks of a record with the given mode.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is malformed or truncated.
    pub fn decode_marks(&self, rec: &PhoneDict, mode: u8) -> DictResult<Vec<UpmMark>> {
        let seg = self.segment(rec)?;
        Ok(decode_marks(seg, mode))
    }
}

/// Parses the UPM data file.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub const fn parse_upm(data: &[u8]) -> DictResult<UpmFile<'_>> {
    Ok(UpmFile { data })
}

#[must_use]
pub fn decode_marks(seg: &[u8], mode: u8) -> Vec<UpmMark> {
    let mut out = Vec::new();
    if seg.len() < 2 {
        return out;
    }
    let (w_pitch, start) = if mode == 2 {
        (seg[0], 0usize)
    } else {
        if seg.len() < 3 {
            return out;
        }
        (seg[1], 1usize)
    };
    let mut addr: u16 = 0;
    let mut i = 0usize;
    loop {
        let cur = if i == 0 {
            w_pitch
        } else {
            out[i - 1].next_pitch
        };
        let d = seg[start + i];
        let next = seg.get(start + i + 1).copied().unwrap_or(0);
        out.push(UpmMark {
            addr,
            cur_pitch: cur,
            next_pitch: next,
        });
        i += 1;
        if seg.len() - 1 <= start + i {
            break;
        }
        addr = addr.wrapping_add(u16::from(d));
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthGroup {
    pub group_no: u16,
    pub phones: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthGroupIdx {
    pub groups: Vec<SynthGroup>,
}

/// Parses the synthesis group index.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_group_idx(data: &[u8]) -> DictResult<SynthGroupIdx> {
    let mut r = Reader::new(data);
    let n = r.u16()? as usize;
    let mut groups = Vec::with_capacity(n);
    for i in 0..n {
        let no = r.u16()?;
        if no as usize != i {
            return Err(DictError::new(
                format!("groupNo sequence violation: expected {i}, got {no}"),
                r.pos - 2,
            ));
        }
        let cnt = r.u16()? as usize;
        let mut phones = Vec::with_capacity(cnt);
        for _ in 0..cnt {
            phones.push(r.u16()?);
        }
        groups.push(SynthGroup {
            group_no: no,
            phones,
        });
    }
    if r.remaining() != 0 {
        return Err(DictError::new(
            format!(
                "synth_group.idx full consumption failed: {}B left",
                r.remaining()
            ),
            r.pos,
        ));
    }
    Ok(SynthGroupIdx { groups })
}

#[derive(Debug, Clone, PartialEq)]
pub struct TriangularTable {
    pub n: u16,
    pub rows: Vec<Vec<f32>>,
}

impl TriangularTable {
    #[must_use]
    pub fn total_floats(&self) -> usize {
        self.rows.iter().map(Vec::len).sum()
    }
    #[must_use]
    pub fn at(&self, row: usize, col: usize) -> Option<f32> {
        self.rows.get(row)?.get(col).copied()
    }
}

/// Parses the triangular table.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
/// # Panics
///
/// Panics if the table size does not fit in `u16`.
pub fn parse_triangular_table(data: &[u8]) -> DictResult<TriangularTable> {
    let mut r = Reader::new(data);
    let n = r.u16()? as usize;
    let mut rows = Vec::with_capacity(n);
    for _ in 0..n {
        let mut row = Vec::with_capacity(rows.len() + 1);
        for _ in 0..=rows.len() {
            row.push(r.f32()?);
        }
        rows.push(row);
    }
    if r.remaining() != 0 {
        return Err(DictError::new(
            format!(
                "triangular table full consumption failed: {}B left",
                r.remaining()
            ),
            r.pos,
        ));
    }
    Ok(TriangularTable {
        n: u16::try_from(n).expect("table size fits u16"),
        rows,
    })
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::float_cmp,
        reason = "oracle assertions use exact float equality"
    )]
    use super::*;

    fn data_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(
            std::env::var("KTTSDB_DIR")
                .expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
        )
    }

    fn read(p: impl AsRef<std::path::Path>) -> Vec<u8> {
        std::fs::read(p).unwrap_or_else(|e| panic!("read failed: {e}"))
    }

    fn idx_path() -> String {
        format!("{}/KSpeechDic/woman/synth.idx", data_dir().display())
    }

    #[test]
    fn synth_idx_full_parse() {
        let data = read(idx_path());
        let idx = parse_idx(&data).expect("synth.idx parse");
        assert_eq!(idx.units.len(), 38_624);
        assert_eq!(idx.record_count, 269_127);
        let r0 = &idx.units[0].records[0];
        assert_eq!(r0.ch_dict_file_id, 0);
        assert_eq!(r0.ch_phone_mode, 0);
        assert_eq!(r0.w_pcm_size, 548);
        assert_eq!(r0.n_pcm_start, 0);
        assert_eq!(r0.phones, [0x0e, 0x23, 0x02, 0x23, 0x57]);
        assert_eq!(r0.energy_pitch, [0x8d, 0xac, 0x49, 0x40, 0x4f]);
        assert_eq!(r0.w_upm_size, 5);
        assert_eq!(r0.n_upm_start, 0);
    }

    #[test]
    fn synth_idx_pcm_upm_tiling() {
        let idx = parse_idx(&read(idx_path())).unwrap();
        let pcm_len = std::fs::metadata(format!(
            "{}/KSpeechDic/woman/synth.pcm",
            data_dir().display()
        ))
        .unwrap()
        .len()
        .try_into()
        .expect("file size fits usize");
        let upm_len = std::fs::metadata(format!(
            "{}/KSpeechDic/woman/synth.upm",
            data_dir().display()
        ))
        .unwrap()
        .len()
        .try_into()
        .expect("file size fits usize");
        let mut pr: Vec<(u32, u16)> = idx
            .records()
            .map(|r| (r.n_pcm_start, r.w_pcm_size))
            .collect();
        let mut ur: Vec<(u32, u16)> = idx
            .records()
            .map(|r| (r.n_upm_start, r.w_upm_size))
            .collect();
        for (name, recs, flen) in [("pcm", &mut pr, pcm_len), ("upm", &mut ur, upm_len)] {
            recs.sort_unstable();
            let mut prev_end: Option<u32> = None;
            let mut overlaps = 0usize;
            let mut gaps = 0usize;
            for &(s, sz) in recs.iter() {
                let e = s + u32::from(sz);
                if let Some(pe) = prev_end {
                    if s < pe {
                        overlaps += 1;
                    }
                    if s > pe {
                        gaps += 1;
                    }
                }
                prev_end = Some(prev_end.map_or(e, |pe: u32| pe.max(e)));
            }
            assert_eq!(overlaps, 0, "{name} overlaps");
            assert_eq!(gaps, 0, "{name} gaps");
            assert_eq!(prev_end.unwrap() as usize, flen, "{name} end = file size");
        }
    }

    #[test]
    fn synth_upm_cumulative_equals_pcm_size() {
        let idx = parse_idx(&read(idx_path())).unwrap();
        let upm_data = read(format!(
            "{}/KSpeechDic/woman/synth.upm",
            data_dir().display()
        ));
        let upm = parse_upm(&upm_data).unwrap();
        let mut checked = 0usize;
        for rec in idx.records() {
            if rec.w_upm_size == 0 {
                continue;
            }
            let pos = upm.mark_positions(rec).expect("upm range");
            let cum: u32 = pos.last().copied().unwrap_or(0);
            assert_eq!(cum, u32::from(rec.w_pcm_size));
            checked += 1;
        }
        assert_eq!(checked, 269_127);
    }

    #[test]
    fn synth_upm_decode_marks() {
        let idx = parse_idx(&read(idx_path())).unwrap();
        let upm_data = read(format!(
            "{}/KSpeechDic/woman/synth.upm",
            data_dir().display()
        ));
        let upm = parse_upm(&upm_data).unwrap();
        let r0 = &idx.units[0].records[0];
        assert_eq!(r0.w_upm_size, 5);
        let marks = upm.decode_marks(r0, 0).unwrap();
        assert_eq!(marks.len(), 3);
        assert_eq!(marks[0].addr, 0);
        assert_eq!(marks[0].cur_pitch, 0x7d);
        assert_eq!(marks[0].next_pitch, 0x64);
        assert_eq!(marks[2].addr, 0x7d + 0x64);
        let pos = upm.mark_positions(r0).unwrap();
        assert_eq!(
            pos,
            vec![
                0x8e,
                0x8e + 0x7d,
                0x8e + 0x7d + 0x64,
                0x8e + 0x7d + 0x64 + 0x56,
                548
            ]
        );
    }

    #[test]
    fn synth_pcm_uraw() {
        let data = read(format!(
            "{}/KSpeechDic/woman/synth.pcm",
            data_dir().display()
        ));
        assert_eq!(data.len(), 406_216_419);
        let pcm = parse_pcm(&data).unwrap();
        assert_eq!(pcm.byte_len(), 406_216_419);
        assert_eq!(data[0], 0xc1);
        assert_eq!(data[1], 0xc2);
        assert_eq!(uraw_to_pcm(i8::from_ne_bytes([0xc1])), 1820);
        assert_eq!(uraw_to_pcm(i8::from_ne_bytes([0xff])), 0);
        assert_eq!(uraw_to_pcm(0x00), -32124);
    }

    #[test]
    fn synth_group_idx_full_parse() {
        let data = read(format!(
            "{}/KSpeechDic/woman/synth_group.idx",
            data_dir().display()
        ));
        let g = parse_group_idx(&data).expect("synth_group.idx parse");
        assert_eq!(g.groups.len(), 190);
        assert_eq!(g.groups[0].phones, vec![408]);
        assert_eq!(g.groups[1].phones, vec![412]);
        assert_eq!(g.groups[2].phones, vec![414, 415, 416]);
        assert_eq!(g.groups[7].phones.len(), 24);
        assert_eq!(g.groups[7].phones[0], 0);
        assert_eq!(g.groups[7].phones[23], 391);
    }

    #[test]
    fn pec_triangular_tables() {
        for name in ["energy", "pitch"] {
            let data = read(format!(
                "{}/KSpeechDic/p_e_c/{name}.tbl",
                data_dir().display()
            ));
            assert_eq!(data.len(), 80_402, "{name}.tbl size");
            let t = parse_triangular_table(&data).unwrap_or_else(|_| panic!("{name}.tbl parse"));
            assert_eq!(t.n, 200);
            assert_eq!(t.total_floats(), 20_100);
            assert_eq!(t.rows.len(), 200);
            assert_eq!(t.rows[0].len(), 1);
            assert_eq!(t.rows[199].len(), 200);
            for i in 0..200 {
                assert_eq!(t.at(i, i), Some(0.0), "{name} diagonal[{i}]");
            }
            let sat = t.rows.iter().flatten().filter(|&&v| v == 4.0).count();
            assert_eq!(sat, 199, "{name} count of saturated 4.0 values");
            let zeros = t.rows.iter().flatten().filter(|&&v| v == 0.0).count();
            assert_eq!(zeros, 200, "{name} zero value count");
        }
        let e = parse_triangular_table(&read(format!(
            "{}/KSpeechDic/p_e_c/energy.tbl",
            data_dir().display()
        )))
        .unwrap();
        let p = parse_triangular_table(&read(format!(
            "{}/KSpeechDic/p_e_c/pitch.tbl",
            data_dir().display()
        )))
        .unwrap();
        let neq = e
            .rows
            .iter()
            .flatten()
            .zip(p.rows.iter().flatten())
            .filter(|(a, b)| a != b)
            .count();
        assert!(neq > 0);
    }
}
