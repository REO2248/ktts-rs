use crate::bi::BiWord;
use crate::cvc;
use ktts_dict::cart::CartTree;

#[derive(Debug, Clone, Default)]
pub(crate) struct LetterValue {
    pub next_phrase_start_tag: u8,
    pub next_start_tag: u8,
    pub next_end_tag: u8,
    pub prev_phrase_end_tag: u8,
    pub prev_start_tag: u8,
    pub prev_end_tag: u8,
    pub cur_start_tag: u8,
    pub cur_end_tag: u8,
    pub cur_tag: u8,
    pub cur_morph_num: u8,
    pub cur_morph_index: u8,
    pub cur_morph_pos: u8,
    pub cho_code: u8,
    pub jung_code: u8,
    pub jong_code: u8,
    pub cvc_index: u8,
    pub phrase_type: u8,
    pub letter_type: u8,
    pub word_letter_type: u8,
    pub total_letter_num: u8,
    pub word_letter_num: u8,
    pub cur_letter_index: u8,
    pub rest_letter_index: u8,
    pub rate_letter_of_word: f32,
    pub cur_total_letter_index: u8,
    pub rest_total_letter_index: u8,
    pub rate_letter_of_sen: f32,
    pub no_end_letter_flag: u8,
    pub end_letter_acc_info: u8,
    pub letter_acc_info: i32,
    pub letter0: u8,
    pub letter1: u8,
    pub letter2: u8,
    pub letter3: u8,
    pub letter4: u8,
    pub letter5: u8,
    pub letter6: u8,
    pub letter_mode: [u8; 2],
    pub connec: [[u8; 4]; 3],
    pub phone_length: [f32; 3],
    pub freq: [f32; 12],
}

impl LetterValue {
    fn feat_duration(&self, mode: usize) -> [f32; 41] {
        let mut f = [0f32; 41];
        let c = &self.connec[mode];
        f[0] = f32::from(c[0]);
        f[1] = f32::from(c[1]);
        f[2] = f32::from(c[2]);
        f[3] = f32::from(self.letter_mode[0]);
        f[4] = f32::from(self.letter_mode[1]);
        f[5] = f32::from(self.next_phrase_start_tag);
        f[6] = f32::from(self.next_start_tag);
        f[7] = f32::from(self.next_end_tag);
        f[8] = f32::from(self.prev_phrase_end_tag);
        f[9] = f32::from(self.prev_start_tag);
        f[10] = f32::from(self.prev_end_tag);
        f[11] = f32::from(self.cur_start_tag);
        f[12] = f32::from(self.cur_end_tag);
        f[13] = f32::from(self.cur_tag);
        f[14] = f32::from(self.cur_morph_num);
        f[15] = f32::from(self.cur_morph_index);
        f[16] = f32::from(self.cur_morph_pos);
        f[17] = f32::from(self.cho_code);
        f[18] = f32::from(self.jung_code);
        f[19] = f32::from(self.jong_code);
        f[20] = f32::from(self.cvc_index);
        f[21] = f32::from(self.phrase_type);
        f[22] = f32::from(self.letter_type);
        f[23] = f32::from(self.word_letter_type);
        f[24] = f32::from(self.total_letter_num);
        f[25] = f32::from(self.word_letter_num);
        f[26] = f32::from(self.cur_letter_index);
        f[27] = f32::from(self.rest_letter_index);
        f[28] = self.rate_letter_of_word;
        f[29] = f32::from(self.cur_total_letter_index);
        f[30] = f32::from(self.rest_total_letter_index);
        f[31] = self.rate_letter_of_sen;
        f[32] = f32::from(self.no_end_letter_flag);
        f
    }

    fn feat_tobi(&self) -> [f32; 41] {
        let mut f = [0f32; 41];
        f[5] = f32::from(self.next_phrase_start_tag);
        f[6] = f32::from(self.next_start_tag);
        f[7] = f32::from(self.next_end_tag);
        f[8] = f32::from(self.prev_phrase_end_tag);
        f[9] = f32::from(self.prev_start_tag);
        f[10] = f32::from(self.prev_end_tag);
        f[11] = f32::from(self.cur_start_tag);
        f[12] = f32::from(self.cur_end_tag);
        f[13] = f32::from(self.cur_tag);
        f[14] = f32::from(self.cur_morph_num);
        f[15] = f32::from(self.cur_morph_index);
        f[16] = f32::from(self.cur_morph_pos);
        f[17] = f32::from(self.cho_code);
        f[18] = f32::from(self.jung_code);
        f[19] = f32::from(self.jong_code);
        f[20] = f32::from(self.cvc_index);
        f[21] = f32::from(self.phrase_type);
        f[22] = f32::from(self.letter_type);
        f[23] = f32::from(self.word_letter_type);
        f[24] = f32::from(self.total_letter_num);
        f[25] = f32::from(self.word_letter_num);
        f[26] = f32::from(self.cur_letter_index);
        f[27] = f32::from(self.rest_letter_index);
        f[28] = self.rate_letter_of_word;
        f[29] = f32::from(self.cur_total_letter_index);
        f[30] = f32::from(self.rest_total_letter_index);
        f[31] = self.rate_letter_of_sen;
        f
    }

    fn feat_f0(&self) -> [f32; 41] {
        let mut f = [0f32; 41];
        f[5] = f32::from(self.next_phrase_start_tag);
        f[6] = f32::from(self.next_start_tag);
        f[7] = f32::from(self.next_end_tag);
        f[8] = f32::from(self.prev_phrase_end_tag);
        f[9] = f32::from(self.prev_start_tag);
        f[10] = f32::from(self.prev_end_tag);
        f[11] = f32::from(self.cur_start_tag);
        f[12] = f32::from(self.cur_end_tag);
        f[13] = f32::from(self.cur_tag);
        f[14] = f32::from(self.cur_morph_num);
        f[15] = f32::from(self.cur_morph_index);
        f[16] = f32::from(self.cur_morph_pos);
        f[17] = f32::from(self.cho_code);
        f[18] = f32::from(self.jung_code);
        f[19] = f32::from(self.jong_code);
        f[20] = f32::from(self.cvc_index);
        f[21] = f32::from(self.phrase_type);
        f[22] = f32::from(self.letter_type);
        f[23] = f32::from(self.word_letter_type);
        f[24] = f32::from(self.total_letter_num);
        f[25] = f32::from(self.word_letter_num);
        f[26] = f32::from(self.cur_letter_index);
        f[27] = f32::from(self.rest_letter_index);
        f[28] = self.rate_letter_of_word;
        f[29] = f32::from(self.cur_total_letter_index);
        f[30] = f32::from(self.rest_total_letter_index);
        f[31] = self.rate_letter_of_sen;
        f[32] = f32::from(self.no_end_letter_flag);
        f[33] = f32::from(self.end_letter_acc_info);
        f[34] = f32::from(self.letter0);
        f[35] = f32::from(self.letter1);
        f[36] = f32::from(self.letter2);
        f[37] = f32::from(self.letter3);
        f[38] = f32::from(self.letter4);
        f[39] = f32::from(self.letter5);
        f[40] = f32::from(self.letter6);
        f
    }
}

pub(crate) struct Phrase {
    pub words: Vec<usize>,
}

pub(crate) struct SentenceWord<'a> {
    pub word: &'a BiWord,
    pub letter_start: usize,
    pub cvc: Vec<[u8; 3]>,
    pub syl_morphs: Vec<(u8, u8)>,
}

pub(crate) struct Sentence<'a> {
    pub words: Vec<SentenceWord<'a>>,
    pub phrases: Vec<Phrase>,
    pub letter_count: usize,
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
fn round_half_up(f: f32) -> i32 {
    (f + 0.5).floor() as i32
}

pub(crate) struct ProsodyTrees {
    pub dur: [CartTree; 7],
    pub bound_tobi: CartTree,
    pub non_bound_tobi: CartTree,
    pub pitch_f0: CartTree,
}

#[derive(Debug, Clone)]
pub(crate) struct LetterTarget {
    pub dur_ms: f32,
    pub ave_length: [u16; 3],
    pub f0: [f32; 12],
    pub tobi: f32,
}

#[inline]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn phone_length_value(r: f32) -> u16 {
    if r == 0.0 {
        return 0;
    }
    let v = (f64::from(r).exp() * 16.0).floor();
    if v <= 0.0 {
        0
    } else if v >= 32767.0 {
        32767
    } else {
        v as u16
    }
}

#[allow(clippy::needless_range_loop)]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
#[expect(
    clippy::suboptimal_flops,
    reason = "C port: float-op order kept bit-exact (mul_add/FMA would change rounding)"
)]
fn phone_accent_arrange(freqs: &mut [[f32; 12]], start: usize, len: usize) {
    const C1: f64 = 0.090_909_093_618_392_94;
    const C2: f64 = 0.076_923_079_788_684_84;
    if len <= 1 {
        return;
    }
    for n in 0..len - 1 {
        let cur = start + n;
        let nxt = cur + 1;
        let mut sr_ave = [0f32; 7];
        let mut sr_ave_next = [0f32; 7];
        for i in 0..7 {
            let v = f64::from(freqs[cur][5 + i]);
            sr_ave[i] = (v * v * 0.5).sqrt() as f32;
            let w = f64::from(freqs[nxt][i]);
            sr_ave_next[i] = (w * w * 0.5).sqrt() as f32;
        }
        sr_ave.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        sr_ave_next.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let c = f64::from(sr_ave[3]);
        let n = f64::from(sr_ave_next[3]);
        let v6 = (n + c) * 0.5;
        let f_cur = f64::from(((v6 / c).sqrt()) as f32);
        let f_next = (v6 / n).sqrt();
        for (k, i) in (-11..=0).enumerate() {
            let m_cur = f64::from(i) * (f_cur - 1.0) * C1 + f_cur;
            let m_next = f64::from(i) * (1.0 - f_next) * C1 + 1.0;
            freqs[cur][k] = (m_cur * f64::from(freqs[cur][k])) as f32;
            freqs[nxt][k] = (m_next * f64::from(freqs[nxt][k])) as f32;
        }
        let nf6 = f64::from(freqs[nxt][6]);
        let delta = nf6 - f64::from(freqs[cur][5]);
        for j in 0..7 {
            let val = nf6 + ((j as i64 - 13) as f64) * delta * C2;
            freqs[cur][5 + j] = val as f32;
        }
        let delta2 = nf6 - f64::from(freqs[cur][5]);
        for j in 0..7 {
            let val = nf6 + ((j as i64 - 6) as f64) * delta2 * C2;
            freqs[nxt][j] = val as f32;
        }
    }
}

pub(crate) fn get_length_and_ave_pitch(trees: &ProsodyTrees, sent: &Sentence) -> Vec<LetterTarget> {
    let n_letters = sent.letter_count;
    let mut letters: Vec<LetterValue> = vec![LetterValue::default(); n_letters];
    let mut acc_info: Vec<i32> = vec![0; n_letters];
    let mut tobi_raw: Vec<f32> = vec![0.0; n_letters];

    loop1_tobi_tags(&mut letters, &mut acc_info, &mut tobi_raw, sent, trees);

    loop2_cvc_duration(&mut letters, sent, trees);

    loop3_f0(&mut letters, &acc_info, n_letters, trees);

    loop35_phone_accent(&mut letters, sent);

    assemble_output(&letters, &tobi_raw, n_letters)
}
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
#[expect(
    clippy::similar_names,
    clippy::too_many_lines,
    reason = "C port: disassembler/domain variable names kept as-is; large ported function"
)]
fn loop1_tobi_tags(
    letters: &mut [LetterValue],
    acc_info: &mut [i32],
    tobi_raw: &mut [f32],
    sent: &Sentence,
    trees: &ProsodyTrees,
) {
    let mut end_acc_info: u8 = 7;
    for (pi, phrase) in sent.phrases.iter().enumerate() {
        let n_phrase_words = phrase.words.len();
        let phrase_letter_num: usize = phrase
            .words
            .iter()
            .map(|&wi| sent.words[wi].word.len())
            .sum();
        if n_phrase_words == 0 {
            continue;
        }
        let mut phrase_letter_idx = 0usize;
        for (wi_in_phrase, &wi) in phrase.words.iter().enumerate() {
            let word = &sent.words[wi].word;
            let _n_words = sent.words.len();
            let w_len = word.len();
            let first_tag = *word.morph_pos.first().unwrap_or(&0);
            let last_tag = *word.morph_pos.last().unwrap_or(&0);
            let next_phrase_start_tag: u8 = if pi + 1 == sent.phrases.len() {
                b'.'
            } else {
                let nw = sent.phrases[pi + 1].words[0];
                *sent.words[nw].word.morph_pos.first().unwrap_or(&0)
            };
            let prev_phrase_end_tag: u8 = if pi == 0 {
                b'.'
            } else {
                let pw = sent.phrases[pi - 1].words.last().copied().unwrap_or(0);
                let pword = &sent.words[pw].word;
                *pword.morph_pos.last().unwrap_or(&0)
            };
            let next_start_tag = {
                let lw = phrase.words[n_phrase_words - 1];
                *sent.words[lw].word.morph_pos.first().unwrap_or(&0)
            };
            let next_end_tag = {
                let lw = phrase.words[n_phrase_words - 1];
                *sent.words[lw].word.morph_pos.last().unwrap_or(&0)
            };
            let prev_start_tag = {
                let fw = phrase.words[0];
                *sent.words[fw].word.morph_pos.first().unwrap_or(&0)
            };
            let prev_end_tag = {
                let fw = phrase.words[0];
                *sent.words[fw].word.morph_pos.last().unwrap_or(&0)
            };
            let cur_start_tag = first_tag;
            let cur_end_tag = last_tag;
            let morph_num = word.w_morph_cnt as u8;

            let sw = &sent.words[wi];
            let morph_map_ok = sw.syl_morphs.len() == w_len;
            for li in 0..w_len {
                let letter = &mut letters[sw.letter_start + li];
                letter.next_start_tag = next_start_tag;
                letter.next_end_tag = next_end_tag;
                letter.next_phrase_start_tag = next_phrase_start_tag;
                letter.prev_phrase_end_tag = prev_phrase_end_tag;
                letter.prev_start_tag = prev_start_tag;
                letter.prev_end_tag = prev_end_tag;
                letter.cur_start_tag = cur_start_tag;
                letter.cur_end_tag = cur_end_tag;
                letter.cur_morph_num = morph_num;
                if morph_map_ok {
                    let (mi, mp) = sw.syl_morphs[li];
                    letter.cur_morph_index = mi;
                    letter.cur_morph_pos = mp;
                    letter.cur_tag = *word
                        .morph_pos
                        .get(mi as usize)
                        .unwrap_or_else(|| &word.morph_pos[li.min(word.morph_pos.len() - 1)]);
                } else {
                    letter.cur_morph_index = li as u8;
                    letter.cur_morph_pos = 0;
                    letter.cur_tag = *word.morph_pos.get(li).unwrap_or(&0);
                }

                let [cho, jung, jong] = sw.cvc[li];
                let s_idx = cvc::syllable_index([cho, jung, jong]);
                letter.cvc_index = s_idx;
                match s_idx {
                    0 => {
                        letter.cho_code = 13;
                        letter.jong_code = b'A';
                        letter.jung_code = jung + 0x20;
                    }
                    1 => {
                        letter.jong_code = b'A';
                        letter.cho_code = cho;
                        letter.jung_code = jung + 0x20;
                    }
                    2 => {
                        letter.cho_code = 13;
                        letter.jung_code = jung + 0x20;
                        letter.jong_code = jong + 0x40;
                    }
                    _ => {
                        letter.cho_code = cho;
                        letter.jung_code = jung + 0x20;
                        letter.jong_code = jong + 0x40;
                    }
                }

                let ch_phrase_num = sent.phrases.len();
                letter.phrase_type = if ch_phrase_num < 3 {
                    if pi == 0 || ch_phrase_num < 2 { 0 } else { 2 }
                } else if pi == 0 {
                    0
                } else if pi + 1 == ch_phrase_num {
                    2
                } else {
                    1
                };
                letter.letter_type = if phrase_letter_num < 3 {
                    if phrase_letter_idx == 0 || phrase_letter_num == 1 {
                        0
                    } else {
                        u8::from(phrase_letter_num == 2)
                    }
                } else if phrase_letter_idx == 0 {
                    0
                } else if phrase_letter_idx == 1 {
                    1
                } else {
                    2
                };
                letter.word_letter_type = if w_len < 3 {
                    if li == 0 || w_len == 1 {
                        0
                    } else {
                        u8::from(w_len == 2)
                    }
                } else if li == 0 {
                    0
                } else if li == 1 {
                    1
                } else {
                    2
                };
                letter.total_letter_num = phrase_letter_num as u8;
                letter.word_letter_num = w_len as u8;
                letter.cur_letter_index = li as u8;
                letter.rest_letter_index = (w_len - 1 - li) as u8;
                letter.cur_total_letter_index = phrase_letter_idx as u8;
                letter.rest_total_letter_index = (phrase_letter_num - 1 - phrase_letter_idx) as u8;
                letter.rate_letter_of_word = li as f32 / w_len as f32;
                letter.rate_letter_of_sen = phrase_letter_idx as f32 / phrase_letter_num as f32;
                let is_last_letter = li + 1 == w_len;
                let is_last_word = wi_in_phrase + 1 == n_phrase_words;
                letter.no_end_letter_flag = u8::from(!(is_last_letter && is_last_word));
                letter.letter_mode[0] = if w_len < 3 {
                    if li == 0 || w_len == 1 { 0 } else { 2 }
                } else if li == 0 {
                    0
                } else if li + 1 == w_len {
                    2
                } else {
                    1
                };
                letter.letter_mode[1] = if phrase_letter_num < 3 {
                    if phrase_letter_idx == 0 || phrase_letter_num == 1 {
                        0
                    } else {
                        2
                    }
                } else if phrase_letter_idx == 0 {
                    0
                } else if phrase_letter_idx + 1 == phrase_letter_num {
                    2
                } else {
                    1
                };
                letter.end_letter_acc_info = end_acc_info;

                let feat = letter.feat_tobi();
                let tree = if letter.no_end_letter_flag == 0 {
                    &trees.bound_tobi
                } else {
                    &trees.non_bound_tobi
                };
                let out = tree.eval(&feat).map_or(0.0, |v| v[0]);
                tobi_raw[sent.words[wi].letter_start + li] = out;
                let acc = round_half_up(out);
                acc_info[sent.words[wi].letter_start + li] = acc;
                letter.letter_acc_info = acc;

                if is_last_letter && is_last_word {
                    end_acc_info = acc as u8;
                }
                phrase_letter_idx += 1;
            }
        }
    }
}
#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
fn loop2_cvc_duration(letters: &mut [LetterValue], sent: &Sentence, trees: &ProsodyTrees) {
    for sw in &sent.words {
        let word = sw.word;
        let w_len = word.len();
        for li in 0..w_len {
            let gi = sw.letter_start + li;
            let [cho, jung, jong] = sw.cvc[li];
            let s_idx = letters[gi].cvc_index;
            let prev: Option<(u8, u8)> = if li > 0 {
                let p = letters[gi - 1].cvc_index;
                let pj = letters[gi - 1].jung_code;
                let pc = letters[gi - 1].jong_code;
                Some((p, if p == 2 || p == 3 { pc } else { pj }))
            } else {
                None
            };
            let next: Option<(u8, u8)> = if li + 1 < w_len {
                let n = letters[gi + 1].cvc_index;
                let nj = letters[gi + 1].jung_code;
                let nc = letters[gi + 1].cho_code;
                Some((n, if n == 1 || n == 3 { nc } else { nj }))
            } else {
                None
            };
            let prev_code = prev.map_or(0, |(_, c)| c);
            let next_code = next.map_or(0, |(_, c)| c);
            let (con0, con1, con2, grp0, grp1, grp2): ([u8; 3], [u8; 3], [u8; 3], u8, u8, u8);
            match s_idx {
                0 => {
                    con0 = [prev_code, jung + 0x20, next_code];
                    con1 = [0, 0, 0];
                    con2 = [0, 0, 0];
                    grp0 = cvc::cvc_group_index(jung + 0x20);
                    grp1 = 0;
                    grp2 = 0;
                }
                1 => {
                    con0 = [prev_code, cho, jung + 0x20];
                    con1 = [cho, jung + 0x20, next_code];
                    con2 = [0, 0, 0];
                    grp0 = cvc::cvc_group_index(cho);
                    grp1 = cvc::cvc_group_index(jung + 0x20);
                    grp2 = 0;
                }
                2 => {
                    con0 = [prev_code, jung + 0x20, jong + 0x40];
                    con1 = [jung + 0x20, jong + 0x40, next_code];
                    con2 = [0, 0, 0];
                    grp0 = cvc::cvc_group_index(jung + 0x20);
                    grp1 = cvc::cvc_group_index(jong + 0x40);
                    grp2 = 0;
                }
                _ => {
                    con0 = [prev_code, cho, jung + 0x20];
                    con1 = [cho, jung + 0x20, jong + 0x40];
                    con2 = [jung + 0x20, jong + 0x40, next_code];
                    grp0 = cvc::cvc_group_index(cho);
                    grp1 = cvc::cvc_group_index(jung + 0x20);
                    grp2 = cvc::cvc_group_index(jong + 0x40);
                }
            }
            let l = &mut letters[gi];
            l.connec[0] = [con0[0], con0[1], con0[2], grp0];
            l.connec[1] = [con1[0], con1[1], con1[2], grp1];
            l.connec[2] = [con2[0], con2[1], con2[2], grp2];
            l.phone_length = [0.0; 3];
            match s_idx {
                0 => {
                    l.phone_length[0] = eval_duration(trees, l, 0);
                }
                1 => {
                    l.phone_length[0] = eval_duration(trees, l, 0);
                    l.phone_length[1] = eval_duration(trees, l, 1);
                }
                2 => {
                    l.phone_length[0] = eval_duration(trees, l, 0);
                    if grp1 == 3 || grp1 == 4 {
                        l.phone_length[1] = eval_duration(trees, l, 1);
                    }
                }
                _ => {
                    l.phone_length[0] = eval_duration(trees, l, 0);
                    l.phone_length[1] = eval_duration(trees, l, 1);
                    if grp2 == 3 || grp2 == 4 {
                        l.phone_length[2] = eval_duration(trees, l, 2);
                    }
                }
            }
        }
    }
}
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn loop3_f0(letters: &mut [LetterValue], acc_info: &[i32], n_letters: usize, trees: &ProsodyTrees) {
    for gi in 0..n_letters {
        let l = &mut letters[gi];
        let n = n_letters;
        let acc = |idx: i32| -> u8 {
            if idx < 0 || idx >= n as i32 {
                7
            } else {
                acc_info[idx as usize] as u8
            }
        };
        l.letter0 = acc(gi as i32 - 3);
        l.letter1 = acc(gi as i32 - 2);
        l.letter2 = acc(gi as i32 - 1);
        l.letter3 = acc(gi as i32);
        l.letter4 = if gi + 1 < n {
            acc_info[gi + 1] as u8
        } else {
            7
        };
        l.letter5 = if gi + 2 < n {
            acc_info[gi + 2] as u8
        } else {
            7
        };
        l.letter6 = if gi + 3 < n {
            acc_info[gi + 3] as u8
        } else {
            7
        };
        let feat = l.feat_f0();
        let out = trees
            .pitch_f0
            .eval(&feat)
            .map(<[f32]>::to_vec)
            .unwrap_or_default();
        let mut f0 = [0f32; 12];
        for (i, v) in out.iter().enumerate().take(12) {
            f0[i] = *v;
        }
        l.freq = f0;
    }
}
fn loop35_phone_accent(letters: &mut [LetterValue], sent: &Sentence) {
    for sw in &sent.words {
        let w_len = sw.word.len();
        if w_len > 1 {
            let start = sw.letter_start;
            let mut freqs: Vec<[f32; 12]> = letters[start..start + w_len]
                .iter()
                .map(|l| l.freq)
                .collect();
            phone_accent_arrange(&mut freqs, 0, w_len);
            for (i, f) in freqs.iter().enumerate() {
                letters[start + i].freq = *f;
            }
        }
    }
}
fn assemble_output(
    letters: &[LetterValue],
    tobi_raw: &[f32],
    n_letters: usize,
) -> Vec<LetterTarget> {
    let mut out = Vec::with_capacity(n_letters);
    for (gi, l) in letters.iter().enumerate() {
        let pl = &l.phone_length;
        let ave_length: [u16; 3] = match l.cvc_index {
            0 => [0, phone_length_value(pl[0]), 0],
            1 => [phone_length_value(pl[0]), phone_length_value(pl[1]), 0],
            2 => [0, phone_length_value(pl[0]), phone_length_value(pl[1])],
            _ => [
                phone_length_value(pl[0]),
                phone_length_value(pl[1]),
                phone_length_value(pl[2]),
            ],
        };
        let dur_ms = {
            let mut samples = 0f32;
            for &a in &ave_length {
                samples += f32::from(a);
            }
            samples / 16.0
        };
        out.push(LetterTarget {
            dur_ms,
            ave_length,
            f0: l.freq,
            tobi: tobi_raw[gi],
        });
    }
    out
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
pub(crate) fn question_mark_f0_transform(freq: &mut [f32; 12]) {
    const GR_ACCENT_HIGH: [f64; 12] = [
        0.98, 0.96, 0.94, 0.92, 0.89, 0.86, 0.82, 0.78, 0.74, 0.70, 0.65, 0.60,
    ];
    if freq[0] <= 0.0 {
        return;
    }
    let p0 = (16000.0f64 / f64::from(freq[0]) + 0.5).trunc();
    for k in 1..12 {
        let sw = (p0 * GR_ACCENT_HIGH[k] + 0.5).trunc();
        if sw >= 1.0 {
            freq[k] = (16000.0f64 / sw) as f32;
        }
    }
}

fn eval_duration(trees: &ProsodyTrees, l: &LetterValue, mode: usize) -> f32 {
    let tree = match l.connec[mode][3] {
        0 => &trees.dur[0],
        1 => &trees.dur[1],
        2 => &trees.dur[2],
        3 => &trees.dur[3],
        4 => &trees.dur[4],
        5 => &trees.dur[5],
        _ => &trees.dur[6],
    };
    let feat = l.feat_duration(mode);
    tree.eval(&feat).map_or(0.0, |v| v[0])
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::float_cmp,
        reason = "test fixtures: oracle values converted with intentional casts"
    )]
    use super::*;

    #[test]
    fn round_half_up_behavior() {
        assert_eq!(round_half_up(0.0), 0);
        assert_eq!(round_half_up(0.49), 0);
        assert_eq!(round_half_up(0.5), 1);
        assert_eq!(round_half_up(4.01), 4);
        assert_eq!(round_half_up(5.75), 6);
    }

    #[allow(clippy::field_reassign_with_default)]
    #[test]
    fn feat_vectors_match_ph2_tree_layout() {
        let mut l = LetterValue::default();
        l.next_phrase_start_tag = 46;
        l.cho_code = 2;
        l.jung_code = 35;
        l.jong_code = 65;
        let fd = l.feat_duration(0);
        assert_eq!(fd[5], 46.0);
        assert_eq!(fd[17], 2.0);
        assert_eq!(fd[18], 35.0);
        assert_eq!(fd[19], 65.0);
        let ft = l.feat_tobi();
        assert_eq!(ft[5], 46.0);
        assert_eq!(ft[17], 2.0);
        l.no_end_letter_flag = 1;
        l.end_letter_acc_info = 7;
        l.letter3 = 3;
        let ff = l.feat_f0();
        assert_eq!(ff[32], 1.0);
        assert_eq!(ff[33], 7.0);
        assert_eq!(ff[37], 3.0);
    }

    #[test]
    fn duration_feature_x_range() {
        let mut l = LetterValue::default();
        l.connec[0] = [0, 35, 0, 5];
        l.letter_mode = [0, 1];
        l.cur_morph_num = 2;
        l.cur_morph_index = 1;
        l.cur_morph_pos = 0;
        let fd = l.feat_duration(0);
        assert_eq!(fd[0], 0.0);
        assert_eq!(fd[1], 35.0);
        assert_eq!(fd[3], 0.0);
        assert_eq!(fd[4], 1.0);
        assert_eq!(fd[14], 2.0);
        assert_eq!(fd[15], 1.0);
        assert_eq!(fd[16], 0.0);
        assert_eq!(fd[33..41], [0.0; 8]);
    }

    #[test]
    fn phone_length_value_matches_c() {
        assert_eq!(phone_length_value(0.0), 0);
        assert_eq!(phone_length_value(1.0), 43);
        assert_eq!(phone_length_value(2.0), 118);
        assert_eq!(phone_length_value(-1.0), 5);
        assert_eq!(phone_length_value(20.0), 32767);
        assert_eq!(phone_length_value(-10.0), 0);
    }

    #[test]
    fn ave_length_cvc_index_mapping() {
        let map = |idx: u8, pl: [f32; 3]| -> [u16; 3] {
            match idx {
                0 => [0, phone_length_value(pl[0]), 0],
                1 => [phone_length_value(pl[0]), phone_length_value(pl[1]), 0],
                2 => [0, phone_length_value(pl[0]), phone_length_value(pl[1])],
                _ => [
                    phone_length_value(pl[0]),
                    phone_length_value(pl[1]),
                    phone_length_value(pl[2]),
                ],
            }
        };
        assert_eq!(map(0, [1.0, 0.0, 0.0]), [0, 43, 0]);
        assert_eq!(map(1, [1.0, 2.0, 0.0]), [43, 118, 0]);
        assert_eq!(map(2, [1.0, 2.0, 0.0]), [0, 43, 118]);
        assert_eq!(map(3, [1.0, 2.0, 3.0]), [43, 118, 321]);
        assert_eq!(map(3, [0.0, 2.0, 0.0]), [0, 118, 0]);
    }
}
