use std::path::Path;

use ktts_dict::common::DataMap;

use crate::types::{
    Morph, PipelineError, PronSyllable, PronText, SyllableTarget, VoiceParams, WordAnal,
    WordMorphInfo,
};

pub const VOICE: &str = "woman";

/// Builds a data map from a kttsdb directory.
///
/// Keys are paths relative to the root with `/` separators (e.g.
/// `KLangDic/KMPADict/kmorph_hash.bin`), which is exactly the key layout the
/// engine `*_bytes` loaders expect.
///
/// # Errors
///
/// Returns an error if the directory cannot be read.
///
/// # Panics
///
/// Panics if a walked path cannot be relativized to the data directory
/// (impossible for paths produced by walking the directory itself).
pub fn load_datamap(data_dir: &Path) -> Result<DataMap, PipelineError> {
    let mut files = DataMap::new();
    let mut stack = vec![data_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|e| {
            PipelineError::Engine("ktts-cli", format!("read_dir {}: {e}", dir.display()))
        })?;
        for entry in entries {
            let entry = entry.map_err(|e| {
                PipelineError::Engine("ktts-cli", format!("read_dir entry: {e}"))
            })?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(data) = std::fs::read(&path) {
                let rel = path
                    .strip_prefix(data_dir)
                    .expect("paths are walked under the data dir");
                files.insert(rel.to_string_lossy().replace('\\', "/"), data);
            }
        }
    }
    Ok(files)
}

/// Runs the full synthesis pipeline: KMA, pronunciation, prosody, synthesis.
///
/// # Errors
///
/// Returns an error if any engine stage fails.
pub fn run_pipeline(
    text: &str,
    data_dir: &Path,
    params: &VoiceParams,
) -> Result<Vec<i16>, PipelineError> {
    let files = load_datamap(data_dir)?;
    run_pipeline_files(text, files, params)
}

/// Runs the full synthesis pipeline against pre-loaded dictionary data.
///
/// The map is consumed: the synthesis stage moves the (large) PCM/UPM blobs
/// out of it instead of cloning them.
///
/// # Errors
///
/// Returns an error if any engine stage fails.
pub fn run_pipeline_files(
    text: &str,
    files: DataMap,
    params: &VoiceParams,
) -> Result<Vec<i16>, PipelineError> {
    let text_u16 = crate::codec::utf8_to_u16(text);

    let words: Vec<WordAnal> = stage_kma(&text_u16, &files)?;

    let pron: PronText = stage_pron(&words, &files)?;

    let mut targets: Vec<SyllableTarget> = stage_prosody(&pron, &words, &files)?;

    apply_sentence_end_boundaries(&pron, &words, &mut targets);

    if targets.is_empty() {
        return Ok(Vec::new());
    }

    let samples: Vec<i16> = stage_synth(&pron, &targets, files, params)?;

    Ok(samples)
}

fn stage_kma(text_u16: &[u16], files: &DataMap) -> Result<Vec<WordAnal>, PipelineError> {
    let ctx = ktts_kma::load_kma_dicts_bytes(files)
        .map_err(|e| PipelineError::Engine("ktts-kma", format!("load_kma_dicts: {e}")))?;
    let text = crate::codec::u16_to_utf8(text_u16);
    let raw = ktts_kma::analyze(&ctx, &text)
        .map_err(|e| PipelineError::Engine("ktts-kma", format!("analyze: {e}")))?;
    Ok(raw.into_iter().map(conv_kma_word).collect())
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: sentence marker byte truncation"
)]
fn conv_kma_word(w: ktts_kma::WordAnal) -> WordAnal {
    WordAnal {
        morphs: w
            .morphs
            .into_iter()
            .map(|m| Morph {
                cvc: m.cvc,
                pos: m.pos,
                prob: m.prob,
                surface_len: m.surface_len,
            })
            .collect(),
        w_byte_num: w.w_byte_num,
        word_cvc: w.word_cvc,
        b_word_sen: w.b_word_sen,
        b_word_sen_char: if w.b_word_sen {
            w.source.last().copied().unwrap_or(0) as u8
        } else {
            0
        },
        b_sentence_end: w.b_sentence_end,
        source: w.source,
    }
}

fn apply_sentence_end_boundaries(
    pron: &PronText,
    words: &[WordAnal],
    targets: &mut [SyllableTarget],
) {
    debug_assert_eq!(
        pron.syllables.len(),
        targets.len(),
        "syllable count does not match target count"
    );
    let mut last_syllable_of_word: Vec<usize> = Vec::with_capacity(words.len());
    for (i, s) in pron.syllables.iter().enumerate() {
        while last_syllable_of_word.len() <= s.word_idx {
            last_syllable_of_word.push(0);
        }
        last_syllable_of_word[s.word_idx] = i;
    }
    for (w, word) in words.iter().enumerate() {
        if word.b_sentence_end
            && let Some(&idx) = last_syllable_of_word.get(w)
            && let Some(t) = targets.get_mut(idx)
        {
            t.boundary = 0x15;
        }
    }
}

fn stage_pron(words: &[WordAnal], files: &DataMap) -> Result<PronText, PipelineError> {
    let pron_words: Vec<ktts_pron::kma_types::WordAnal> =
        words.iter().map(conv_my_word_to_pron).collect();

    let ctx = ktts_pron::load_pron_dicts_bytes(files)
        .map_err(|e| PipelineError::Engine("ktts-pron", format!("load_pron_dicts: {e}")))?;

    let mut ranges = sentence_word_ranges(words);
    if ranges.is_empty() && !words.is_empty() {
        ranges.push((0, words.len() - 1));
    }
    let mut syllables: Vec<PronSyllable> = Vec::new();
    let mut phoneme_codes: Vec<u8> = Vec::new();
    let mut word_morphs: Vec<WordMorphInfo> = Vec::new();
    for (ws, we) in ranges {
        let raw = ktts_pron::pronounce(&ctx, &pron_words[ws..=we]).map_err(|e| {
            PipelineError::Engine(
                "ktts-pron",
                format!("pronounce (sentence words {ws}..={we}): {e}"),
            )
        })?;
        let mut part = conv_pron_to_my(&raw);
        for s in &mut part.syllables {
            s.word_idx += ws;
        }
        syllables.extend(part.syllables);
        phoneme_codes.extend_from_slice(&part.phoneme_codes);
        word_morphs.extend(part.word_morphs);
    }
    let mut my = PronText {
        syllables,
        phoneme_codes,
        word_morphs,
        word_sen: vec![],
    };
    my.word_sen = words.iter().map(|w| w.b_word_sen_char).collect();
    Ok(my)
}

fn conv_my_word_to_pron(w: &WordAnal) -> ktts_pron::kma_types::WordAnal {
    ktts_pron::kma_types::WordAnal {
        morphs: w
            .morphs
            .iter()
            .map(|m| ktts_pron::kma_types::Morph {
                cvc: m.cvc.clone(),
                pos: m.pos,
                prob: m.prob,
                surface_len: m.surface_len,
            })
            .collect(),
        w_byte_num: w.w_byte_num as usize,
        word_cvc: w.word_cvc.clone(),
        source: w.source.clone(),
        b_word_sen: w.b_word_sen,
    }
}

fn conv_pron_to_my(p: &ktts_pron::PronText) -> PronText {
    PronText {
        syllables: p
            .syllables
            .iter()
            .map(|s| PronSyllable {
                cvc: s.cvc.clone(),
                word_idx: s.word_idx,
                is_word_start: s.is_word_start,
                pos: s.pos,
                morph_idx: s.morph_idx,
                morph_pos: s.morph_pos,
            })
            .collect(),
        phoneme_codes: p.phoneme_codes.clone(),
        word_morphs: p
            .word_morphs
            .iter()
            .map(|w| WordMorphInfo {
                pos: w.pos.clone(),
                first_chars: w.first_chars.clone(),
                surfaces: w.surfaces.clone(),
                source: w.source.clone(),
            })
            .collect(),
        word_sen: vec![],
    }
}

fn sentence_word_ranges(words: &[WordAnal]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = 0usize;
    for (i, w) in words.iter().enumerate() {
        if w.b_sentence_end {
            ranges.push((start, i));
            start = i + 1;
        }
    }
    if start < words.len() {
        ranges.push((start, words.len() - 1));
    }
    ranges
}

fn slice_sentence(
    pron: &PronText,
    words: &[WordAnal],
    w_start: usize,
    w_end: usize,
) -> ktts_prosody::PronText {
    let syllables = pron
        .syllables
        .iter()
        .filter(|s| s.word_idx >= w_start && s.word_idx <= w_end)
        .map(|s| {
            let mut cvc = [1u8; 3];
            let n = s.cvc.len().min(3);
            cvc[..n].copy_from_slice(&s.cvc[..n]);
            ktts_prosody::PronSyllable {
                cvc,
                word_idx: s.word_idx - w_start,
                is_word_start: s.is_word_start,
                pos: s.pos[0],
                morph_idx: s.morph_idx,
                morph_pos: s.morph_pos,
            }
        })
        .collect();
    let word_morphs = pron
        .word_morphs
        .get(w_start..=w_end)
        .map(|wm| {
            wm.iter()
                .map(|w| ktts_prosody::WordMorphs {
                    pos: w.pos.clone(),
                    first_chars: w.first_chars.clone(),
                    surfaces: w.surfaces.clone(),
                    source: w.source.clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    ktts_prosody::PronText {
        syllables,
        phoneme_codes: vec![],
        word_morphs,
        word_sen: words
            .get(w_start..=w_end)
            .map(|ws| ws.iter().map(|w| w.b_word_sen_char).collect())
            .unwrap_or_default(),
    }
}

fn stage_prosody(
    pron: &PronText,
    words: &[WordAnal],
    files: &DataMap,
) -> Result<Vec<SyllableTarget>, PipelineError> {
    let ctx = ktts_prosody::load_prosody_dicts_bytes(files, VOICE).map_err(|e| {
        PipelineError::Engine("ktts-prosody", format!("load_prosody_dicts: {e}"))
    })?;

    let mut ranges = sentence_word_ranges(words);
    if ranges.is_empty() && !words.is_empty() {
        ranges.push((0, words.len() - 1));
    }
    let mut out: Vec<SyllableTarget> = Vec::with_capacity(pron.syllables.len());
    for (ws, we) in ranges {
        let sub = slice_sentence(pron, words, ws, we);
        let raw = ktts_prosody::prosody(&ctx, &sub).map_err(|e| {
            PipelineError::Engine(
                "ktts-prosody",
                format!("prosody (sentence words {ws}..={we}): {e}"),
            )
        })?;
        out.extend(raw.into_iter().map(conv_target_to_my));
    }
    Ok(out)
}

#[cfg(test)]
fn conv_pron_to_prosody(p: &PronText) -> ktts_prosody::PronText {
    let syllables = p
        .syllables
        .iter()
        .map(|s| {
            let mut cvc = [1u8; 3];
            let n = s.cvc.len().min(3);
            cvc[..n].copy_from_slice(&s.cvc[..n]);
            ktts_prosody::PronSyllable {
                cvc,
                word_idx: s.word_idx,
                is_word_start: s.is_word_start,
                pos: s.pos[0],
                morph_idx: s.morph_idx,
                morph_pos: s.morph_pos,
            }
        })
        .collect();
    ktts_prosody::PronText {
        syllables,
        phoneme_codes: p.phoneme_codes.clone(),
        word_morphs: p
            .word_morphs
            .iter()
            .map(|w| ktts_prosody::WordMorphs {
                pos: w.pos.clone(),
                first_chars: w.first_chars.clone(),
                surfaces: w.surfaces.clone(),
                source: w.source.clone(),
            })
            .collect(),
        word_sen: vec![],
    }
}

const fn conv_target_to_my(t: ktts_prosody::SyllableTarget) -> SyllableTarget {
    SyllableTarget {
        dur: t.dur,
        ave_length: t.ave_length,
        f0: t.f0,
        tobi: t.tobi,
        boundary: t.boundary,
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: intentional truncation"
)]
fn stage_synth(
    pron: &PronText,
    targets: &[SyllableTarget],
    files: DataMap,
    params: &VoiceParams,
) -> Result<Vec<i16>, PipelineError> {
    let mut ctx = ktts_synth::load_synth_db_bytes(files, VOICE)
        .map_err(|e| PipelineError::Engine("ktts-synth", format!("load_synth_db: {e}")))?;

    if (params.speed - 1.0).abs() > 1e-6 {
        ctx.set_speed((100.0 * params.speed) as i32);
    }
    if params.pitch.abs() > 1e-6 {
        ctx.set_pitch((150.0 * (1.0 + params.pitch)) as i32);
    }
    if (params.volume - 1.0).abs() > 1e-6 {
        ctx.set_volume((150.0 * params.volume) as i32);
    }

    let input = conv_pron_to_synth(pron);
    let tgts: Vec<ktts_synth::SyllableTarget> = targets
        .iter()
        .map(|t| ktts_synth::SyllableTarget {
            dur: t.dur,
            ave_length: t.ave_length,
            f0: t.f0,
            tobi: t.tobi,
            boundary: t.boundary,
        })
        .collect();

    ktts_synth::synthesize(&ctx, &input, &tgts)
        .map_err(|e| PipelineError::Engine("ktts-synth", format!("synthesize: {e}")))
}

fn conv_pron_to_synth(p: &PronText) -> ktts_synth::PronText {
    let syllables: Vec<ktts_synth::PronSyllable> = p
        .syllables
        .iter()
        .map(|s| ktts_synth::PronSyllable {
            cvc: s.cvc.iter().map(|&b| b as char).collect(),
            word_idx: s.word_idx,
            is_word_start: s.is_word_start,
            pos: s.pos[0],
        })
        .collect();
    ktts_synth::PronText {
        syllables,
        phoneme_codes: p.phoneme_codes.clone(),
        word_sen: p.word_sen.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_yields_empty_pcm() {
        let err = run_pipeline("   ", Path::new("/nonexistent"), &VoiceParams::default());
        assert!(
            matches!(err, Err(PipelineError::Engine(name, _)) if name == "ktts-cli"),
            "expected Engine(ktts-cli) for missing data dir, got {err:?}"
        );
    }

    #[test]
    fn pipeline_errors_on_missing_data_dir() {
        let err = run_pipeline(
            "안녕하세요",
            Path::new("/nonexistent"),
            &VoiceParams::default(),
        );
        assert!(
            matches!(err, Err(PipelineError::Engine(name, _)) if name == "ktts-cli"),
            "expected Engine(ktts-cli), got {err:?}"
        );
    }

    #[test]
    fn pron_word_conversion_carries_b_word_sen() {
        let w = WordAnal {
            morphs: vec![Morph {
                cvc: vec![13, 3, 2],
                pos: [b'n', 0],
                prob: -1.5,
                surface_len: 1,
            }],
            w_byte_num: 2,
            word_cvc: vec![13, 3, 2],
            b_word_sen: true,
            b_word_sen_char: b'?',
            b_sentence_end: true,
            source: vec![],
        };
        let pw = conv_my_word_to_pron(&w);
        assert_eq!(pw.morphs.len(), 1);
        assert_eq!(pw.morphs[0].cvc, vec![13, 3, 2]);
        assert_eq!(pw.morphs[0].pos, [b'n', 0]);
        assert_eq!(pw.w_byte_num, 2usize);
        assert_eq!(pw.word_cvc, vec![13, 3, 2]);
        assert!(pw.b_word_sen);
    }

    #[test]
    fn conv_kma_word_copies_sentence_end() {
        let w = ktts_kma::WordAnal {
            morphs: vec![],
            w_byte_num: 2,
            word_cvc: vec![13, 3, 2],
            b_word_sen: false,
            b_sentence_end: true,
            source: vec![],
        };
        let mine = conv_kma_word(w);
        assert!(mine.b_sentence_end);
        assert!(!mine.b_word_sen);
    }

    #[test]
    fn sentence_end_boundaries_forced_on_mid_sentence_last_syllable() {
        let syl = |cvc: Vec<u8>, word_idx: usize| PronSyllable {
            cvc,
            word_idx,
            is_word_start: false,
            pos: [b'0', 0],
            morph_idx: 0,
            morph_pos: 0,
        };
        let pron = PronText {
            syllables: vec![
                syl(vec![13, 3, 1], 0),
                syl(vec![16, 29, 1], 0),
                syl(vec![8, 27, 5], 0),
                syl(vec![9, 29, 5], 1),
                syl(vec![4, 3, 1], 1),
                syl(vec![7, 3, 1], 1),
                syl(vec![13, 27, 5], 2),
                syl(vec![2, 27, 1], 2),
                syl(vec![8, 10, 1], 2),
                syl(vec![14, 3, 1], 3),
                syl(vec![13, 21, 5], 3),
                syl(vec![5, 13, 1], 3),
            ],
            phoneme_codes: vec![],
            word_morphs: vec![],

            word_sen: vec![],
        };
        let words = vec![
            WordAnal {
                morphs: vec![],
                w_byte_num: 3,
                word_cvc: vec![],
                b_word_sen: false,
                b_word_sen_char: 0,
                b_sentence_end: false,
                source: vec![],
            },
            WordAnal {
                morphs: vec![],
                w_byte_num: 3,
                word_cvc: vec![],
                b_word_sen: false,
                b_word_sen_char: 0,
                b_sentence_end: true,
                source: vec![],
            },
            WordAnal {
                morphs: vec![],
                w_byte_num: 3,
                word_cvc: vec![],
                b_word_sen: false,
                b_word_sen_char: 0,
                b_sentence_end: false,
                source: vec![],
            },
            WordAnal {
                morphs: vec![],
                w_byte_num: 3,
                word_cvc: vec![],
                b_word_sen: false,
                b_word_sen_char: 0,
                b_sentence_end: true,
                source: vec![],
            },
        ];
        let mut targets: Vec<SyllableTarget> = (0..12)
            .map(|i| SyllableTarget {
                dur: 100.0,
                ave_length: [0; 3],
                f0: [150.0; 12],
                tobi: 0.0,
                boundary: match i {
                    2 | 8 => 0x0a,
                    5 => 0x0b,
                    11 => 0x15,
                    _ => 0x00,
                },
            })
            .collect();
        apply_sentence_end_boundaries(&pron, &words, &mut targets);
        assert_eq!(targets[2].boundary, 0x0a);
        assert_eq!(targets[5].boundary, 0x15);
        assert_eq!(targets[8].boundary, 0x0a);
        assert_eq!(targets[11].boundary, 0x15);
    }

    #[test]
    fn pron_to_prosody_syllable_conversion() {
        let p = PronText {
            syllables: vec![
                PronSyllable {
                    cvc: vec![13, 3, 2],
                    word_idx: 0,
                    is_word_start: true,
                    pos: [b'n', 0],
                    morph_idx: 0,
                    morph_pos: 0,
                },
                PronSyllable {
                    cvc: vec![b'.'],
                    word_idx: 1,
                    is_word_start: true,
                    pos: [b'L', 0],
                    morph_idx: 0,
                    morph_pos: 0,
                },
            ],
            phoneme_codes: vec![13, 3, 2, b'.'],
            word_morphs: vec![],

            word_sen: vec![],
        };
        let pp = conv_pron_to_prosody(&p);
        assert_eq!(pp.syllables.len(), 2);
        assert_eq!(pp.syllables[0].cvc, [13, 3, 2]);
        assert_eq!(pp.syllables[0].pos, b'n');
        assert_eq!(pp.syllables[1].cvc, [b'.', 1, 1]);
        assert_eq!(pp.syllables[1].pos, b'L');
        assert_eq!(pp.phoneme_codes, vec![13, 3, 2, b'.']);
    }

    #[test]
    fn pron_to_synth_passes_raw_cvc_unchanged() {
        let p = PronText {
            syllables: vec![PronSyllable {
                cvc: vec![13, 3, 2],
                word_idx: 0,
                is_word_start: true,
                pos: [b'n', 0],
                morph_idx: 0,
                morph_pos: 0,
            }],
            phoneme_codes: vec![13, 3, 2],
            word_morphs: vec![],

            word_sen: vec![],
        };
        let sp = conv_pron_to_synth(&p);
        assert_eq!(sp.syllables.len(), 1);
        assert_eq!(sp.syllables[0].cvc, "\r\x03\x02");
        assert_eq!(sp.syllables[0].pos, b'n');
        assert_eq!(sp.phoneme_codes, vec![13, 3, 2]);
    }

    #[test]
    fn pron_to_synth_passes_raw_annyong() {
        let p = PronText {
            syllables: vec![
                PronSyllable {
                    cvc: vec![13, 3, 5],
                    word_idx: 0,
                    is_word_start: true,
                    pos: [b'n', 0],
                    morph_idx: 0,
                    morph_pos: 0,
                },
                PronSyllable {
                    cvc: vec![13, 11, 23],
                    word_idx: 0,
                    is_word_start: false,
                    pos: [b'n', 0],
                    morph_idx: 0,
                    morph_pos: 0,
                },
            ],
            phoneme_codes: vec![13, 3, 5, 13, 11, 23],
            word_morphs: vec![],

            word_sen: vec![],
        };
        let sp = conv_pron_to_synth(&p);
        assert_eq!(sp.syllables[0].cvc, "\x0d\x03\x05");
        assert_eq!(sp.syllables[1].cvc, "\x0d\x0b\x17");
        assert_eq!(sp.phoneme_codes, vec![13, 3, 5, 13, 11, 23]);
    }

    #[test]
    fn pron_to_synth_phoneme_codes_raw() {
        let p = PronText {
            syllables: vec![
                PronSyllable {
                    cvc: vec![13, 3, 5],
                    word_idx: 0,
                    is_word_start: true,
                    pos: [b'n', 0],
                    morph_idx: 0,
                    morph_pos: 0,
                },
                PronSyllable {
                    cvc: vec![1, 1, 19],
                    word_idx: 0,
                    is_word_start: false,
                    pos: [b'n', 0],
                    morph_idx: 0,
                    morph_pos: 0,
                },
                PronSyllable {
                    cvc: vec![b'.'],
                    word_idx: 1,
                    is_word_start: true,
                    pos: [b'L', 0],
                    morph_idx: 0,
                    morph_pos: 0,
                },
            ],
            phoneme_codes: vec![13, 3, 5, 1, 1, 19, b'.'],
            word_morphs: vec![],

            word_sen: vec![],
        };
        let sp = conv_pron_to_synth(&p);
        assert_eq!(sp.syllables[0].cvc, "\x0d\x03\x05");
        assert_eq!(sp.syllables[1].cvc, "\x01\x01\x13");
        assert_eq!(sp.syllables[2].cvc, ".");
        assert_eq!(sp.phoneme_codes, vec![13, 3, 5, 1, 1, 19, b'.']);
    }

    const ANTHEM_1LINE: &str =
        "아침은 빛나라 이 강산 은금에 자원도 가득한 이 세상 아름다운 내 조국 반만년 오랜 력사에";
    const ANTHEM_4LINE: &str =
        "아침은 빛나라 이 강산\n은금에 자원도 가득한\n이 세상 아름다운 내 조국\n반만년 오랜 력사에";

    fn test_data_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(
            std::env::var("KTTSDB_DIR")
                .expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
        )
    }

    fn test_files() -> DataMap {
        load_datamap(&test_data_dir()).expect("data map")
    }

    fn run_stages(text: &str) -> (Vec<WordAnal>, PronText, Vec<SyllableTarget>) {
        let files = test_files();
        let text_u16 = crate::codec::utf8_to_u16(text);
        let words = stage_kma(&text_u16, &files).expect("kma");
        let pron = stage_pron(&words, &files).expect("pron");
        let targets = stage_prosody(&pron, &words, &files).expect("prosody");
        (words, pron, targets)
    }

    fn word_end_boundaries(pron: &PronText, targets: &[SyllableTarget]) -> Vec<u8> {
        let mut last_syl: Vec<usize> = Vec::new();
        for (i, s) in pron.syllables.iter().enumerate() {
            while last_syl.len() <= s.word_idx {
                last_syl.push(0);
            }
            last_syl[s.word_idx] = i;
        }
        last_syl.iter().map(|&i| targets[i].boundary).collect()
    }

    fn synth_rest_flags(pron: &PronText, targets: &[SyllableTarget]) -> Vec<u8> {
        let sp = conv_pron_to_synth(pron);
        let tgts: Vec<ktts_synth::SyllableTarget> = targets
            .iter()
            .map(|t| ktts_synth::SyllableTarget {
                dur: t.dur,
                ave_length: t.ave_length,
                f0: t.f0,
                tobi: t.tobi,
                boundary: t.boundary,
            })
            .collect();
        let phrase = ktts_synth::context::build_phrase(&sp, &tgts).expect("build_phrase");
        phrase.words.iter().map(|w| w.rest_flag).collect()
    }

    #[test]
    fn sentence_word_ranges_splits_on_b_sentence_end() {
        let w = |end: bool| WordAnal {
            morphs: vec![],
            w_byte_num: 2,
            word_cvc: vec![],
            b_word_sen: false,
            b_word_sen_char: 0,
            b_sentence_end: end,
            source: vec![],
        };
        let words = vec![
            w(false),
            w(false),
            w(false),
            w(true),
            w(false),
            w(false),
            w(true),
        ];
        assert_eq!(sentence_word_ranges(&words), vec![(0, 3), (4, 6)],);
        let words2 = vec![w(false), w(false), w(true), w(false)];
        assert_eq!(sentence_word_ranges(&words2), vec![(0, 2), (3, 3)]);
        assert_eq!(sentence_word_ranges(&[w(false), w(false)]), vec![(0, 1)]);
    }

    #[test]
    fn slice_sentence_renumbers_word_idx_and_slices_morphs() {
        let syl = |cvc: Vec<u8>, word_idx: usize| PronSyllable {
            cvc,
            word_idx,
            is_word_start: false,
            pos: [b'0', 0],
            morph_idx: 1,
            morph_pos: 2,
        };
        let pron = PronText {
            syllables: vec![
                syl(vec![13, 3, 1], 0),
                syl(vec![16, 29, 1], 0),
                syl(vec![9, 29, 5], 1),
                syl(vec![4, 3, 1], 1),
                syl(vec![13, 27, 5], 2),
            ],
            phoneme_codes: vec![],
            word_morphs: vec![
                WordMorphInfo {
                    pos: vec![b'a'],
                    first_chars: vec![0xac00],
                    surfaces: vec![],
                    source: vec![],
                },
                WordMorphInfo {
                    pos: vec![b'b'],
                    first_chars: vec![0xbc00],
                    surfaces: vec![],
                    source: vec![],
                },
                WordMorphInfo {
                    pos: vec![b'c'],
                    first_chars: vec![0xcc00],
                    surfaces: vec![],
                    source: vec![],
                },
            ],

            word_sen: vec![],
        };
        let words = vec![
            WordAnal {
                morphs: vec![],
                w_byte_num: 6,
                word_cvc: vec![],
                b_word_sen: false,
                b_word_sen_char: 0,
                b_sentence_end: true,
                source: vec![],
            },
            WordAnal {
                morphs: vec![],
                w_byte_num: 6,
                word_cvc: vec![],
                b_word_sen: false,
                b_word_sen_char: 0,
                b_sentence_end: true,
                source: vec![],
            },
            WordAnal {
                morphs: vec![],
                w_byte_num: 6,
                word_cvc: vec![],
                b_word_sen: false,
                b_word_sen_char: 0,
                b_sentence_end: true,
                source: vec![],
            },
        ];
        let sub = slice_sentence(&pron, &words, 1, 1);
        assert_eq!(sub.syllables.len(), 2);
        assert_eq!(sub.syllables[0].word_idx, 0);
        assert_eq!(sub.syllables[1].word_idx, 0);
        assert_eq!(sub.syllables[0].cvc, [9, 29, 5]);
        assert_eq!(sub.syllables[1].cvc, [4, 3, 1]);
        assert_eq!(sub.syllables[0].morph_idx, 1);
        assert_eq!(sub.syllables[0].morph_pos, 2);
        assert_eq!(sub.word_morphs.len(), 1);
        assert_eq!(sub.word_morphs[0].pos, vec![b'b']);
        assert_eq!(sub.word_morphs[0].first_chars, vec![0xbc00]);
        let sub2 = slice_sentence(&pron, &words, 0, 1);
        assert_eq!(sub2.syllables.len(), 4);
        assert_eq!(sub2.word_morphs.len(), 2);
        assert_eq!(sub2.syllables[3].word_idx, 1);
    }

    #[test]
    fn between_word_rule_resets_at_sentence_boundary() {
        let (words, pron, _targets) = run_stages("좋은 날\n대 앞에 나무가 있다");
        assert!(words[1].b_sentence_end);
        let da = pron
            .syllables
            .iter()
            .find(|s| s.word_idx == 2)
            .expect("syllable of the first word (대) in sentence 2");
        assert_eq!(da.cvc, vec![5, 4, 1],);
        assert!(da.is_word_start);

        let (_w2, pron2, _t2) = run_stages("좋은 날 대 앞에 나무가 있다");
        let da2 = pron2
            .syllables
            .iter()
            .find(|s| s.word_idx == 2)
            .expect("대 in the one-sentence version");
        assert_eq!(da2.cvc, vec![6, 4, 1]);
    }

    #[test]
    fn stage_pron_concatenates_sentences_in_order() {
        let (words, pron, _t) = run_stages("좋은 날\n대 앞에 나무가 있다");
        let idxs: Vec<usize> = pron.syllables.iter().map(|s| s.word_idx).collect();
        assert!(idxs.windows(2).all(|w| w[0] <= w[1]),);
        assert_eq!(pron.word_morphs.len(), words.len(),);
        let mut flat: Vec<u8> = Vec::new();
        for s in &pron.syllables {
            flat.extend_from_slice(&s.cvc);
        }
        assert_eq!(pron.phoneme_codes, flat,);
    }

    #[test]
    fn four_line_yeon_ave_jong_matches_oracle_2910() {
        let (words, pron, targets) = run_stages(ANTHEM_4LINE);
        assert_eq!(words.len(), 15);
        assert_eq!(pron.syllables.len(), 36);
        assert_eq!(targets.len(), 36);
        let yeon = &targets[30];
        assert_eq!(yeon.ave_length[2], 2910);
        assert_eq!(yeon.ave_length[0], 590);
        assert!(
            (i32::from(yeon.ave_length[1]) - 1530).abs() <= 1,
            "sentence 4 년 jung = {} (expected 1530±1)",
            yeon.ave_length[1]
        );
        assert!(
            (180.0..195.0).contains(&yeon.f0[0]),
            "sentence 4 년 f0[0] = {} (expected near 186, old value 158)",
            yeon.f0[0]
        );
        assert!(
            (170.0..185.0).contains(&yeon.f0[8]),
            "sentence 4 년 f0[8] = {} (expected near 182)",
            yeon.f0[8]
        );
    }

    #[test]
    fn four_line_rest_flags_match_oracle() {
        let (words, pron, mut targets) = run_stages(ANTHEM_4LINE);
        apply_sentence_end_boundaries(&pron, &words, &mut targets);
        let flags = synth_rest_flags(&pron, &targets);
        assert_eq!(
            flags,
            vec![
                0x63, 0x61, 0x63, 0x60, 0x62, 0x63, 0x60, 0x62, 0x63, 0x63, 0x63, 0x60, 0x63, 0x63,
                0x60,
            ],
        );
        let bnds = word_end_boundaries(&pron, &targets);
        for &i in &[3usize, 6, 11, 14] {
            assert_eq!(bnds[i], 0x15, "sentence-final word {i} boundary is 0x15");
        }
        assert_eq!(bnds[0], 0x0a);
        assert_eq!(bnds[1], 0x14);
    }

    #[test]
    fn anthem_one_line_no_regression() {
        let (_words, pron, targets) = run_stages(ANTHEM_1LINE);
        assert_eq!(targets.len(), 36);
        let ctx =
            ktts_prosody::load_prosody_dicts(&test_data_dir().join("KSpeechDic").join("woman"))
                .expect("prosody dictionary");
        let direct = ktts_prosody::prosody(&ctx, &conv_pron_to_prosody(&pron)).expect("prosody");
        let direct: Vec<SyllableTarget> = direct.into_iter().map(conv_target_to_my).collect();
        assert_eq!(targets, direct,);
        assert_eq!(targets[30].ave_length, [590, 1530, 2244]);
        let bnds = word_end_boundaries(&pron, &targets);
        assert_eq!(bnds[14], 0x15);
        for (i, &b) in bnds.iter().enumerate().take(14) {
            assert_ne!(
                b, 0x15,
                "one-line version: middle word {i} must not have a sentence-final boundary"
            );
        }
    }
}
