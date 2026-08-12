pub mod dicts;
pub mod digits;
pub mod engine;
pub mod english;
pub use ktts_kma::eng_tables;
pub mod kma_code;
pub mod kma_types;
pub mod symbols;
pub mod tables;

pub use dicts::{PronContext, PronError, PronResult};
pub use kma_types::{Morph, WordAnal};

pub type DataMap = ktts_dict::common::DataMap;

pub use ktts_dict;

use std::sync::Mutex;

static RULE_DICT: Mutex<Option<ktts_dict::pronrule::PronRuleDict>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronSyllable {
    pub cvc: Vec<u8>,
    pub word_idx: usize,
    pub is_word_start: bool,
    pub pos: [u8; 2],
    pub morph_idx: u8,
    pub morph_pos: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronText {
    pub syllables: Vec<PronSyllable>,
    pub phoneme_codes: Vec<u8>,
    pub word_morphs: Vec<WordMorphInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WordMorphInfo {
    pub pos: Vec<u8>,
    pub first_chars: Vec<u16>,
    pub surfaces: Vec<Vec<u16>>,
    pub source: Vec<u16>,
    pub b_word_sen: bool,
}

/// Loads the pronunciation dictionaries from a directory.
///
/// # Errors
///
/// Returns an error if the dictionary data is missing or malformed.
pub fn load_pron_dicts(klang_dic: &std::path::Path) -> PronResult<PronContext> {
    let mut files: DataMap = std::collections::HashMap::new();
    for rel in dicts::PRON_DICT_FILE_RELS {
        if let Ok(data) = std::fs::read(klang_dic.join(rel)) {
            files.insert(format!("KLangDic/{rel}"), data);
        }
    }
    load_pron_dicts_bytes(&files)
}

/// Loads the pronunciation dictionaries from a data map.
///
/// # Errors
///
/// Returns an error if the dictionary data is missing or malformed.
///
/// # Panics
///
/// Panics if the dictionary data is inconsistent.
pub fn load_pron_dicts_bytes(files: &DataMap) -> PronResult<PronContext> {
    let ctx = PronContext::load_bytes(files)?;
    if let Some(r) = &ctx.pronrule {
        *RULE_DICT.lock().expect("RULE_DICT lock poisoned") = Some(r.clone());
    }
    Ok(ctx)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::too_many_lines,
    reason = "C port: index/math casts with wrap semantics; faithful C port of a large function"
)]
/// Computes the pronunciation of analyzed words.
///
/// # Errors
///
/// Returns an error if the dictionary data is missing or malformed.
pub fn pronounce(ctx: &PronContext, words: &[WordAnal]) -> PronResult<PronText> {
    let mut out = PronText {
        syllables: Vec::new(),
        phoneme_codes: Vec::new(),
        word_morphs: Vec::with_capacity(words.len()),
    };

    let unit_words: Vec<symbols::UnitWord> = words.iter().map(symbols::unit_word).collect();
    let mut unit_reads: Vec<Option<String>> = vec![None; words.len()];
    for i in 0..words.len() {
        if let Some((j, r)) = symbols::process_unit(ctx, &unit_words, i) {
            unit_reads[j] = Some(r);
        }
    }

    let mut prev: Option<engine::WordCvc> = None;
    for (wi, word) in words.iter().enumerate() {
        let mut wc = process_word(ctx, word, wi, unit_reads[wi].clone());
        if let Some(f) = prev.as_mut() {
            engine::pronun_between_word(f, &mut wc);
        }
        engine::merge_solo_jong(&mut wc);
        let mut starts = wc.syllable_starts();
        starts.sort_unstable();
        let drop_last_morph = if starts.len() > 1
            && let Some(&last) = starts.last()
            && wc.is_x(last)
            && matches!(wc.cvc[last], 0x2C | 0x2E | 0x21 | 0x3F)
        {
            starts.pop();
            wc.cvc[last] != 0x2C
        } else {
            false
        };
        let morphs_view: Vec<Morph> = if drop_last_morph {
            word.morphs[..word.morphs.len().saturating_sub(1)].to_vec()
        } else {
            word.morphs.clone()
        };
        let word_morphs_src: &[Morph] = &morphs_view;
        let surf_lens: Vec<usize> = word_morphs_src
            .iter()
            .map(|m| m.surface_len as usize)
            .collect();
        let mut syl_morphs: Vec<(u8, u8)> = Vec::with_capacity(starts.len());
        {
            let mut mi = 0usize;
            let mut mp = 0usize;
            let mut remaining = surf_lens.first().copied().unwrap_or(1);
            for _ in &starts {
                while remaining == 0 && mi + 1 < surf_lens.len() {
                    mi += 1;
                    remaining = surf_lens[mi];
                    mp = 0;
                }
                syl_morphs.push((mi as u8, mp as u8));
                mp += 1;
                remaining = remaining.saturating_sub(1);
            }
        }
        for (si, &pos) in starts.iter().enumerate() {
            let (morph_idx, morph_pos) = syl_morphs[si];
            if wc.is_h(pos) {
                let syl = wc.cvc[pos..pos + 3].to_vec();
                out.phoneme_codes.extend_from_slice(&syl);
                let tag = wc.tag_at(pos);
                let wan = kma_code::conv_cvc_to_uni_wan(&syl);
                out.syllables.push(PronSyllable {
                    cvc: syl,
                    word_idx: wi,
                    is_word_start: si == 0,
                    pos: [tag, 0],
                    morph_idx,
                    morph_pos,
                });
                let _ = wan;
            } else {
                out.phoneme_codes.push(wc.cvc[pos]);
                let tag = wc.tag_at(pos);
                out.syllables.push(PronSyllable {
                    cvc: vec![wc.cvc[pos]],
                    word_idx: wi,
                    is_word_start: si == 0,
                    pos: [tag, 0],
                    morph_idx,
                    morph_pos,
                });
            }
        }
        {
            let mm_pos: Vec<u8> = word_morphs_src.iter().map(|m| m.pos[0]).collect();
            let mut first_chars = vec![0u16; word_morphs_src.len()];
            for &pos in &starts {
                let m = wc.mpos_at(pos) as usize;
                if m < first_chars.len() && first_chars[m] == 0 {
                    first_chars[m] = if wc.is_h(pos) {
                        kma_code::conv_cvc_to_uni_wan(&wc.cvc[pos..pos + 3])
                    } else {
                        u16::from(wc.cvc[pos])
                    };
                }
            }
            for (m, mm) in word_morphs_src.iter().enumerate() {
                if first_chars[m] != 0 {
                    continue;
                }
                let cvc = &mm.cvc;
                if cvc.is_empty() {
                    continue;
                }
                first_chars[m] = if cvc.len() >= 3 {
                    kma_code::conv_cvc_to_uni_wan(&cvc[0..3])
                } else {
                    u16::from(cvc[0])
                };
            }
            let surfaces: Vec<Vec<u16>> = word_morphs_src
                .iter()
                .map(|m| {
                    let mut s: Vec<u16> = Vec::new();
                    if WordAnal::is_symbol_morph(m) {
                        if m.cvc.len() >= 3 && m.cvc.len() % 3 == 0 {
                            let mut i = 0;
                            while i + 2 < m.cvc.len() {
                                s.push(kma_code::conv_cvc_to_uni_wan(&m.cvc[i..i + 3]));
                                i += 3;
                            }
                        } else {
                            s.extend(m.cvc.iter().map(|&b| u16::from(b)));
                        }
                    } else {
                        let mut i = 0;
                        while i + 2 < m.cvc.len() {
                            s.push(kma_code::conv_cvc_to_uni_wan(&m.cvc[i..i + 3]));
                            i += 3;
                        }
                    }
                    s
                })
                .collect();
            out.word_morphs.push(WordMorphInfo {
                pos: mm_pos,
                first_chars,
                surfaces,
                source: word.source.clone(),
                b_word_sen: word.b_word_sen,
            });
        }
        prev = Some(wc);
    }
    Ok(out)
}

#[allow(clippy::explicit_counter_loop)]
#[allow(clippy::needless_range_loop)]
#[expect(
    clippy::cast_possible_truncation,
    clippy::too_many_lines,
    reason = "C port: index/math casts with wrap semantics; faithful C port of a large function"
)]
fn process_word(
    ctx: &PronContext,
    word: &WordAnal,
    _wi: usize,
    unit_read: Option<String>,
) -> engine::WordCvc {
    let mut pyogi_override: Option<String> = unit_read;
    let mut has_digit_read = false;

    let mut chars: Vec<u16> = Vec::new();
    let mut all_korean = true;
    let mut all_symbol = true;
    let mut digit_count = 0;
    let mut alpha_count = 0;
    let mut has_read_cvc = false;

    for m in &word.morphs {
        let is_sym = WordAnal::is_symbol_morph(m);
        if !m.cvc.is_empty() {
            has_read_cvc = true;
        }
        if is_sym {
            all_korean = false;
            if m.cvc.len() >= 3 && m.cvc.len() % 3 == 0 {
                has_digit_read = true;
                continue;
            }
            for &b in &m.cvc {
                chars.push(u16::from(b));
                if b.is_ascii_digit() {
                    digit_count += 1;
                } else if b.is_ascii_alphabetic() {
                    alpha_count += 1;
                }
            }
        } else {
            all_symbol = false;
            let mut i = 0;
            let cvc = &m.cvc;
            while i + 2 < cvc.len() {
                let w = kma_code::conv_cvc_to_uni_wan(&cvc[i..i + 3]);
                if kma_code::is_uni_wansong(w) || kma_code::is_uni_korean_jamo(w) {
                    chars.push(w);
                } else {
                    all_korean = false;
                    chars.push(u16::from(cvc[i]));
                }
                i += 3;
            }
        }
    }
    if chars.is_empty() && !word.morphs.is_empty() {
        let cvc = &word.cvc();
        let mut i = 0;
        while i + 2 < cvc.len() {
            let w = kma_code::conv_cvc_to_uni_wan(&cvc[i..i + 3]);
            chars.push(w);
            i += 3;
        }
        all_korean = chars
            .iter()
            .all(|&c| kma_code::is_uni_wansong(c) || kma_code::is_uni_korean_jamo(c));
    }
    if digit_count == 0 && alpha_count == 0 {
        for &b in &word.cvc() {
            if b.is_ascii_digit() {
                digit_count += 1;
            } else if b.is_ascii_alphabetic() {
                alpha_count += 1;
            }
        }
    }

    let word_chars: Vec<u16> = chars.clone();
    if pyogi_override.is_none() && !all_korean && !has_digit_read {
        if alpha_count > 0 && digit_count == 0 && all_symbol {
            let ascii: Vec<u8> = chars.iter().map(|&c| c as u8).collect();
            if let Some(p) = english::english_word_to_pyogi(ctx, &ascii) {
                pyogi_override = Some(p);
            }
        } else if digit_count > 0 || alpha_count > 0 {
            let mut s = String::new();
            let mut raw_runs: Vec<Vec<u8>> = Vec::new();
            let wc_bytes = word.cvc();
            let mut i = 0usize;
            while i < wc_bytes.len() {
                let b = wc_bytes[i];
                if b > 29 {
                    if b.is_ascii_alphanumeric() {
                        let mut run = Vec::new();
                        while i < wc_bytes.len()
                            && wc_bytes[i] > 29
                            && wc_bytes[i].is_ascii_alphanumeric()
                        {
                            run.push(wc_bytes[i]);
                            i += 1;
                        }
                        raw_runs.push(run);
                    } else {
                        raw_runs.push(vec![b]);
                        i += 1;
                    }
                } else {
                    i += 3;
                }
            }
            let telephone = raw_runs.iter().any(|r| r.len() == 1 && r[0] == b'-')
                && raw_runs
                    .iter()
                    .filter(|r| r.iter().all(u8::is_ascii_digit))
                    .count()
                    >= 2;
            let mut run_idx = 0usize;
            for m in &word.morphs {
                let is_sym = WordAnal::is_symbol_morph(m);
                if !is_sym && !m.cvc.is_empty() {
                    s.push_str(&kma_code::conv_cvc_to_pyogi(&m.cvc));
                } else if is_sym && !m.cvc.is_empty() {
                    s.push_str(&resolve_raw_run(ctx, &m.cvc, telephone));
                    run_idx += 1;
                } else if run_idx < raw_runs.len() {
                    s.push_str(&resolve_raw_run(ctx, &raw_runs[run_idx], telephone));
                    run_idx += 1;
                }
            }
            while run_idx < raw_runs.len() {
                s.push_str(&resolve_raw_run(ctx, &raw_runs[run_idx], telephone));
                run_idx += 1;
            }
            if !s.is_empty() {
                pyogi_override = Some(s);
            }
        } else if !has_read_cvc
            || word
                .morphs
                .first()
                .is_some_and(|m| m.pos[0] == b'R' || m.pos[0] == b'S')
        {
            let s = if symbols::is_remove_symbol(&word_chars) {
                " ".to_string()
            } else {
                symbols::symbol_word_pyogi(ctx, &word_chars)
            };
            if !s.is_empty() {
                pyogi_override = Some(s);
            }
        }
    }

    let mut wc = engine::WordCvc::default();
    if let Some(p) = &pyogi_override {
        if p == " " {
            wc.cvc.push(b' ');
            wc.ty.push(b'X');
            wc.tag.push(b'k');
            wc.mpos.push(0);
        } else {
            let first_tag = word.morphs.first().map_or(b'0', |m| m.pos[0]);
            let cvc = kma_code::conv_pyogi_to_cvc(p.as_bytes());
            let mut i = 0;
            while i + 2 < cvc.len() {
                wc.cvc.extend_from_slice(&cvc[i..i + 3]);
                wc.ty.extend_from_slice(b"HHH");
                wc.tag.extend_from_slice(&[first_tag, first_tag, first_tag]);
                wc.mpos.push(0);
                i += 3;
            }
        }
    } else {
        let mut mi = 0u8;
        for m in &word.morphs {
            let is_sym = WordAnal::is_symbol_morph(m);
            let cvc = &m.cvc;
            if is_sym && cvc.len() >= 3 && cvc.len() % 3 == 0 {
                let mut i = 0;
                while i + 2 < cvc.len() {
                    wc.cvc.extend_from_slice(&cvc[i..i + 3]);
                    wc.ty.extend_from_slice(b"HHH");
                    wc.tag.extend_from_slice(&[m.pos[0]; 3]);
                    wc.mpos.extend_from_slice(&[mi; 3]);
                    i += 3;
                }
            } else if is_sym {
                for &b in cvc {
                    wc.cvc.push(b);
                    wc.ty.push(b'X');
                    wc.tag.push(m.pos[0]);
                    wc.mpos.push(mi);
                }
            } else {
                let mut i = 0;
                while i + 2 < cvc.len() {
                    wc.cvc.extend_from_slice(&cvc[i..i + 3]);
                    wc.ty.extend_from_slice(b"HHH");
                    wc.tag.extend_from_slice(&[m.pos[0]; 3]);
                    wc.mpos.extend_from_slice(&[mi; 3]);
                    i += 3;
                }
            }
            mi += 1;
        }
    }

    if wc.cvc.iter().any(|&b| b != 0) {
        engine::pronun_intra_word(&mut wc);
    }

    if wc.cvc.is_empty() {
        let mut pushed = false;
        for m in &word.morphs {
            if WordAnal::is_symbol_morph(m) && !m.cvc.is_empty() {
                for &b in &m.cvc {
                    wc.cvc.push(b);
                    wc.ty.push(b'X');
                    wc.tag.push(m.pos[0]);
                    wc.mpos.push(0);
                }
                pushed = true;
            }
        }
        if !pushed {
            wc.cvc.push(b' ');
            wc.ty.push(b'X');
            wc.tag.push(word.morphs.first().map_or(b'0', |m| m.pos[0]));
            wc.mpos.push(0);
        }
    }
    wc
}
fn resolve_raw_run(ctx: &PronContext, run: &[u8], telephone: bool) -> String {
    if run.iter().all(u8::is_ascii_digit) {
        return if telephone {
            digits::digits_telephone(run)
        } else {
            digits::digits_cardinal(run)
        };
    }
    if run.iter().all(u8::is_ascii_alphabetic)
        && let Some(p) = english::english_word_to_pyogi(ctx, run)
    {
        return p;
    }
    let mut s = String::new();
    for &b in run {
        if let Some(items) = symbols::symbol_char_items(ctx, u16::from(b)) {
            for (p, _tag) in items {
                s.push_str(&p);
            }
            continue;
        }
        if b.is_ascii_alphanumeric() {
            continue;
        }
        s.push(char::from_u32(u32::from(b)).unwrap_or(' '));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kma_types::{Morph, WordAnal};

    fn word_from_cvc(cvc: Vec<u8>) -> WordAnal {
        WordAnal {
            morphs: vec![Morph {
                cvc: cvc.clone(),
                pos: [b'n', 0],
                prob: 0.0,
                surface_len: 1,
            }],
            w_byte_num: 0,
            word_cvc: cvc,
            source: vec![],
            b_word_sen: false,
        }
    }

    fn word(morphs: Vec<Morph>, word_cvc: Vec<u8>) -> WordAnal {
        WordAnal {
            morphs,
            w_byte_num: word_cvc.len(),
            word_cvc,
            source: vec![],
            b_word_sen: false,
        }
    }

    fn kor(cvc: Vec<u8>) -> Morph {
        Morph {
            cvc,
            pos: [b'4', 0],
            prob: 0.0,
            surface_len: 1,
        }
    }

    fn sym(tag: u8, cvc: Vec<u8>) -> Morph {
        Morph {
            cvc,
            pos: [tag, 0],
            prob: 0.0,
            surface_len: 1,
        }
    }

    fn assert_all_words_have_syllables(out: &PronText, n_words: usize) {
        assert!(
            !out.syllables.is_empty(),
            "syllables are empty (word count {n_words})"
        );
        for wi in 0..n_words {
            assert!(
                out.syllables.iter().any(|s| s.word_idx == wi),
                "word {wi} has no syllables (a word with zero syllables exists)"
            );
        }
        let mut last = 0usize;
        for s in &out.syllables {
            assert!(
                s.word_idx >= last,
                "word_idx not in ascending order: {} < {}",
                s.word_idx,
                last
            );
            last = s.word_idx;
        }
    }

    #[test]
    fn acceptance_mixed_digit_korean_words() {
        let ctx = PronContext::empty();
        let gu_sip_o = vec![2, 20, 1, 11, 29, 19, 13, 3, 1];
        let sip = vec![11, 29, 19];
        let sip_chil = vec![11, 29, 19, 16, 29, 9];
        let il = vec![13, 29, 9];
        let nyon = vec![4, 3, 4];
        let wol = vec![13, 21, 9];
        let cha = vec![16, 3, 1];
        let deung = vec![5, 13, 18];
        let words = vec![
            word(vec![sym(b'I', gu_sip_o), kor(nyon)], Vec::new()),
            word(vec![sym(b'I', sip), kor(wol)], Vec::new()),
            word(vec![sym(b'I', sip_chil), kor(cha)], Vec::new()),
            word(vec![sym(b'I', il), kor(deung)], Vec::new()),
        ];
        let out = pronounce(&ctx, &words).expect("pronounce must complete");
        assert_all_words_have_syllables(&out, 4);
        let w0: String = out
            .syllables
            .iter()
            .take_while(|s| s.word_idx == 0)
            .map(|s| kma_code::conv_cvc_to_pyogi(&s.cvc))
            .collect();
        assert!(
            w0.starts_with("gu"),
            "95년 must start with the digit reading 구: {w0}"
        );
        assert!(
            w0.ends_with("naN") || w0.ends_with("naG"),
            "95년 must end with 년: {w0}"
        );
    }

    #[test]
    fn acceptance_mixed_digit_korean_legacy_shape() {
        let ctx = PronContext::empty();
        let je = vec![14, 12, 1];
        let cha = vec![16, 3, 1];
        let mut wc = je.clone();
        wc.extend_from_slice(b"17");
        wc.extend_from_slice(&cha);
        let words = vec![word(vec![kor(je), sym(b'I', Vec::new()), kor(cha)], wc)];
        let out = pronounce(&ctx, &words).expect("pronounce must complete");
        assert_all_words_have_syllables(&out, 1);
        let pyogi: String = out
            .syllables
            .iter()
            .map(|s| kma_code::conv_cvc_to_pyogi(&s.cvc))
            .collect();
        assert!(pyogi.contains("siBciL"), "제17차 → 제…십칠…차: {pyogi}");
    }

    #[test]
    fn acceptance_telephone_number() {
        let ctx = PronContext::empty();
        let jeon_hwa_beon_ho = vec![14, 3, 4, 20, 21, 1, 9, 3, 4, 20, 13, 1];
        let raw = b"010-1234-5678".to_vec();
        let words = vec![
            word_from_cvc(jeon_hwa_beon_ho),
            word(
                vec![
                    sym(b'I', Vec::new()),
                    sym(b'6', Vec::new()),
                    sym(b'I', Vec::new()),
                    sym(b'6', Vec::new()),
                    sym(b'I', Vec::new()),
                ],
                raw,
            ),
        ];
        let out = pronounce(&ctx, &words).expect("pronounce must complete");
        assert_all_words_have_syllables(&out, 2);
        let tel: String = out
            .syllables
            .iter()
            .skip_while(|s| s.word_idx == 0)
            .map(|s| kma_code::conv_cvc_to_pyogi(&s.cvc))
            .collect();
        assert!(tel.contains("yoN"), "010 must start with 영일영: {tel}");
        assert!(tel.contains("iL"), "single-digit reading (일): {tel}");
        assert!(tel.contains("yuGciL"), "5678 must read 오육칠…: {tel}");
        assert_eq!(out.syllables.len(), 4 + 11);
    }

    #[test]
    fn acceptance_english_korean_mixed() {
        let ctx = PronContext::empty();
        let eseo = vec![13, 10, 1, 11, 7, 1];
        let words = vec![
            word(vec![sym(b'K', b"KCC".to_vec()), kor(eseo)], b"KCC".to_vec()),
            word_from_cvc(vec![13, 29, 9, 20, 3, 1, 9, 27, 1, 5, 3, 1]),
        ];
        let out = pronounce(&ctx, &words).expect("pronounce must complete");
        assert_all_words_have_syllables(&out, 2);
    }

    #[test]
    fn acceptance_punctuation_and_long_sentence() {
        let ctx = PronContext::empty();
        let an_nyeong = vec![13, 3, 1, 4, 11, 4, 20, 12, 1, 13, 13, 1];
        let ban_gap = vec![9, 3, 4, 2, 3, 19, 11, 27, 1, 5, 3, 1];
        let words = vec![
            word_from_cvc(an_nyeong),
            word(vec![sym(b'L', vec![b'.'])], vec![b'.']),
            word_from_cvc(ban_gap),
            word(vec![sym(b'L', vec![b'!'])], vec![b'!']),
        ];
        let out = pronounce(&ctx, &words).expect("pronounce must complete");
        assert_all_words_have_syllables(&out, 4);
        assert_eq!(
            out.syllables.iter().find(|s| s.word_idx == 1).unwrap().cvc,
            vec![b'.']
        );
        assert_eq!(
            out.syllables.iter().find(|s| s.word_idx == 3).unwrap().cvc,
            vec![b'!']
        );
    }

    #[test]
    fn acceptance_dates_and_time() {
        let ctx = PronContext::empty();
        let il_gu_gu_o = vec![13, 29, 9, 2, 20, 1, 2, 20, 1, 13, 3, 1];
        let sip = vec![11, 29, 19];
        let sip_chil = vec![11, 29, 19, 16, 29, 9];
        let nyon = vec![4, 3, 4];
        let wol = vec![13, 21, 9];
        let il2 = vec![13, 29, 9];
        let oneul_eun = vec![13, 3, 1, 4, 27, 9, 13, 27, 4];
        let sam = vec![11, 3, 17];
        let si = vec![11, 29, 1];
        let i_sip_o = vec![13, 29, 1, 11, 29, 19, 13, 3, 1];
        let bun = vec![9, 20, 4];
        let ibnida = vec![13, 29, 19, 9, 27, 1, 5, 3, 1];
        let words = vec![
            word(vec![sym(b'I', il_gu_gu_o), kor(nyon)], Vec::new()),
            word(vec![sym(b'I', sip), kor(wol)], Vec::new()),
            word(vec![sym(b'I', sip_chil), kor(il2)], Vec::new()),
            word(vec![kor(oneul_eun), sym(b'I', sam), kor(si)], Vec::new()),
            word(vec![sym(b'I', i_sip_o), kor(bun), kor(ibnida)], Vec::new()),
        ];
        let out = pronounce(&ctx, &words).expect("pronounce must complete");
        assert_all_words_have_syllables(&out, 5);
    }

    #[test]
    fn acceptance_unresolvable_digit_word_gets_placeholder() {
        let ctx = PronContext::empty();
        let words = vec![
            word(vec![sym(b'I', Vec::new())], vec![b'1']),
            word_from_cvc(vec![2, 3, 1]),
        ];
        let out = pronounce(&ctx, &words).expect("pronounce must complete");
        assert_all_words_have_syllables(&out, 2);
    }

    #[test]
    fn pronounce_bnida_does_not_hang() {
        let ctx = PronContext::empty();
        let words = vec![word_from_cvc(vec![1, 1, 19, 4, 29, 1, 5, 3, 1])];
        let out = pronounce(&ctx, &words).expect("pronounce must complete");
        assert_eq!(out.syllables.len(), 3);
        assert_eq!(out.syllables[0].cvc, vec![1, 1, 17],);
        assert_eq!(out.phoneme_codes.len(), 9);
    }

    #[test]
    fn pronounce_ibnida_does_not_hang() {
        let ctx = PronContext::empty();
        let words = vec![word_from_cvc(vec![13, 29, 19, 1, 1, 19, 4, 29, 1, 5, 3, 1])];
        let out = pronounce(&ctx, &words).expect("pronounce must complete");
        assert_eq!(out.syllables.len(), 3);
    }

    #[test]
    fn pronounce_hamnida_does_not_hang() {
        let ctx = PronContext::empty();
        let words = vec![word_from_cvc(vec![20, 3, 1, 1, 1, 19, 4, 29, 1, 5, 3, 1])];
        let out = pronounce(&ctx, &words).expect("pronounce must complete");
        assert_eq!(out.syllables.len(), 3);
        assert_eq!(out.syllables[0].cvc, vec![20, 3, 17]);
        assert_eq!(out.syllables[1].cvc, vec![4, 29, 1], "니");
        assert_eq!(out.syllables[2].cvc, vec![5, 3, 1], "다");
    }

    #[test]
    fn merge_solo_jong_ore_n_becomes_oraen() {
        let ctx = PronContext::empty();
        let words = vec![word(
            vec![
                Morph {
                    cvc: vec![13, 13, 1, 7, 4, 1],
                    pos: [b'C', 0],
                    prob: 0.0,
                    surface_len: 1,
                },
                Morph {
                    cvc: vec![1, 1, 5],
                    pos: [b'_', 0],
                    prob: 0.0,
                    surface_len: 1,
                },
            ],
            Vec::new(),
        )];
        let out = pronounce(&ctx, &words).expect("pronounce must complete");
        assert_eq!(out.syllables.len(), 2);
        assert_eq!(out.syllables[0].cvc, vec![13, 13, 1], "오");
        assert_eq!(out.syllables[1].cvc, vec![7, 4, 5]);
        assert_eq!(out.phoneme_codes, vec![13, 13, 1, 7, 4, 5],);
    }

    #[test]
    fn merge_solo_jong_gadeuk_ha_n_becomes_gadeukan() {
        let ctx = PronContext::empty();
        let words = vec![word(
            vec![
                Morph {
                    cvc: vec![2, 3, 1, 5, 27, 2],
                    pos: [b'2', 0],
                    prob: 0.0,
                    surface_len: 1,
                },
                Morph {
                    cvc: vec![20, 3, 1],
                    pos: [b'D', 0],
                    prob: 0.0,
                    surface_len: 1,
                },
                Morph {
                    cvc: vec![1, 1, 5],
                    pos: [b'_', 0],
                    prob: 0.0,
                    surface_len: 1,
                },
            ],
            Vec::new(),
        )];
        let out = pronounce(&ctx, &words).expect("pronounce must complete");
        assert_eq!(out.syllables.len(), 3);
        assert_eq!(out.syllables[0].cvc, vec![2, 3, 1], "가");
        assert_eq!(out.syllables[1].cvc, vec![5, 27, 1], "드 (평음화)");
        assert_eq!(out.syllables[2].cvc, vec![17, 3, 5],);
        assert_eq!(out.phoneme_codes, vec![2, 3, 1, 5, 27, 1, 17, 3, 5],);
    }

    #[test]
    fn pronounce_program_ibnida_sentence_does_not_hang() {
        let ctx = PronContext::empty();
        let mut cvc = vec![
            19, 27, 1, 7, 13, 1, 2, 27, 1, 7, 3, 17, 13, 29, 19, 1, 1, 19, 4, 29, 1, 5, 3, 1,
        ];
        let kor = word_from_cvc(cvc.clone());
        cvc.clear();
        cvc.push(b'.');
        let dot = WordAnal {
            morphs: vec![Morph {
                cvc,
                pos: [b'L', 0],
                prob: 0.0,
                surface_len: 1,
            }],
            w_byte_num: 2,
            word_cvc: vec![b'.'],
            source: vec![],
            b_word_sen: true,
        };
        let out = pronounce(&ctx, &[kor, dot]).expect("pronounce must complete");
        assert_eq!(out.syllables.len(), 8);
        assert_eq!(out.syllables[7].cvc, vec![b'.']);
    }
}
