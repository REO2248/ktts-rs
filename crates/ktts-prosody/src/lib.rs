pub mod bi;
pub mod context;
pub mod cvc;
pub mod pron_types;
pub mod sletter;

use bi::BiWord;
pub use context::{ProsodyContext, load_prosody_dicts, load_prosody_dicts_bytes};
pub use pron_types::{PronSyllable, PronText, WordMorphs};
use sletter::{Phrase, ProsodyTrees, Sentence, SentenceWord, get_length_and_ave_pitch};

pub type DataMap = ktts_dict::common::DataMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyllableTarget {
    pub dur: f32,
    pub ave_length: [u16; 3],
    pub f0: [f32; 12],
    pub tobi: f32,
    pub boundary: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProsodyError(pub String);

impl std::fmt::Display for ProsodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for ProsodyError {}

pub type ProsodyResult<T> = Result<T, ProsodyError>;

fn group_words(text: &PronText) -> Result<Vec<(usize, Vec<&PronSyllable>)>, ProsodyError> {
    if text.syllables.is_empty() {
        return Ok(Vec::new());
    }
    let mut words: Vec<(usize, Vec<&PronSyllable>)> = Vec::new();
    let mut cur_word = text.syllables[0].word_idx;
    let mut cur: Vec<&PronSyllable> = Vec::new();
    for s in &text.syllables {
        if s.word_idx != cur_word {
            words.push((cur_word, std::mem::take(&mut cur)));
            cur_word = s.word_idx;
        }
        cur.push(s);
    }
    words.push((cur_word, cur));
    for (i, &(w, _)) in words.iter().enumerate() {
        if w != i {
            return Err(ProsodyError(format!(
                "word_idx discontinuous: at position {i} got {w} (expected sequential 0..n-1)"
            )));
        }
    }
    Ok(words)
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
/// Computes prosody targets (duration, pitch, `ToBI`) for a pronunciation.
///
/// # Errors
///
/// Returns an error if a dictionary file is missing or malformed.
pub fn prosody(ctx: &ProsodyContext, text: &PronText) -> ProsodyResult<Vec<SyllableTarget>> {
    if text.syllables.is_empty() {
        return Ok(Vec::new());
    }
    let word_groups = group_words(text)?;

    let mut bi_words: Vec<BiWord> = Vec::with_capacity(word_groups.len());
    for (wi, syls) in &word_groups {
        let surfaces: Vec<u16> = syls.iter().map(|s| cvc::cvc_to_char(s.cvc)).collect();
        let wm = text.word_morphs.get(*wi);
        let (morph_pos, morph_surfaces, w_morph_cnt) = match wm {
            Some(w) if !w.pos.is_empty() && w.pos.len() == w.first_chars.len() => (
                w.pos.clone(),
                if !w.surfaces.is_empty() && w.surfaces.len() == w.pos.len() {
                    w.surfaces.clone()
                } else {
                    w.first_chars
                        .iter()
                        .map(|&c| vec![c])
                        .collect::<Vec<Vec<u16>>>()
                },
                w.pos.len() as u16,
            ),
            _ => {
                let mps: Vec<Vec<u16>> = surfaces.iter().map(|&c| vec![c]).collect();
                let mp: Vec<u8> = syls.iter().map(|s| s.pos).collect();
                (mp, mps, syls.len() as u16)
            }
        };
        let to_word = morph_surfaces
            .last()
            .cloned()
            .unwrap_or_else(|| surfaces.clone());
        let root = morph_surfaces
            .first()
            .cloned()
            .unwrap_or_else(|| surfaces.clone());
        bi_words.push(BiWord {
            surface: surfaces.clone(),
            root,
            to_word,
            morph_pos,
            morph_surfaces,
            word_of_sen: wm.map_or_else(|| surfaces.clone(), |w| w.source.clone()),
            b_word_sen: text.word_sen.get(*wi).copied().unwrap_or(0),
            w_morph_cnt,
            b_break_info: 0,
        });
    }
    let breaks = bi::bi_proc(&mut bi_words, &ctx.birule, &ctx.biprob, ctx.use_birule);

    let mut phrases: Vec<Phrase> = Vec::new();
    let mut cur: Vec<usize> = Vec::new();
    for (wi, &b) in breaks.iter().enumerate() {
        cur.push(wi);
        if b == 0x14 || b == 0x15 || b == 3 {
            phrases.push(Phrase {
                words: std::mem::take(&mut cur),
            });
        }
    }
    if !cur.is_empty() {
        phrases.push(Phrase { words: cur });
    }
    if phrases.is_empty() && !breaks.is_empty() {
        phrases.push(Phrase {
            words: (0..breaks.len()).collect(),
        });
    }

    let mut letter_count = 0usize;
    let mut sent_words: Vec<SentenceWord> = Vec::with_capacity(bi_words.len());
    for (i, bw) in bi_words.iter().enumerate() {
        let cvc: Vec<[u8; 3]> = word_groups[i].1.iter().map(|s| s.cvc).collect();
        let syl_morphs: Vec<(u8, u8)> = word_groups[i]
            .1
            .iter()
            .map(|s| (s.morph_idx, s.morph_pos))
            .collect();
        sent_words.push(SentenceWord {
            word: bw,
            letter_start: letter_count,
            cvc,
            syl_morphs,
        });
        letter_count += bw.len();
    }
    let sent = Sentence {
        words: sent_words,
        phrases,
        letter_count,
    };
    let trees = ProsodyTrees {
        dur: ctx.dur_trees.clone(),
        bound_tobi: ctx.bound_tobi.clone(),
        non_bound_tobi: ctx.non_bound_tobi.clone(),
        pitch_f0: ctx.pitch_f0.clone(),
    };

    let mut targets = get_length_and_ave_pitch(&trees, &sent);

    if !text.word_sen.is_empty() {
        let mut syl = 0usize;
        for (w, words) in word_groups.iter().enumerate() {
            let n = words.1.len();
            if n > 0 {
                syl += n - 1;
                if text.word_sen.get(w) == Some(&b'?') {
                    sletter::question_mark_f0_transform(&mut targets[syl].f0);
                }
                syl += 1;
            }
        }
    }

    let mut out = Vec::with_capacity(targets.len());
    let mut word_idx = 0usize;
    for (gi, t) in targets.iter().enumerate() {
        let mut boundary = 0u8;
        while word_idx + 1 < bi_words.len() && sent.words[word_idx + 1].letter_start <= gi {
            word_idx += 1;
        }
        let w = &bi_words[word_idx];
        let w_start = sent.words[word_idx].letter_start;
        if gi + 1 == w_start + w.len() {
            boundary = breaks[word_idx];
        }
        out.push(SyllableTarget {
            dur: t.dur_ms,
            ave_length: t.ave_length,
            f0: t.f0,
            tobi: t.tobi,
            boundary,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::cast_precision_loss,
        clippy::similar_names,
        reason = "test fixtures: oracle values converted with intentional casts"
    )]
    use super::*;

    fn woman_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(
            std::env::var("KTTSDB_DIR")
                .expect("set KTTSDB_DIR to the dictionary data (kttsdb) directory"),
        )
        .join("KSpeechDic")
        .join("woman")
    }

    fn syl(cvc: [u8; 3], word_idx: usize, pos: u8) -> PronSyllable {
        PronSyllable {
            cvc,
            word_idx,
            is_word_start: false,
            pos,
            morph_idx: 0,
            morph_pos: 0,
        }
    }

    fn hello_text() -> PronText {
        PronText {
            syllables: vec![
                syl([13, 3, 5], 0, b'0'),
                syl([4, 11, 23], 0, b'0'),
                syl([13, 3, 1], 1, b'0'),
                syl([13, 10, 1], 1, b'0'),
                syl([13, 19, 1], 2, b'g'),
            ],
            phoneme_codes: vec![],
            word_morphs: vec![],
            word_sen: vec![],
        }
    }

    #[test]
    fn prosody_end_to_end() {
        let ctx = load_prosody_dicts(&woman_dir()).expect("dictionary load failed");
        let text = hello_text();
        let out = prosody(&ctx, &text).expect("prosody prediction failed");
        assert_eq!(out.len(), 5);
        for (i, t) in out.iter().enumerate() {
            assert!(t.dur > 0.0, "dur[{}] > 0: {}", i, t.dur);
            assert!(t.dur < 2000.0, "dur[{}] too large: {}", i, t.dur);
            let sum: u32 = t.ave_length.iter().map(|&v| u32::from(v)).sum();
            assert!(
                (sum as f32 / 16.0 - t.dur).abs() < 1.0,
                "ave_length[{}] inconsistent with dur: {:?} vs {}",
                i,
                t.ave_length,
                t.dur
            );
            for &f in &t.f0 {
                assert!(f > 80.0 && f < 300.0, "f0[{i}] out of range: {f}");
            }
            assert!(!t.tobi.is_nan(), "tobi[{i}] NaN");
        }
        assert_eq!(out[0].ave_length[0], 0, "안 (VC): no cho");
        assert!(out[0].ave_length[1] > 0, "안 (VC): jung long");
        assert!(out[0].ave_length[2] > 0, "안 (VC): jong (ㄴ) long");
        assert!(out[1].ave_length[0] > 0, "녕 (CVC): cho long");
        assert!(out[1].ave_length[1] > 0, "녕 (CVC): jung long");
        assert!(out[1].ave_length[2] > 0, "녕 (CVC): jong (ㅇ) long");
        for i in [2usize, 3, 4] {
            assert_eq!(out[i].ave_length[0], 0, "[{i}] (V): no cho");
            assert!(out[i].ave_length[1] > 0, "[{i}] (V): jung long");
            assert_eq!(out[i].ave_length[2], 0, "[{i}] (V): no jong");
        }
        assert_eq!(out[4].boundary, 0x15);
        assert_eq!(out[0].boundary, 0x00, "안 (internal)");
        assert_eq!(out[2].boundary, 0x00, "하 (internal)");
        assert!(
            out[1].boundary == 0x0a || out[1].boundary == 0x0b || out[1].boundary == 0x14,
            "boundary of 안녕: {:#x}",
            out[1].boundary
        );
        assert!(
            out[3].boundary == 0x0a || out[3].boundary == 0x0b || out[3].boundary == 0x14,
            "boundary of 하세: {:#x}",
            out[3].boundary
        );
    }

    #[test]
    fn prosody_empty_text() {
        let ctx = load_prosody_dicts(&woman_dir()).expect("dictionary load failed");
        let out = prosody(&ctx, &PronText::default()).expect("empty input");
        assert!(out.is_empty());
    }

    #[test]
    fn prosody_single_word() {
        let ctx = load_prosody_dicts(&woman_dir()).expect("dictionary load failed");
        let text = PronText {
            syllables: vec![syl([2, 3, 1], 0, b'0')],
            phoneme_codes: vec![],
            word_morphs: vec![],
            word_sen: vec![],
        };
        let out = prosody(&ctx, &text).expect("prosody prediction failed");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].boundary, 0x15, "single word → end-of-sentence B4");
    }

    #[test]
    fn prosody_with_birule() {
        let ctx = load_prosody_dicts(&woman_dir())
            .expect("dictionary load failed")
            .with_birule(true);
        let text = hello_text();
        let out = prosody(&ctx, &text).expect("prosody prediction failed");
        assert_eq!(out.len(), 5);
        assert_eq!(out[4].boundary, 0x15);
    }

    #[test]
    fn prosody_stress_random() {
        let ctx = load_prosody_dicts(&woman_dir()).expect("dictionary load failed");
        let mut seed: u64 = 0x1234_5678;
        let mut rnd = move || {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (seed >> 33) as usize
        };
        let pos_chars: &[u8] = b"0123456789@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklm";
        let jung_codes: &[u8] = &[
            3, 4, 5, 6, 7, 10, 11, 12, 13, 14, 15, 18, 19, 20, 21, 22, 23, 26, 27, 28, 29,
        ];
        let cho_codes: &[u8] = &[2, 4, 5, 7, 8, 9, 11, 12, 13, 14, 16, 17, 18, 19, 20];
        let jong_codes: &[u8] = &[1, 2, 5, 7, 8, 9, 17, 19, 21, 22, 23];
        for trial in 0..40 {
            let n_words = 1 + rnd() % 8;
            let mut syllables = Vec::new();
            let mut word_idx = 0usize;
            for w in 0..n_words {
                let n_syl = 1 + rnd() % 4;
                for _ in 0..n_syl {
                    let cho = cho_codes[rnd() % cho_codes.len()];
                    let jung = jung_codes[rnd() % jung_codes.len()];
                    let jong = jong_codes[rnd() % jong_codes.len()];
                    syllables.push(PronSyllable {
                        cvc: [cho, jung, jong],
                        word_idx: w,
                        is_word_start: false,
                        pos: pos_chars[rnd() % pos_chars.len()],
                        morph_idx: 0,
                        morph_pos: 0,
                    });
                }
                word_idx = w + 1;
            }
            let _ = word_idx;
            let text = PronText {
                syllables,
                phoneme_codes: vec![],
                word_morphs: vec![],
                word_sen: vec![],
            };
            let out = prosody(&ctx, &text).expect("prosody prediction must not panic");
            assert_eq!(
                out.len(),
                text.syllables.len(),
                "trial {trial}: output length"
            );
            for (i, t) in out.iter().enumerate() {
                assert!(t.dur >= 0.0 && t.dur < 5000.0, "trial {trial} dur[{i}]");
                let sum: u32 = t.ave_length.iter().map(|&v| u32::from(v)).sum();
                assert!(
                    (sum as f32 / 16.0 - t.dur).abs() < 1.0,
                    "trial {trial}: ave_length[{i}] inconsistent with dur"
                );
                for &v in &t.ave_length {
                    assert!(v <= 32767, "trial {trial}: ave_length[{i}] too large: {v}");
                }
                for &f in &t.f0 {
                    assert!(
                        f > 50.0 && f < 400.0 || f == 0.0,
                        "trial {trial} f0[{i}] = {f}"
                    );
                }
                assert!(
                    t.boundary == 0x00
                        || t.boundary == 0x0a
                        || t.boundary == 0x0b
                        || t.boundary == 0x14
                        || t.boundary == 0x15,
                    "trial {trial} boundary[{i}] = {:#x}",
                    t.boundary
                );
            }
            assert_eq!(
                out.last().unwrap().boundary,
                0x15,
                "trial {trial}: end-of-sentence B4"
            );
        }
    }

    #[test]
    fn prosody_deterministic() {
        let ctx = load_prosody_dicts(&woman_dir()).expect("dictionary load failed");
        let text = hello_text();
        let a = prosody(&ctx, &text).expect("first run");
        let b = prosody(&ctx, &text).expect("second run");
        assert_eq!(a, b);
    }

    #[test]
    fn load_bytes_equiv_path_load() {
        let dir = woman_dir();
        let c1 = load_prosody_dicts(&dir).expect("path load");
        let mut files: DataMap = std::collections::HashMap::new();
        let rels = [
            "Break/BIRule.bin",
            "Break/BIProb_hash.bin",
            "Break/BIProb_hash.dic",
            "tone/DURATION/pahyul.tree2.bin",
            "tone/DURATION/pachal.tree2.bin",
            "tone/DURATION/machal.tree2.bin",
            "tone/DURATION/nasal.tree2.bin",
            "tone/DURATION/glide.tree2.bin",
            "tone/DURATION/mono.tree2.bin",
            "tone/DURATION/di.tree2.bin",
            "tone/boundary_tobi.tree2.bin",
            "tone/non_boundary_tobi.tree2.bin",
            "tone/Pitch_f0.tree2.bin",
        ];
        for rel in rels {
            if let Ok(data) = std::fs::read(dir.join(rel)) {
                files.insert(format!("KSpeechDic/woman/{rel}"), data);
            }
        }
        let c2 = load_prosody_dicts_bytes(&files, "woman").expect("bytes load");

        assert_eq!(c1.birule.w_rule_num, c2.birule.w_rule_num);
        assert_eq!(c1.birule.w_attrib_num, c2.birule.w_attrib_num);
        assert_eq!(c1.biprob.key_count, c2.biprob.key_count);
        for i in 0..7 {
            assert_eq!(c1.dur_trees[i].nodes.len(), c2.dur_trees[i].nodes.len());
        }
        assert_eq!(c1.bound_tobi.nodes.len(), c2.bound_tobi.nodes.len());
        assert_eq!(c1.non_bound_tobi.nodes.len(), c2.non_bound_tobi.nodes.len());
        assert_eq!(c1.pitch_f0.nodes.len(), c2.pitch_f0.nodes.len());
        let text = hello_text();
        let o1 = prosody(&c1, &text).expect("path prosody");
        let o2 = prosody(&c2, &text).expect("bytes prosody");
        assert_eq!(o1, o2);
    }
}
