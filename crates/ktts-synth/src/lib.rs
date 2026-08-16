#[allow(
    dead_code,
    unreachable_pub,
    reason = "ported engine implementation stays internal"
)]
mod codec;
#[allow(
    dead_code,
    unreachable_pub,
    reason = "ported engine implementation stays internal"
)]
mod consts;
#[allow(
    unreachable_pub,
    reason = "ported engine implementation stays internal"
)]
mod context;
#[allow(unreachable_pub, reason = "voice engine units stay behind the facade")]
mod setting;
#[allow(
    unreachable_pub,
    reason = "ported engine implementation stays internal"
)]
mod tables;
mod types;

#[cfg(test)]
use ktts_prosody as _;
#[allow(
    dead_code,
    unreachable_pub,
    reason = "ported engine implementation stays internal"
)]
mod unitselect;
#[allow(
    dead_code,
    unreachable_pub,
    reason = "ported engine implementation stays internal"
)]
mod waveform;

use std::path::{Path, PathBuf};

use context::Phrase;
use ktts_dict::synthdb::{
    PhoneDict, SynthGroupIdx, SynthIdx, parse_group_idx, parse_idx, parse_triangular_table,
};
use setting::{IniParams, SynthParams};
use unitselect::BestPhone;

pub use types::{PronSyllable, PronText, SyllableTarget, VoiceParams};

pub type DataMap = ktts_dict::common::DataMap;

#[derive(Debug)]
pub enum SynthError {
    Io(std::io::Error),
    Parse(String),
    Invalid(String),
}

impl std::fmt::Display for SynthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO: {e}"),
            Self::Parse(s) => write!(f, "parse: {s}"),
            Self::Invalid(s) => write!(f, "invalid input: {s}"),
        }
    }
}

impl std::error::Error for SynthError {}

impl From<std::io::Error> for SynthError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub type SynthResult<T> = Result<T, SynthError>;

#[derive(Debug)]
struct SynthDb {
    idx: SynthIdx,
    groups: SynthGroupIdx,
    pitch_tbl: ktts_dict::synthdb::TriangularTable,
    eng_tbl: ktts_dict::synthdb::TriangularTable,
    pcm_files: Vec<Option<Vec<u8>>>,
    upm_files: Vec<Option<Vec<u8>>>,
    codec_mode: u8,
    sample_rate: u32,
    base: PathBuf,
}

const PCM_FILE_NAMES: [&str; 3] = ["synth.pcm", "synth-eng.pcm", "synth-num.pcm"];
const PCM_FILE_NAMES_BIN: [&str; 3] = ["synth_bin.pcm", "synth-eng_bin.pcm", "synth-num_bin.pcm"];
const UPM_FILE_NAMES: [&str; 3] = ["synth.upm", "synth-eng.upm", "synth-num.upm"];
const UPM_FILE_NAMES_BIN: [&str; 3] = ["synth_bin.upm", "synth-eng_bin.upm", "synth-num_bin.upm"];

impl SynthDb {
    /// Returns the phone record for a best-phone selection.
    ///
    /// # Errors
    ///
    /// Returns an error if the input is invalid.
    fn rec(&self, bp: BestPhone) -> SynthResult<&PhoneDict> {
        self.idx
            .units
            .get(bp.unit_no as usize)
            .and_then(|u| u.records.get(bp.type_no as usize))
            .ok_or_else(|| {
                SynthError::Invalid(format!(
                    "unit out of range: unit={} type={}",
                    bp.unit_no, bp.type_no
                ))
            })
    }
    #[expect(
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        reason = "C port: index/math casts with wrap semantics"
    )]
    /// Returns the decoded PCM segment for a phone record.
    ///
    /// # Errors
    ///
    /// Returns an error if the input is invalid.
    fn pcm_segment(&self, rec: &PhoneDict) -> SynthResult<Vec<i16>> {
        let data = self
            .pcm_files
            .get(rec.ch_dict_file_id as usize)
            .and_then(|f| f.as_ref())
            .ok_or_else(|| {
                SynthError::Invalid(format!(
                    "no PCM file for chDictFileID={} (synth-eng/synth-num not bundled: {}. \
                     the original engine gets a NULL FILE* on fopen failure and crashes when the record is used)",
                    rec.ch_dict_file_id,
                    PCM_FILE_NAMES[rec.ch_dict_file_id.clamp(0, 2) as usize]
                ))
            })?;
        let n_read: usize = match self.codec_mode {
            0 => rec.w_pcm_size as usize * 2,
            5 => codec::get_codec_byte_number(rec.w_pcm_size, self.sample_rate as i32).max(0)
                as usize,
            _ => rec.w_pcm_size as usize,
        };
        let start = rec.n_pcm_start as usize;
        let end = start.saturating_add(n_read).min(data.len());
        let raw = &data[start..end];
        codec::decode_pcm(
            self.codec_mode,
            raw,
            rec.w_pcm_size,
            self.sample_rate as i32,
        )
        .map_err(SynthError::Invalid)
    }
    #[expect(
        clippy::cast_sign_loss,
        reason = "C port: index/math casts with wrap semantics"
    )]
    fn upm_segment(&self, rec: &PhoneDict) -> SynthResult<Vec<u8>> {
        let data = self
            .upm_files
            .get(rec.ch_dict_file_id as usize)
            .and_then(|f| f.as_ref())
            .ok_or_else(|| {
                SynthError::Invalid(format!(
                    "no UPM file for chDictFileID={} (synth-eng/synth-num not bundled)",
                    rec.ch_dict_file_id
                ))
            })?;
        let rng = rec.upm_range();
        data.get(rng.clone()).map(<[u8]>::to_vec).ok_or_else(|| {
            SynthError::Invalid(format!(
                "UPM range out of bounds: {:?} (len={})",
                rng,
                data.len()
            ))
        })
    }
}

#[derive(Debug)]
pub struct SynthContext {
    db: SynthDb,
    /// Engine-unit voice parameters loaded from `InfoDic.wdic`.
    base: IniParams,
}

impl SynthContext {
    fn params(&self, params: VoiceParams) -> SynthParams {
        IniParams::from_api(self.base, params).params()
    }
    #[cfg(test)]
    #[must_use]
    const fn db_ref(&self) -> &SynthDb {
        &self.db
    }
}

/// Loads the synthesis database for a gender from a directory.
///
/// # Errors
///
/// Returns an error if the input is invalid.
pub fn load_synth_db(dir: &Path, gender: &str) -> SynthResult<SynthContext> {
    let d = dir.join(gender);
    if !d.is_dir() {
        return Err(SynthError::Invalid(format!(
            "voice DB directory does not exist: {}",
            d.display()
        )));
    }
    let mut files: DataMap = std::collections::HashMap::new();
    let mut add = |rel: &str, p: &Path| {
        if let Ok(data) = std::fs::read(p) {
            files.insert(rel.to_string(), data);
        }
    };
    for name in [
        "synth.idx",
        "synth_group.idx",
        "synth.pcm",
        "synth.upm",
        "synth-eng.pcm",
        "synth-eng.upm",
        "synth-num.pcm",
        "synth-num.upm",
        "synth_bin.pcm",
        "synth_bin.upm",
        "synth-eng_bin.pcm",
        "synth-eng_bin.upm",
        "synth-num_bin.pcm",
        "synth-num_bin.upm",
    ] {
        add(&format!("KSpeechDic/{gender}/{name}"), &d.join(name));
    }
    let pec = dir.join("p_e_c");
    add("KSpeechDic/p_e_c/pitch.tbl", &pec.join("pitch.tbl"));
    add("KSpeechDic/p_e_c/energy.tbl", &pec.join("energy.tbl"));
    let wdic = if dir.join("InfoDic.wdic").exists() {
        dir.join("InfoDic.wdic")
    } else {
        dir.parent().unwrap_or(dir).join("InfoDic.wdic")
    };
    add("InfoDic.wdic", &wdic);
    let mut ctx = load_synth_db_bytes(files, gender)?;
    ctx.db.base = dir.to_path_buf();
    Ok(ctx)
}

/// Loads the synthesis database for a gender from a data map.
///
/// Consumes the map: the (large) PCM/UPM blobs are moved out of it instead of
/// being cloned, so the caller's peak memory is roughly halved. The map is
/// borrowed for the small `InfoDic.wdic` lookups only.
///
/// # Errors
///
/// Returns an error if the input is invalid.
pub fn load_synth_db_bytes(mut files: DataMap, gender: &str) -> SynthResult<SynthContext> {
    let idx = parse_idx(&take_file(
        &mut files,
        &format!("KSpeechDic/{gender}/synth.idx"),
    )?)
    .map_err(|e| SynthError::Parse(e.to_string()))?;
    let groups = parse_group_idx(&take_file(
        &mut files,
        &format!("KSpeechDic/{gender}/synth_group.idx"),
    )?)
    .map_err(|e| SynthError::Parse(e.to_string()))?;
    let pitch_tbl = parse_triangular_table(&take_file(&mut files, "KSpeechDic/p_e_c/pitch.tbl")?)
        .map_err(|e| SynthError::Parse(e.to_string()))?;
    let eng_tbl = parse_triangular_table(&take_file(&mut files, "KSpeechDic/p_e_c/energy.tbl")?)
        .map_err(|e| SynthError::Parse(e.to_string()))?;
    let codec_mode = read_codec_mode_bytes(&files);
    let (pcm_names, upm_names) = if codec_mode == codec::CODEC_G729 {
        (PCM_FILE_NAMES_BIN, UPM_FILE_NAMES_BIN)
    } else {
        (PCM_FILE_NAMES, UPM_FILE_NAMES)
    };
    let mut pcm_files: Vec<Option<Vec<u8>>> = Vec::with_capacity(3);
    for name in pcm_names {
        pcm_files.push(files.remove(&format!("KSpeechDic/{gender}/{name}")));
    }
    let mut upm_files: Vec<Option<Vec<u8>>> = Vec::with_capacity(3);
    for name in upm_names {
        upm_files.push(files.remove(&format!("KSpeechDic/{gender}/{name}")));
    }
    let pcm0 = pcm_files[0].as_ref().ok_or_else(|| {
        SynthError::Invalid(format!(
            "cannot open synth.pcm: KSpeechDic/{gender}/synth.pcm"
        ))
    })?;
    if pcm0.len() < 2 {
        return Err(SynthError::Invalid(format!(
            "synth.pcm too short: {} bytes",
            pcm0.len()
        )));
    }
    let db = SynthDb {
        idx,
        groups,
        pitch_tbl,
        eng_tbl,
        pcm_files,
        upm_files,
        codec_mode,
        sample_rate: 16000,
        base: PathBuf::from(format!("KSpeechDic/{gender}")),
    };
    let base = read_ini_params_bytes(&files);
    Ok(SynthContext { db, base })
}

fn take_file(files: &mut DataMap, key: &str) -> SynthResult<Vec<u8>> {
    files.remove(key).ok_or_else(|| {
        SynthError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{key}: key not found in data map"),
        ))
    })
}

fn read_codec_mode_bytes(files: &DataMap) -> u8 {
    if let Some(data) = files.get("InfoDic.wdic")
        && let Ok(w) = ktts_dict::wdic::parse(data)
        && let Some(v) = w
            .get_str("KOREAPCM")
            .and_then(|s| codec::codec_from_name(&s))
    {
        return v;
    }
    0
}

fn read_ini_params_bytes(files: &DataMap) -> IniParams {
    if let Some(data) = files.get("InfoDic.wdic")
        && let Ok(w) = ktts_dict::wdic::parse(data)
    {
        let get = |k: &str, def: i32| -> i32 {
            w.get_str(k)
                .and_then(|s| s.trim().parse::<i32>().ok())
                .unwrap_or(def)
        };
        return IniParams {
            pitch: get("PITCH", 150),
            speed: get("SPEED", 100),
            voice: get("VOICE", 100),
            volume: get("VOLUME", 150),
        };
    }
    IniParams::defaults()
}

/// Synthesizes PCM samples for a pronunciation text.
///
/// # Errors
///
/// Returns an error if the input is invalid.
pub fn synthesize(
    ctx: &SynthContext,
    text: &PronText,
    targets: &[SyllableTarget],
) -> SynthResult<Vec<i16>> {
    synthesize_with_params(ctx, text, targets, VoiceParams::default())
}

/// Synthesizes PCM samples with frontend voice controls.
///
/// Voice database defaults, engine-unit conversion, and clamping stay inside
/// this module, and the context is not mutated between calls.
///
/// # Errors
///
/// Returns an error if the input is invalid.
pub fn synthesize_with_params(
    ctx: &SynthContext,
    text: &PronText,
    targets: &[SyllableTarget],
    params: VoiceParams,
) -> SynthResult<Vec<i16>> {
    let phrase = context::build_phrase(text, targets).map_err(SynthError::Invalid)?;
    synthesize_phrase_with_params(ctx, &phrase, ctx.params(params))
}

fn synthesize_phrase_with_params(
    ctx: &SynthContext,
    phrase: &Phrase,
    params: SynthParams,
) -> SynthResult<Vec<i16>> {
    let uctx = unitselect::SynthCtx {
        idx: &ctx.db.idx,
        groups: &ctx.db.groups,
        pitch_tbl: &ctx.db.pitch_tbl,
        eng_tbl: &ctx.db.eng_tbl,
    };
    let selection = unitselect::select_units(&uctx, phrase);
    waveform::synthesize_wave(&ctx.db, phrase, &selection, params)
        .map_err(|e| SynthError::Invalid(e.to_string()))
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "test fixtures: oracle values converted with intentional casts"
    )]
    use super::*;

    fn data_dir() -> PathBuf {
        PathBuf::from(
            std::env::var("KTTSDB_DIR")
                .expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
        )
        .join("KSpeechDic")
    }

    fn syl(cvc: &str, word: usize, start: bool) -> PronSyllable {
        PronSyllable {
            cvc: cvc.to_string(),
            word_idx: word,
            is_word_start: start,
            pos: 0,
        }
    }

    fn tgt_for(cvc: &str, dur: f32, f0: f32) -> SyllableTarget {
        let total = dur * 16.0;
        let bytes: Vec<u8> = cvc.bytes().collect();
        let has_cho = bytes
            .first()
            .is_some_and(|&b| b != 0 && b != 1 && b != 0x0d);
        let has_jong = bytes.get(2).is_some_and(|&b| b != 0 && b != 1);
        let mut out = [0u16; 3];
        if has_cho {
            out[0] = (total * 0.3 + 0.5) as u16;
        }
        out[1] = (total
            * (0.5 + if has_cho { 0.0 } else { 0.3 } + if has_jong { 0.0 } else { 0.2 })
            + 0.5) as u16;
        if has_jong {
            out[2] = (total * 0.2 + 0.5) as u16;
        }
        SyllableTarget {
            dur,
            ave_length: out,
            f0: [f0; 12],
            tobi: 0.0,
            boundary: 0,
        }
    }

    #[test]
    #[ignore = "requires real dictionary data in KTTSDB_DIR"]
    fn load_bytes_equiv_path_load() {
        let dir = data_dir();
        let c1 = load_synth_db(&dir, "woman").expect("path load");
        let mut files: DataMap = std::collections::HashMap::new();
        let mut add = |rel: &str, p: &Path| {
            if let Ok(data) = std::fs::read(p) {
                files.insert(rel.to_string(), data);
            }
        };
        for name in [
            "synth.idx",
            "synth_group.idx",
            "synth.pcm",
            "synth.upm",
            "synth-eng.pcm",
            "synth-eng.upm",
            "synth-num.pcm",
            "synth-num.upm",
        ] {
            add(
                &format!("KSpeechDic/woman/{name}"),
                &dir.join("woman").join(name),
            );
        }
        add(
            "KSpeechDic/p_e_c/pitch.tbl",
            &dir.join("p_e_c").join("pitch.tbl"),
        );
        add(
            "KSpeechDic/p_e_c/energy.tbl",
            &dir.join("p_e_c").join("energy.tbl"),
        );
        let wdic = if dir.join("InfoDic.wdic").exists() {
            dir.join("InfoDic.wdic")
        } else {
            dir.parent().unwrap_or(dir.as_path()).join("InfoDic.wdic")
        };
        add("InfoDic.wdic", &wdic);
        let c2 = load_synth_db_bytes(files, "woman").expect("bytes load");

        assert_eq!(c1.db_ref().codec_mode, c2.db_ref().codec_mode);
        assert_eq!(c1.db_ref().sample_rate, c2.db_ref().sample_rate);
        assert_eq!(
            c1.params(VoiceParams::default()),
            c2.params(VoiceParams::default())
        );
        assert_eq!(c1.db_ref().idx, c2.db_ref().idx);
        assert_eq!(c1.db_ref().groups, c2.db_ref().groups);
        assert_eq!(c1.db_ref().pitch_tbl, c2.db_ref().pitch_tbl);
        assert_eq!(c1.db_ref().eng_tbl, c2.db_ref().eng_tbl);
        for i in 0..3 {
            let rec = &c1.db_ref().idx.units[0].records[i];
            let p1 = c1.db_ref().pcm_segment(rec).expect("path decode");
            let p2 = c2.db_ref().pcm_segment(rec).expect("bytes decode");
            assert_eq!(p1, p2, "PCM mismatch for record {i}");
        }
        let text = PronText {
            syllables: vec![syl("\x0d\x03\x01", 0, true)],
            phoneme_codes: vec![0x0d, 0x03, 0x01],

            word_sen: vec![],
        };
        let targets = vec![tgt_for("\x0d\x03\x01", 150.0, 200.0)];
        let w1 = synthesize(&c1, &text, &targets).expect("path synthesis");
        let w2 = synthesize(&c2, &text, &targets).expect("bytes synthesis");
        assert_eq!(w1, w2);
    }
}
