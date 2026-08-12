use crate::types::{PronText, SyllableTarget};

pub const REST_SENT_END: u8 = 0x60;
pub const REST_SPACE: u8 = 0x61;
pub const REST_WORD: u8 = 0x62;
pub const REST_STRONG: u8 = 0x63;

const JONG_CONNECTED_RAW: [u8; 3] = [2, 8, 19];
const JONG_CONNECTED: [u8; 3] = [0x42, 0x48, 0x53];

#[inline]
fn is_connected_jong(code: u8) -> bool {
    JONG_CONNECTED.contains(&code) || JONG_CONNECTED_RAW.contains(&code)
}

#[derive(Debug, Clone, Copy)]
pub struct Letter {
    pub sch_cho: [u8; 7],
    pub sch_jung: [u8; 7],
    pub sch_jong: [u8; 7],
    pub f_cho: i8,
    pub f_jung: i8,
    pub f_jong: i8,
    pub ave_length: [u16; 3],
    pub dict_type: i8,
    pub ave_pitch: [u16; 12],
    pub cvc: [u8; 3],
    pub word_idx: usize,
    pub is_phrase_head: bool,
}

#[derive(Debug, Clone)]
pub struct Word {
    pub letters: std::ops::Range<usize>,
    pub rest_flag: u8,
    pub word_sen: u8,
}

#[derive(Debug, Clone)]
pub struct Phrase {
    pub letters: Vec<Letter>,
    pub words: Vec<Word>,
}

impl Phrase {
    #[must_use]
    pub const fn letter_num(&self) -> usize {
        self.letters.len()
    }
    #[must_use]
    pub const fn word_num(&self) -> usize {
        self.words.len()
    }
}

#[inline]
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
pub fn round(x: f32) -> i32 {
    (x + 0.5) as i32
}

#[inline]
#[must_use]
pub const fn get_ks_byte(raw: u8, idx: usize) -> u8 {
    let c = raw;
    if c > 0x20 {
        return c;
    }
    match idx {
        1 => {
            if c == 0x02 {
                0
            } else if c == 0x0f {
                b'2'
            } else {
                c.wrapping_add(0x20)
            }
        }
        2 => {
            if c == 0x01 {
                0
            } else {
                c.wrapping_add(0x40)
            }
        }
        _ => {
            if c == 0x0d || c == 0x01 {
                0
            } else {
                c
            }
        }
    }
}

#[allow(clippy::needless_range_loop)]
fn cvc_codes(cvc: &str) -> [u8; 3] {
    let bytes: Vec<u8> = cvc.bytes().collect();
    let mut out = [0u8; 3];
    if bytes.is_empty() {
        return out;
    }
    if bytes.len() == 1 {
        out = [bytes[0], 0x20, 0x40];
    } else {
        for (i, &b) in bytes.iter().take(3).enumerate() {
            out[i] = b;
        }
        for i in 0..3 {
            out[i] = get_ks_byte(out[i], i);
        }
    }
    out
}

#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
fn johab_flags(cvc: [u8; 3]) -> (i8, i8, i8) {
    let f_cho = if cvc[0] == 0 || cvc[0] == 1 || cvc[0] == 0x0d {
        -1
    } else {
        1
    };
    let f_jung = 1;
    let f_jong = if cvc[2] == 0 || cvc[2] == 1 {
        -1
    } else if is_connected_jong(cvc[2]) {
        -2
    } else {
        1
    };
    (f_cho, f_jung, f_jong)
}

#[inline]
const fn ave_length_from_target(t: &SyllableTarget) -> [u16; 3] {
    t.ave_length
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn f0_to_pitch(f0: f32) -> u16 {
    if f0 <= 0.0 {
        return 0;
    }
    ((f64::from(crate::consts::SAMPLE_RATE) / f64::from(f0) + 0.5).trunc()).max(0.0) as u16
}

#[allow(clippy::needless_range_loop)]
#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
/// Builds a synthesis phrase from pronunciation and targets.
///
/// # Errors
///
/// Returns an error if the input is invalid.
pub fn build_phrase(text: &PronText, targets: &[SyllableTarget]) -> Result<Phrase, String> {
    if text.syllables.is_empty() {
        return Err("PronText has no syllables".into());
    }
    if targets.len() != text.syllables.len() {
        return Err(format!(
            "syllable count mismatch: syllables={} targets={}",
            text.syllables.len(),
            targets.len()
        ));
    }
    let mut word_of: Vec<usize> = Vec::with_capacity(text.syllables.len());
    let mut words: Vec<Word> = Vec::new();
    let mut cur_word = 0usize;
    for (i, s) in text.syllables.iter().enumerate() {
        if i > 0 && s.word_idx != text.syllables[i - 1].word_idx {
            cur_word += 1;
        }
        word_of.push(cur_word);
    }
    let word_num = cur_word + 1;
    let mut letter_start = vec![0usize; word_num];
    for i in 0..word_num {
        letter_start[i] = word_of.iter().position(|&w| w == i).unwrap_or(0);
    }
    for w in 0..word_num {
        let start = letter_start[w];
        let end = word_of.iter().filter(|&&x| x == w).count() + start;
        let b = targets[end - 1].boundary;
        let rest_flag = if w == word_num - 1 || b == 0x15 {
            REST_SENT_END
        } else {
            match b {
                0x0a => REST_STRONG,
                0x0b => REST_WORD,
                _ => REST_SPACE,
            }
        };
        words.push(Word {
            letters: start..end,
            rest_flag,
            word_sen: text.word_sen.get(w).copied().unwrap_or(0),
        });
    }

    let mut word_is_phrase_head = vec![false; word_num];
    word_is_phrase_head[0] = true;
    for w in 1..word_num {
        let prev_rest = words[w - 1].rest_flag;
        word_is_phrase_head[w] = prev_rest == REST_SENT_END || prev_rest == REST_SPACE;
    }

    let mut letters = Vec::with_capacity(text.syllables.len());
    for (i, (s, t)) in text.syllables.iter().zip(targets.iter()).enumerate() {
        let cvc = cvc_codes(&s.cvc);
        let (f_cho, f_jung, f_jong) = johab_flags(cvc);
        let ave_length = ave_length_from_target(t);
        let mut ave_pitch = [0u16; 12];
        for k in 0..12 {
            ave_pitch[k] = f0_to_pitch(t.f0[k]);
        }
        let w = word_of[i];
        let is_phrase_head = word_is_phrase_head[w] && i == words[w].letters.start;
        letters.push(Letter {
            sch_cho: [0; 7],
            sch_jung: [0; 7],
            sch_jong: [0; 7],
            f_cho,
            f_jung,
            f_jong,
            ave_length,
            dict_type: 1,
            ave_pitch,
            cvc,
            word_idx: w,
            is_phrase_head,
        });
    }

    fill_seven_phone(&mut letters, &words);
    Ok(Phrase { letters, words })
}

fn word_string(letters: &[Letter], word: &Word) -> Vec<u8> {
    let mut s = Vec::with_capacity(word.letters.len() * 3);
    for l in &letters[word.letters.clone()] {
        s.push(l.cvc[0]);
        s.push(l.cvc[1]);
        s.push(l.cvc[2]);
    }
    s
}

#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
fn prev_code(phrase_letters: &[Letter], words: &[Word], wi: usize) -> [u8; 8] {
    let mut out = [0x60u8; 8];
    if wi == 0 {
        return out;
    }
    let prev = &words[wi - 1];
    let cur_start_letter = words[wi].letters.start;
    match prev.rest_flag {
        REST_SENT_END => {
            out = [0x60; 8];
        }
        REST_SPACE => {
            out = [0x61; 8];
        }
        _ => {
            let prev_letters = &phrase_letters[prev.letters.clone()];
            let last = prev_letters
                .last()
                .expect("word must have at least one letter")
                .cvc;
            out[4] = last[0];
            out[5] = last[1];
            out[6] = last[2];
            out[7] = if prev.rest_flag == REST_WORD {
                0x62
            } else {
                0x63
            };
            if prev_letters.len() >= 2 {
                let second_last = prev_letters[prev_letters.len() - 2].cvc;
                out[0] = second_last[0];
                out[1] = second_last[1];
                out[2] = second_last[2];
                out[3] = 0xff;
            } else if cur_start_letter == 1 {
                out[0..4].copy_from_slice(&[0x60; 4]);
            } else if wi >= 2 {
                if words[wi - 2].rest_flag == REST_SPACE {
                    out[0..4].copy_from_slice(&[0x61; 4]);
                } else {
                    let prev2_letters = &phrase_letters[words[wi - 2].letters.clone()];
                    let p2 = prev2_letters
                        .last()
                        .expect("word must have at least one letter")
                        .cvc;
                    out[0] = p2[0];
                    out[1] = p2[1];
                    out[2] = p2[2];
                    out[3] = if words[wi - 2].rest_flag == REST_WORD {
                        0x62
                    } else {
                        0x63
                    };
                }
            }
        }
    }
    out
}

#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
fn next_code(phrase_letters: &[Letter], words: &[Word], wi: usize) -> ([u8; 8], [u8; 8]) {
    let mut temp = [0x60u8; 8];
    let mut next = [0x60u8; 8];
    let cur = &words[wi];
    let is_last = wi + 1 >= words.len();
    match cur.rest_flag {
        REST_SENT_END => {
            let fill = if cur.word_sen == b'?' { 0x64 } else { 0x60 };
            temp = [fill; 8];
            next[4] = fill;
            next[7] = fill;
        }
        REST_SPACE => {
            temp = [0x61; 8];
            next[4] = 0x61;
            next[7] = 0x61;
        }
        _ => {
            if is_last {
                temp = [0x60; 8];
                next[4] = 0x60;
                next[7] = 0x60;
            } else {
                let nxt = &words[wi + 1];
                let nxt_letters = &phrase_letters[nxt.letters.clone()];
                let first = nxt_letters
                    .first()
                    .expect("word must have at least one letter")
                    .cvc;
                temp[4] = first[0];
                temp[5] = first[1];
                temp[6] = first[2];
                temp[7] = if cur.rest_flag == REST_WORD {
                    0x62
                } else {
                    0x63
                };
                if nxt_letters.len() < 2 {
                    if nxt.rest_flag == REST_SENT_END {
                        let fill = if cur.word_sen == b'?' { 0x64 } else { 0x60 };
                        temp[0..4].copy_from_slice(&[fill; 4]);
                        next[4] = fill;
                        next[7] = fill;
                    } else if nxt.rest_flag == REST_SPACE {
                        temp[0..4].copy_from_slice(&[0x60; 4]);
                        next[4] = 0x60;
                        next[7] = 0x60;
                    } else if wi + 2 < words.len() {
                        let nn_letters = &phrase_letters[words[wi + 2].letters.clone()];
                        let f = nn_letters
                            .first()
                            .expect("word must have at least one letter")
                            .cvc;
                        temp[0] = f[0];
                        temp[1] = f[1];
                        next[7] = f[2];
                        temp[2] = f[2];
                        if nxt.rest_flag == REST_WORD {
                            temp[3] = 0x62;
                            next[4] = 0x62;
                        } else {
                            temp[3] = 0x63;
                            next[4] = 0x63;
                        }
                    } else {
                        temp[0..4].copy_from_slice(&[0x60; 4]);
                        next[4] = 0x60;
                        next[7] = 0x60;
                    }
                } else {
                    let second = nxt_letters[1].cvc;
                    temp[0] = second[0];
                    temp[1] = second[1];
                    next[7] = second[2];
                    temp[2] = second[2];
                    next[4] = 0xff;
                    temp[3] = 0xff;
                }
            }
        }
    }
    let c8 = temp[7];
    next[0] = temp[7];
    next[5] = temp[0];
    next[1] = temp[4];
    next[6] = temp[1];
    next[2] = temp[5];
    next[3] = temp[6];
    let _ = c8;
    (temp, next)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn fill_letter_contexts(
    word_str: &[u8],
    letter_idx: usize,
    prev: [u8; 8],
    next_temp: [u8; 8],
    next_code: [u8; 8],
    letter: &mut Letter,
) {
    let base = letter_idx * 3;
    for pos in 0..3usize {
        let ctx: &mut [u8; 7] = match pos {
            0 => &mut letter.sch_cho,
            1 => &mut letter.sch_jung,
            _ => &mut letter.sch_jong,
        };
        ctx.fill(0);
        let cur = word_str[base + pos];
        if cur == 0 {
            continue;
        }
        let mut idx = 3usize;
        let mut i = (base + pos) as i64;
        let mut ptr = (base + pos) as i64;
        let mut c = i64::from(cur);
        loop {
            if c > 0 {
                ctx[idx] = c as u8;
                idx = idx.wrapping_sub(1);
            }
            i -= 1;
            if i == -1 {
                break;
            }
            ptr -= 1;
            if idx == usize::MAX {
                break;
            }
            c = i64::from(word_str[ptr as usize]);
        }
        if i == -1 {
            let mut pc = 7i64;
            while idx != usize::MAX && pc >= 0 {
                let v = i64::from(prev[pc as usize]);
                if (v as i8) > 0 {
                    ctx[idx] = v as u8;
                    idx = idx.wrapping_sub(1);
                }
                if pc == 0 {
                    break;
                }
                pc -= 1;
            }
        }
        let mut idx2 = 4usize;
        let mut ii = (base + pos) as i64 + 1;
        let mut c2: i64;
        if (base + pos) as i64 + 1 < word_str.len() as i64 {
            c2 = i64::from(word_str[(base + pos) + 1]);
            while idx2 < 7 {
                if c2 > 0 {
                    ctx[idx2] = c2 as u8;
                    idx2 += 1;
                }
                ii += 1;
                if ii >= word_str.len() as i64 {
                    if idx2 < 7 {
                        break;
                    }
                    break;
                }
                c2 = i64::from(word_str[ii as usize]);
            }
        }
        if idx2 < 7 {
            let mut c3 = i64::from(next_temp[7]);
            let mut k = 0usize;
            loop {
                if (c3 as i8) > 0 {
                    ctx[idx2] = c3 as u8;
                    idx2 += 1;
                }
                if k + 1 == 8 || idx2 > 6 {
                    break;
                }
                k += 1;
                c3 = i64::from(next_code[k]);
            }
        }
    }
}

fn fill_seven_phone(letters: &mut [Letter], words: &[Word]) {
    for (wi, word) in words.iter().enumerate() {
        let word_str = word_string(letters, word);
        let prev = prev_code(letters, words, wi);
        let (temp, next) = next_code(letters, words, wi);
        for (li, letter) in letters[word.letters.clone()].iter_mut().enumerate() {
            fill_letter_contexts(&word_str, li, prev, temp, next, letter);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PronSyllable;

    fn syl(cvc: &str, word: usize, start: bool) -> PronSyllable {
        PronSyllable {
            cvc: cvc.to_string(),
            word_idx: word,
            is_word_start: start,
            pos: 0,
        }
    }

    fn tgt(dur: f32, f0: f32) -> SyllableTarget {
        tgt_ave(dur, f0, [0; 3])
    }

    fn tgt_ave(dur: f32, f0: f32, ave_length: [u16; 3]) -> SyllableTarget {
        SyllableTarget {
            dur,
            ave_length,
            f0: [f0; 12],
            tobi: 0.0,
            boundary: 0,
        }
    }

    fn tgt_bnd(dur: f32, f0: f32, boundary: u8) -> SyllableTarget {
        SyllableTarget {
            dur,
            ave_length: [0; 3],
            f0: [f0; 12],
            tobi: 0.0,
            boundary,
        }
    }

    fn pron_achimeun_bitnara() -> PronText {
        PronText {
            syllables: vec![
                syl("\x0d\x03\x01", 0, true),
                syl("\x10\x1d\x01", 0, false),
                syl("\x08\x1b\x05", 0, false),
                syl("\x09\x1d\x05", 1, true),
                syl("\x04\x03\x01", 1, false),
                syl("\x07\x03\x01", 1, false),
                syl("\x0d\x1d\x01", 2, true),
                syl("\x02\x03\x17", 3, true),
                syl("\x0b\x03\x05", 3, false),
            ],
            phoneme_codes: vec![
                13, 3, 1, 16, 29, 1, 8, 27, 5, 9, 29, 5, 4, 3, 1, 7, 3, 1, 13, 29, 1, 2, 3, 23, 11,
                3, 5,
            ],
            word_sen: vec![],
        }
    }

    #[test]
    fn rest_flags_set_prosody_rest_space() {
        let p = build_phrase(
            &pron_achimeun_bitnara(),
            &[
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x0a),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x14),
                tgt_bnd(100.0, 180.0, 0x0a),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x15),
            ],
        )
        .unwrap();
        let flags: Vec<u8> = p.words.iter().map(|w| w.rest_flag).collect();
        assert_eq!(flags, [REST_STRONG, REST_SPACE, REST_STRONG, REST_SENT_END]);
    }

    #[test]
    fn rest_flags_set_prosody_rest_word() {
        let p = build_phrase(
            &pron_achimeun_bitnara(),
            &[
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x0a),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x0b),
                tgt_bnd(100.0, 180.0, 0x0a),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x15),
            ],
        )
        .unwrap();
        let flags: Vec<u8> = p.words.iter().map(|w| w.rest_flag).collect();
        assert_eq!(flags, [REST_STRONG, REST_WORD, REST_STRONG, REST_SENT_END]);
    }

    #[test]
    fn rest_flags_multi_sentence_mid_sentence_end() {
        let text = PronText {
            syllables: vec![
                syl("\x0d\x03\x01", 0, true),
                syl("\x10\x1d\x01", 0, false),
                syl("\x08\x1b\x05", 0, false),
                syl("\x09\x1d\x05", 1, true),
                syl("\x04\x03\x01", 1, false),
                syl("\x07\x03\x01", 1, false),
                syl("\x0d\x1b\x05", 2, true),
                syl("\x02\x1b\x01", 2, false),
                syl("\x08\x0a\x01", 2, false),
                syl("\x0e\x03\x01", 3, true),
                syl("\x0d\x15\x05", 3, false),
                syl("\x05\x0d\x01", 3, false),
            ],
            phoneme_codes: vec![],

            word_sen: vec![],
        };
        let p = build_phrase(
            &text,
            &[
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x0a),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x15),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x0a),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x15),
            ],
        )
        .unwrap();
        let flags: Vec<u8> = p.words.iter().map(|w| w.rest_flag).collect();
        assert_eq!(
            flags,
            [REST_STRONG, REST_SENT_END, REST_STRONG, REST_SENT_END]
        );
    }

    #[test]
    fn rest_flags_no_boundary_falls_back_to_space() {
        let text = PronText {
            syllables: vec![syl("\x0d\x03\x05", 0, true), syl("\x0d\x0b\x17", 1, true)],
            phoneme_codes: vec![0x0d, 0x03, 0x05, 0x0d, 0x0b, 0x17],
            word_sen: vec![],
        };
        let p = build_phrase(&text, &[tgt(100.0, 200.0), tgt(120.0, 180.0)]).unwrap();
        assert_eq!(p.words[0].rest_flag, REST_SPACE);
        assert_eq!(p.words[1].rest_flag, REST_SENT_END);
    }

    fn pron_annyong() -> PronText {
        PronText {
            syllables: vec![syl("\x0d\x03\x05", 0, true), syl("\x0d\x0b\x17", 0, false)],
            phoneme_codes: vec![0x0d, 0x03, 0x05, 0x0d, 0x0b, 0x17],
            word_sen: vec![],
        }
    }

    #[test]
    fn johab_flags_ok() {
        assert_eq!(johab_flags([0x0e, 0x23, 0x4a]), (1, 1, 1));
        assert_eq!(johab_flags([1, 0x23, 0]), (-1, 1, -1));
        assert_eq!(johab_flags([0x0d, 0x23, 0]), (-1, 1, -1));
        assert_eq!(johab_flags([0, 0x23, 0]), (-1, 1, -1));
        assert_eq!(johab_flags([0x0e, 0x23, 0x42]), (1, 1, -2));
        assert_eq!(johab_flags([0x0e, 0x23, 0x48]), (1, 1, -2));
        assert_eq!(johab_flags([0x0e, 0x23, 0x53]), (1, 1, -2));
        assert_eq!(johab_flags([0x0e, 0x03, 0x02]), (1, 1, -2));
        assert_eq!(johab_flags([0x0e, 0x03, 0x08]), (1, 1, -2));
        assert_eq!(johab_flags([0x0e, 0x03, 0x13]), (1, 1, -2));
        assert_eq!(johab_flags([0x0e, 0x23, 0x49]), (1, 1, 1));
        assert_eq!(johab_flags([0x0e, 0x23, 0]), (1, 1, -1));
        assert_eq!(johab_flags([0x0e, 0x23, 1]), (1, 1, -1));
    }

    #[test]
    fn get_ks_byte_matches_decomp() {
        assert_eq!(get_ks_byte(1, 0), 0);
        assert_eq!(get_ks_byte(0x0d, 0), 0);
        assert_eq!(get_ks_byte(0x11, 0), 0x11);
        assert_eq!(get_ks_byte(0, 0), 0);
        assert_eq!(get_ks_byte(0x02, 1), 0);
        assert_eq!(get_ks_byte(0x0f, 1), 0x32);
        assert_eq!(get_ks_byte(3, 1), 0x23);
        assert_eq!(get_ks_byte(1, 1), 0x21);
        assert_eq!(get_ks_byte(0, 1), 0x20);
        assert_eq!(get_ks_byte(0x01, 2), 0);
        assert_eq!(get_ks_byte(5, 2), 0x45);
        assert_eq!(get_ks_byte(0, 2), 0x40);
        assert_eq!(get_ks_byte(0x60, 0), 0x60);
        assert_eq!(get_ks_byte(0x60, 1), 0x60);
        assert_eq!(get_ks_byte(0x58, 0), 0x58);
    }

    #[test]
    fn two_stage_chain_equivalent_to_getksbyte() {
        fn cli_conv(r: u8, idx: usize) -> u8 {
            match idx {
                1 => {
                    if r == 1 {
                        0x00
                    } else {
                        r.wrapping_add(0x20)
                    }
                }
                2 => {
                    if r == 1 {
                        0x00
                    } else {
                        r.wrapping_add(0x40)
                    }
                }
                _ => r,
            }
        }
        fn ks_byte_legacy(b: u8, idx: usize) -> u8 {
            match idx {
                0 => {
                    if b == 1 || b == 0x0d {
                        0
                    } else {
                        b
                    }
                }
                1 => match b {
                    0x22 => 0,
                    0x2f => 0x32,
                    1 => 0x21,
                    _ => b,
                },
                _ => {
                    if b == 1 {
                        0
                    } else {
                        b
                    }
                }
            }
        }
        for r in 0u8..=0x20 {
            for idx in 0..3 {
                if idx == 1 && r == 1 {
                    continue;
                }
                assert_eq!(
                    ks_byte_legacy(cli_conv(r, idx), idx),
                    get_ks_byte(r, idx),
                    "r=0x{r:02x} idx={idx}"
                );
            }
        }
        assert_eq!(get_ks_byte(1, 1), 0x21);
        assert_eq!(cvc_codes("\x01\x01\x05"), [0, 0x21, 0x45]);
        assert_eq!(cvc_codes("\x02\x01\x01"), [0x02, 0x21, 0]);
        assert_eq!(cvc_codes("\x01\x01\x01"), [0, 0x21, 0]);
        assert_eq!(get_ks_byte(0x21, 1), 0x21);
        assert_ne!(ks_byte_legacy(cli_conv(0x21, 1), 1), 0x21);
    }

    #[test]
    fn cvc_codes_single_byte_forms() {
        assert_eq!(cvc_codes("a"), [0x61, 0x20, 0x40]);
        assert_eq!(cvc_codes(" "), [0x20, 0x20, 0x40]);
        assert_eq!(cvc_codes("\x01\x01\x05"), [0, 0x21, 0x45]);
        assert_eq!(cvc_codes("\x02\x01\x01"), [0x02, 0x21, 0]);
        assert_eq!(cvc_codes("\x01\x01\x01"), [0, 0x21, 0]);
        assert_eq!(cvc_codes("\x0d\x03\x01"), [0, 0x23, 0]);
        assert_eq!(cvc_codes("\x0d\x0f\x01"), [0, 0x32, 0]);
    }

    #[test]
    fn cvc_codes_three_byte_forms() {
        assert_eq!(cvc_codes("\x0d\x03\x05"), [0, 0x23, 0x45]);
        assert_eq!(cvc_codes("\x11\x0e\x17"), [0x11, 0x2e, 0x57]);
        assert_eq!(cvc_codes("\x0d\x0f\x05"), [0, 0x32, 0x45]);
        assert_eq!(cvc_codes("\x0e\x03\x01"), [0x0e, 0x23, 0]);
        assert_eq!(cvc_codes(""), [0, 0, 0]);
    }

    #[test]
    fn lengths_and_pitch() {
        let p = build_phrase(
            &pron_annyong(),
            &[
                tgt_ave(100.0, 200.0, [0, 960, 640]),
                tgt_ave(120.0, 180.0, [480, 960, 480]),
            ],
        )
        .unwrap();
        assert_eq!(p.letter_num(), 2);
        assert_eq!(p.word_num(), 1);
        assert_eq!(p.letters[0].ave_length, [0, 960, 640]);
        assert_eq!(p.letters[1].ave_length, [480, 960, 480]);
        assert_eq!(p.letters[0].ave_pitch[0], 80);
        assert_eq!(p.letters[1].ave_pitch[0], 89);
        let p2 = build_phrase(
            &pron_annyong(),
            &[
                tgt_ave(100.0, 0.0, [0, 960, 640]),
                tgt_ave(120.0, 180.0, [480, 960, 480]),
            ],
        )
        .unwrap();
        assert_eq!(p2.letters[0].ave_pitch[0], 0);
    }

    #[test]
    fn ave_length_is_used_verbatim() {
        let text = PronText {
            syllables: vec![syl("\x0d\x03\x05", 0, true)],
            phoneme_codes: vec![0x0d, 0x03, 0x05],

            word_sen: vec![],
        };
        let p = build_phrase(&text, &[tgt_ave(500.0, 200.0, [0, 100, 200])]).unwrap();
        assert_eq!(p.letters[0].ave_length, [0, 100, 200]);
        assert_eq!(p.letters[0].cvc, [0, 0x23, 0x45]);
    }

    #[test]
    fn seven_phone_contexts() {
        let p = build_phrase(&pron_annyong(), &[tgt(100.0, 200.0), tgt(120.0, 180.0)]).unwrap();
        let l0 = &p.letters[0];
        assert_eq!(l0.sch_cho, [0; 7]);
        assert_eq!(l0.sch_jung, [0x60, 0x60, 0x60, 0x23, 0x45, 0x2b, 0x57]);
        assert_eq!(l0.sch_jong, [0x60, 0x60, 0x23, 0x45, 0x2b, 0x57, 0x60]);
        let l1 = &p.letters[1];
        assert_eq!(l1.sch_cho, [0; 7]);
        assert_eq!(l1.sch_jung, [0x60, 0x23, 0x45, 0x2b, 0x57, 0x60, 0x60]);
        assert_eq!(l1.sch_jong, [0x23, 0x45, 0x2b, 0x57, 0x60, 0x60, 0x60]);
    }

    #[test]
    fn multiword_rest_flags() {
        let text = PronText {
            syllables: vec![syl("\x0d\x03\x05", 0, true), syl("\x0d\x0b\x17", 1, true)],
            phoneme_codes: vec![0x0d, 0x03, 0x05, 0x0d, 0x0b, 0x17],

            word_sen: vec![],
        };
        let p = build_phrase(&text, &[tgt(100.0, 200.0), tgt(120.0, 180.0)]).unwrap();
        assert_eq!(p.word_num(), 2);
        assert_eq!(p.words[0].rest_flag, REST_SPACE);
        assert_eq!(p.words[1].rest_flag, REST_SENT_END);
        let l0 = &p.letters[0];
        assert_eq!(l0.sch_jung, [0x60, 0x60, 0x60, 0x23, 0x45, 0x61, 0x61]);
        assert_eq!(l0.sch_jong, [0x60, 0x60, 0x23, 0x45, 0x61, 0x61, 0x61]);
        let l1 = &p.letters[1];
        assert_eq!(l1.sch_jung, [0x61, 0x61, 0x61, 0x2b, 0x57, 0x60, 0x60]);
        assert_eq!(l1.sch_jong, [0x61, 0x61, 0x2b, 0x57, 0x60, 0x60, 0x60]);
    }

    #[test]
    fn no_zero_in_contexts_of_existing_phones() {
        let cases: &[&str] = &[
            "\x0d\x03\x05",
            "\x11\x0e\x17",
            "\x0d\x0f\x05",
            "\x0e\x03\x01",
            "\x0d\x04\x00",
            "\x01\x01\x05",
            "\x02\x01\x01",
            "\x01\x01\x01",
            "a",
            "\x14\x1d\x13",
        ];
        for cvc in cases {
            let text = PronText {
                syllables: vec![syl(cvc, 0, true)],
                phoneme_codes: cvc.bytes().collect(),

                word_sen: vec![],
            };
            let p = build_phrase(&text, &[tgt(120.0, 180.0)]).unwrap();
            let l = &p.letters[0];
            let ctxs = [&l.sch_cho, &l.sch_jung, &l.sch_jong];
            for (pos, ctx) in ctxs.iter().enumerate() {
                assert_eq!(ctx.len(), 7);
                for &b in *ctx {
                    assert!(
                        b <= 0x63,
                        "{cvc:?} pos {pos}: out-of-range code in context 0x{b:02x}: {ctx:02x?}"
                    );
                }
                if l.cvc[pos] == 0 {
                    assert_eq!(
                        ctx.as_slice(),
                        &[0u8; 7],
                        "{cvc:?} pos {pos}: missing center is not all zeros: {ctx:02x?}"
                    );
                } else {
                    assert!(
                        !ctx.contains(&0),
                        "{cvc:?} pos {pos}: context of existing center contains 0: {ctx:02x?}"
                    );
                    assert_ne!(ctx[3], 0, "{cvc:?} pos {pos}: center is 0");
                }
            }
        }
    }

    #[test]
    fn multiword_prev_next_code_special_cases() {
        let text = PronText {
            syllables: vec![
                syl("\x02\x03\x01", 0, true),
                syl("\x04\x03\x01", 1, true),
                syl("\x05\x03\x01", 2, true),
            ],
            phoneme_codes: vec![2, 3, 1, 4, 3, 1, 5, 3, 1],

            word_sen: vec![],
        };
        let p = build_phrase(&text, &[tgt(100.0, 180.0); 3]).unwrap();
        let prev1 = prev_code(&p.letters, &p.words, 1);
        assert_eq!(prev1, [0x61; 8]);
        let l1 = &p.letters[1];
        assert_eq!(&l1.sch_jung[0..3], &[0x61, 0x61, 0x04]);
        let (temp1, next1) = next_code(&p.letters, &p.words, 0);
        assert_eq!(temp1, [0x61; 8]);
        assert_eq!(next1, [0x61; 8]);
        let (temp2, next2) = next_code(&p.letters, &p.words, 2);
        assert_eq!(temp2, [0x60; 8]);
        assert_eq!(next2, [0x60; 8]);
    }

    #[test]
    fn mismatch_targets() {
        assert!(build_phrase(&pron_annyong(), &[tgt(100.0, 200.0)]).is_err());
        assert!(
            build_phrase(
                &PronText {
                    syllables: vec![],
                    phoneme_codes: vec![],

                    word_sen: vec![],
                },
                &[]
            )
            .is_err()
        );
    }

    #[test]
    fn phrase_head_flags() {
        let text = PronText {
            syllables: vec![
                syl("\x0d\x03\x01", 0, true),
                syl("\x10\x1d\x01", 0, false),
                syl("\x08\x1b\x05", 0, false),
                syl("\x09\x1d\x05", 1, true),
                syl("\x04\x03\x01", 1, false),
                syl("\x07\x03\x01", 1, false),
                syl("\x0d\x1d\x01", 2, true),
                syl("\x02\x03\x17", 3, true),
                syl("\x0b\x03\x05", 3, false),
            ],
            phoneme_codes: vec![
                13, 3, 1, 16, 29, 1, 8, 27, 5, 9, 29, 5, 4, 3, 1, 7, 3, 1, 13, 29, 1, 2, 3, 23, 11,
                3, 5,
            ],

            word_sen: vec![],
        };
        let p = build_phrase(
            &text,
            &[
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x0a),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x14),
                tgt_bnd(100.0, 180.0, 0x0b),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x15),
            ],
        )
        .unwrap();
        let heads: Vec<bool> = p.letters.iter().map(|l| l.is_phrase_head).collect();
        assert_eq!(
            heads,
            [true, false, false, false, false, false, true, false, false,]
        );
    }

    #[test]
    fn prev_code_reset_after_sentence_end() {
        let text = PronText {
            syllables: vec![
                syl("\x02\x03\x01", 0, true),
                syl("\x05\x1b\x01", 0, false),
                syl("\x14\x03\x05", 0, false),
                syl("\x0d\x1d\x01", 1, true),
            ],
            phoneme_codes: vec![2, 3, 1, 5, 27, 1, 20, 3, 5, 13, 29, 1],

            word_sen: vec![],
        };
        let p = build_phrase(
            &text,
            &[
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x15),
                tgt_bnd(100.0, 180.0, 0x15),
            ],
        )
        .unwrap();
        assert_eq!(p.words[0].rest_flag, REST_SENT_END);
        assert!(p.letters[3].is_phrase_head);
        assert!(p.letters[0].is_phrase_head);
        assert!(!p.letters[1].is_phrase_head);
        assert!(!p.letters[2].is_phrase_head);
        let prev = prev_code(&p.letters, &p.words, 1);
        assert_eq!(prev, [0x60; 8]);
        assert_eq!(
            p.letters[3].sch_jung,
            [0x60, 0x60, 0x60, 0x3d, 0x60, 0x60, 0x60]
        );
        let text2 = PronText {
            syllables: vec![
                syl("\x02\x03\x01", 0, true),
                syl("\x05\x1b\x01", 0, false),
                syl("\x14\x03\x05", 0, false),
                syl("\x0d\x1d\x01", 1, true),
            ],
            phoneme_codes: vec![2, 3, 1, 5, 27, 1, 20, 3, 5, 13, 29, 1],

            word_sen: vec![],
        };
        let p2 = build_phrase(
            &text2,
            &[
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x14),
                tgt_bnd(100.0, 180.0, 0x15),
            ],
        )
        .unwrap();
        assert_eq!(p2.words[0].rest_flag, REST_SPACE);
        assert!(p2.letters[3].is_phrase_head);
        let prev2 = prev_code(&p2.letters, &p2.words, 1);
        assert_eq!(prev2, [0x61; 8]);
        let p3 = build_phrase(
            &text2,
            &[
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x00),
                tgt_bnd(100.0, 180.0, 0x0b),
                tgt_bnd(100.0, 180.0, 0x15),
            ],
        )
        .unwrap();
        assert!(!p3.letters[3].is_phrase_head);
        let prev3 = prev_code(&p3.letters, &p3.words, 1);
        assert_eq!(prev3, [0x05, 0x3b, 0x00, 0xff, 0x14, 0x23, 0x45, 0x62]);
    }
}
