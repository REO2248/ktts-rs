use crate::charstr::{StrInfo, set_tag_from_attribute};
use crate::code::{conv_pyogi_to_cvc, conv_pyogi_to_uni_wan, conv_uni_wan_to_pyogi};
use crate::dict::{KAnalInfo, KmaDicts};
use crate::tables;

#[derive(Debug, Clone)]
pub struct MorphNode {
    pub pyogi: Vec<u8>,
    pub ch_tag: u8,
    pub d_self_prob: f64,
    pub d_accum_prob: f64,
    pub w_route_idx: i16,
    pub b_next_morph_flag: u8,
    pub child: Vec<i16>,
    pub parent: Vec<i16>,
    pub b_standard_to_flag: bool,
    pub w_start_pos: i16,
    pub w_end_pos: i16,
    pub b_retrieved: bool,
}

impl MorphNode {
    const fn new() -> Self {
        Self {
            pyogi: Vec::new(),
            ch_tag: 0,
            d_self_prob: 0.0,
            d_accum_prob: 0.0,
            w_route_idx: -1,
            b_next_morph_flag: b'x',
            child: Vec::new(),
            parent: Vec::new(),
            b_standard_to_flag: false,
            w_start_pos: 0,
            w_end_pos: 0,
            b_retrieved: false,
        }
    }
}

#[derive(Debug, Clone)]
struct MorphCand {
    w_start_pos: i16,
    w_end_pos: i16,
    w_root_pos: i16,
    sw_link_morph_idx: Vec<i16>,
    sch_morph_str: Vec<u8>,
    ch_pumsa: u8,
    d_probability: f64,
    d_part_prob: f64,
    un_to_info: u32,
    b_retrieved: bool,
}

impl MorphCand {
    const fn new() -> Self {
        Self {
            w_start_pos: 0,
            w_end_pos: 0,
            w_root_pos: 0,
            sw_link_morph_idx: Vec::new(),
            sch_morph_str: Vec::new(),
            ch_pumsa: 0,
            d_probability: 0.0,
            d_part_prob: 0.0,
            un_to_info: 0,
            b_retrieved: false,
        }
    }
}

#[derive(Debug, Clone)]
struct PyogiInfo {
    w_self_pos: i16,
    w_next_pos: i16,
    b_start_able_flag: u8,
    b_end_able_flag: u8,
    ch_eng_pyogi: u8,
    sch_con_status: [u8; 20],
}

impl PyogiInfo {
    const fn new() -> Self {
        Self {
            w_self_pos: 0,
            w_next_pos: 0,
            b_start_able_flag: 0,
            b_end_able_flag: 0,
            ch_eng_pyogi: 0,
            sch_con_status: [b' '; 20],
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct FocusInfo {
    w_ini_pos: i16,
    w_fin_pos: i16,
    w_pyogi_cnt: i16,
    m_w_word_start_pos: i16,
    m_w_word_end_pos: i16,
    f_search_direct: u8,
}

#[derive(Debug, Clone)]
pub struct IregulerStr {
    pub pyogi: Vec<u8>,
    pub irr_string: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct MaMorph {
    pub ch_tag: u8,
    pub pyogi: Vec<u8>,
    pub cvc: Vec<u8>,
    pub prob: f64,
    pub b_merged: bool,
}

#[derive(Debug, Clone)]
pub struct MaWord {
    pub source: Vec<u16>,
    pub morphs: Vec<MaMorph>,
    pub b_str_type: u8,
    pub ireguler: Vec<IregulerStr>,
    pub b_sentence_end: bool,
}

pub(crate) struct KlpState {
    pub(crate) words: Vec<MaWord>,
    gb_ini_tag: u8,
    gb_ini_ll_tag: u8,
    gb_end_tag: u8,
    gsch_ini_str: Vec<u8>,
    gsch_ini_ll_str: Vec<u8>,
    gn_total_str_count: i32,
}

#[cfg(test)]
pub(crate) const fn klp_state_with_words(words: Vec<MaWord>) -> KlpState {
    KlpState {
        words,
        gb_ini_tag: b'j',
        gb_ini_ll_tag: b'j',
        gb_end_tag: b'l',
        gsch_ini_str: Vec::new(),
        gsch_ini_ll_str: Vec::new(),
        gn_total_str_count: 0,
    }
}

struct MorphAnalBuf {
    pw_korea_str: Vec<u16>,
    pch_pyogi_str: Vec<u8>,
    w_pyogi_len: i16,
    w_fin_pos: i16,
    ps_pyogi_info: Vec<PyogiInfo>,
    ps_morph_cand: Vec<MorphCand>,
    n_morph_cand_cnt: i32,
    n_morpheme_cnt: i32,
    ps_morpheme: Vec<MorphNode>,
    s_focus_range: [FocusInfo; 3],
    b_focus_idx: i8,
    b_irr_number: i8,
    pps_ireguler: Vec<IregulerStr>,
    gsch_morph_cand_str: Vec<u8>,
    gn_morph_cand_str: usize,
    gs_k_anal_info: Vec<KAnalInfo>,
    gb_ini_tag: u8,
    gb_ini_ll_tag: u8,
    gb_end_tag: u8,
    gsch_ini_str: Vec<u8>,
    gsch_ini_ll_str: Vec<u8>,
    gn_total_str_count: i32,
    gw_pre_tail_phone_start_pos: i16,
    pw_link_idx_cache: Vec<i16>,
}

impl MorphAnalBuf {
    fn new(klp: &KlpState) -> Self {
        Self {
            pw_korea_str: Vec::new(),
            pch_pyogi_str: Vec::new(),
            w_pyogi_len: 0,
            w_fin_pos: 0,
            ps_pyogi_info: Vec::new(),
            ps_morph_cand: Vec::new(),
            n_morph_cand_cnt: -1,
            n_morpheme_cnt: 0,
            ps_morpheme: Vec::new(),
            s_focus_range: [FocusInfo {
                w_ini_pos: 0,
                w_fin_pos: 0,
                w_pyogi_cnt: 0,
                m_w_word_start_pos: 0,
                m_w_word_end_pos: 0,
                f_search_direct: 0,
            }; 3],
            b_focus_idx: -1,
            b_irr_number: -1,
            pps_ireguler: Vec::new(),
            gsch_morph_cand_str: Vec::new(),
            gn_morph_cand_str: 0,
            gs_k_anal_info: Vec::new(),
            gb_ini_tag: klp.gb_ini_tag,
            gb_ini_ll_tag: klp.gb_ini_ll_tag,
            gb_end_tag: klp.gb_end_tag,
            gsch_ini_str: klp.gsch_ini_str.clone(),
            gsch_ini_ll_str: klp.gsch_ini_ll_str.clone(),
            gn_total_str_count: klp.gn_total_str_count,
            gw_pre_tail_phone_start_pos: 0,
            pw_link_idx_cache: Vec::new(),
        }
    }
}

#[expect(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn change_from_use_dic(buf: &mut Vec<u16>, d: &KmaDicts) {
    let mut n = 0usize;
    while n < buf.len() {
        let Some(idx) = d.user_dic.phrase_lookup(&buf[n..]) else {
            n += 1;
            continue;
        };
        let (src, tgt) = d.user_dic.entry(idx);
        let new_len = buf.len() as isize + tgt.len() as isize - src.len() as isize;
        if 499 < new_len {
            break;
        }
        let mut dst: Vec<u16> = Vec::with_capacity(new_len as usize + 1);
        dst.extend_from_slice(&buf[..n]);
        dst.extend_from_slice(tgt);
        dst.extend_from_slice(&buf[n + src.len()..]);
        *buf = dst;
        n += tgt.len();
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "C port: index/math casts with wrap semantics"
)]
/// Runs the morphological analyzer on UTF-16 input.
///
/// # Errors
///
/// Returns an error if the input is invalid.
pub fn klp_proc(ctx: &crate::KmaContext, input: &[u16]) -> Result<Vec<MaWord>, String> {
    let mut buf = input.to_vec();
    text_process(&mut buf, ctx.tts_check_sentence_cut);
    change_from_use_dic(&mut buf, &ctx.d);
    let infos: Vec<StrInfo> = crate::charstr::get_char_type_str(&buf, true, Some(&ctx.d));
    if infos.is_empty() {
        return Ok(Vec::new());
    }
    let mut klp = KlpState {
        words: Vec::new(),
        gb_ini_tag: b'j',
        gb_ini_ll_tag: b'j',
        gb_end_tag: b'l',
        gsch_ini_str: Vec::new(),
        gsch_ini_ll_str: Vec::new(),
        gn_total_str_count: infos.len() as i32,
    };
    kma_anal(&infos, &mut klp, &ctx.d);
    crate::preproc::main_pre_process(&mut klp, &ctx.d);
    Ok(klp.words)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "C port: index/math casts with wrap semantics"
)]
/// Runs the morphological analyzer, returning all analysis candidates.
///
/// # Errors
///
/// Returns an error if the input is invalid.
pub fn klp_proc_all(ctx: &crate::KmaContext, input: &[u16]) -> crate::KmaResult<Vec<MaWord>> {
    let mut words = Vec::new();
    let mut pos = 0usize;
    let mut guard = 0usize;
    while pos < input.len() {
        guard += 1;
        if guard > 10000 {
            break;
        }
        let mut buf = input[pos..].to_vec();
        let consumed = text_process(&mut buf, true);
        pos += consumed.max(1);
        if buf.is_empty() {
            continue;
        }
        change_from_use_dic(&mut buf, &ctx.d);
        let infos: Vec<StrInfo> = crate::charstr::get_char_type_str(&buf, true, Some(&ctx.d));
        if infos.is_empty() {
            continue;
        }
        let mut klp = KlpState {
            words: Vec::new(),
            gb_ini_tag: b'j',
            gb_ini_ll_tag: b'j',
            gb_end_tag: b'l',
            gsch_ini_str: Vec::new(),
            gsch_ini_ll_str: Vec::new(),
            gn_total_str_count: infos.len() as i32,
        };
        kma_anal(&infos, &mut klp, &ctx.d);
        crate::preproc::main_pre_process(&mut klp, &ctx.d);
        if let Some(last) = klp.words.last_mut() {
            last.b_sentence_end = true;
        }
        words.extend(klp.words);
    }
    Ok(words)
}

fn text_process(buf: &mut Vec<u16>, tts_check_sentence_cut: bool) -> usize {
    for w in buf.iter_mut() {
        if w.wrapping_add(0xff) < 0x5e {
            *w = w.wrapping_add(0x120);
        }
    }
    let n = buf.len();
    let mut out: Vec<u16> = Vec::with_capacity(n);
    let mut i = 0usize;
    let mut cut_pos: Option<usize> = None;
    while i < n {
        let mut w = buf[i];
        let mode = sentence_mode(&mut w);
        if mode == 4 {
            i += 1;
            if !out.is_empty() && tts_check_sentence_cut {
                break;
            }
            continue;
        }
        out.push(w);
        let mut cut = false;
        if mode == 1 || mode == 2 {
            if i != 0 && i + 1 < n {
                let prev = buf[i - 1];
                if is_digit_w(prev) {
                    let next = buf[i + 1];
                    if is_digit_w(next) || (next == 0x20 && i + 2 < n && is_digit_w(buf[i + 2])) {
                    } else {
                        cut = true;
                    }
                } else if is_alpha_w(prev) && is_comma_symbol(buf[i + 1]) {
                } else {
                    cut = true;
                }
            } else {
                cut = true;
            }
        } else if mode == 5 {
            if i + 1 >= n || (buf[i + 1] >> 8) == 0 {
                cut = true;
            }
        } else if mode == 3 && !(w == 0x21 && i + 1 < n && buf[i + 1] == 0x300b) {
            cut = true;
        }
        if cut {
            cut_pos = Some(i + 1);
            break;
        }
        i += 1;
        if out.len() >= 500 {
            break;
        }
    }
    *buf = out;
    cut_pos.unwrap_or(i)
}

#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
const fn sentence_mode(pw_byte: &mut u16) -> i32 {
    let u = *pw_byte;
    if u == 0x300b {
        return 5;
    }
    if u > 0x300b {
        if u == 0xf114 {
            *pw_byte = 0xc77c;
            return 0;
        }
        if u < 0xf115 {
            if u == 0xcb70 {
                *pw_byte = 0x2e;
                return 2;
            }
            if u < 0xcb71 {
                if u == 0x3011 {
                    return 5;
                }
                if u == 0x3f3f {
                    *pw_byte = 0x20;
                    return 0;
                }
                return 0;
            }
            if u != 0xf106 {
                if u == 0xf113 {
                    *pw_byte = 0xae40;
                    return 0;
                }
                if u != 0xf105 {
                    return 0;
                }
            }
            *pw_byte = 0x2e;
            return 5;
        }
        if u != 0xf118 {
            if u < 0xf119 {
                if u != 0xf116 {
                    if u < 0xf117 {
                        *pw_byte = 0xc131;
                        return 0;
                    }
                    *pw_byte = 0xc815;
                    return 0;
                }
                return 0;
            }
            if u == 0xf121 {
                *pw_byte = 0xc815;
                return 0;
            }
            if u == 0xf122 {
                *pw_byte = 0xc740;
                return 0;
            }
            if u != 0xf120 {
                return 0;
            }
            *pw_byte = 0xae40;
            return 0;
        }
        *pw_byte = 0xc77c;
        return 0;
    }
    if u == 0x3f {
        return 3;
    }
    if u < 0x40 {
        if u == 0x21 {
            return 3;
        }
        if u < 0x22 {
            if u != 10 && u != 0x0d {
                return 0;
            }
            return 4;
        }
        if u == 0x3a {
            return 2;
        }
        if u == 0x3b {
            return 3;
        }
        if u != 0x2e {
            return 0;
        }
        return 1;
    }
    if u == 0x2026 {
        *pw_byte = 0x2e;
        return 3;
    }
    if u < 0x2027 {
        if u == 0x201a {
            *pw_byte = 0x2c;
            return 0;
        }
        if u == 0x2025 {
            *pw_byte = 0x2e;
            return 3;
        }
    } else {
        if u == 0x3001 {
            *pw_byte = 0x2c;
            return 0;
        }
        if u == 0x3002 {
            *pw_byte = 0x2e;
            return 1;
        }
        if u == 0x3000 {
            *pw_byte = 0x20;
            return 0;
        }
    }
    0
}

const fn is_digit_w(c: u16) -> bool {
    c.wrapping_sub(0x30) < 10
}

const fn is_alpha_w(c: u16) -> bool {
    c.wrapping_sub(0x61) < 0x1a || c.wrapping_sub(0x41) < 0x1a
}

const fn is_comma_symbol(w: u16) -> bool {
    if is_digit_w(w) || is_alpha_w(w) {
        return true;
    }
    w == 0x2f || w == 0x5c
}

fn kma_anal(infos: &[StrInfo], klp: &mut KlpState, d: &KmaDicts) {
    for (i, info) in infos.iter().enumerate() {
        if info.f_char_type == 0 {
            morph_anal_korea(infos, i, klp, d);
        } else {
            morph_anal_symbol(info, klp);
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
fn morph_anal_symbol(info: &StrInfo, klp: &mut KlpState) {
    if klp.words.is_empty() && info.f_char_type == 0x05 {
        return;
    }
    let tag = set_tag_from_attribute(info.f_char_type);
    klp.words.push(MaWord {
        source: info.pw_str.clone(),
        morphs: vec![MaMorph {
            ch_tag: tag,
            pyogi: info.pw_str.iter().map(|&c| c as u8).collect(),
            cvc: Vec::new(),
            prob: 0.0,
            b_merged: false,
        }],
        b_str_type: 0,
        ireguler: Vec::new(),
        b_sentence_end: false,
    });
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn morph_anal_korea(infos: &[StrInfo], n_index: usize, klp: &mut KlpState, d: &KmaDicts) {
    set_ini_tag(infos, n_index, klp);
    let mut ma = MorphAnalBuf::new(klp);
    morph_anal_init(&infos[n_index].pw_str, &mut ma, klp);
    morph_cand_process(&mut ma, d);
    let mut idx_array = vec![0i32; ma.n_morpheme_cnt.max(0x14) as usize + 2];
    let n = viterbi_search(n_index as i32 - 1, &mut ma, d, &mut idx_array);
    let word = format_ma_result(&ma, klp, &idx_array[..n], d);
    klp.words.push(word);
}

fn morph_anal_init(pw_uni_str: &[u16], ma: &mut MorphAnalBuf, klp: &KlpState) {
    ma.b_irr_number = -1;
    ma.b_focus_idx = -1;
    ma.n_morpheme_cnt = 1;
    ma.n_morph_cand_cnt = -1;
    ma.pw_korea_str = pw_uni_str.to_vec();
    let mut bos = MorphNode::new();
    bos.pyogi = if klp.gsch_ini_str.is_empty() {
        b"INI".to_vec()
    } else {
        klp.gsch_ini_str.clone()
    };
    bos.d_self_prob = 0.0;
    bos.b_next_morph_flag = b'x';
    bos.ch_tag = klp.gb_ini_tag;
    bos.d_accum_prob = 0.0;
    bos.child = vec![1, -1];
    bos.parent = vec![-1];
    ma.ps_morpheme = vec![bos];
}

fn set_ini_tag(infos: &[StrInfo], n_index: usize, klp: &mut KlpState) {
    if !klp.words.is_empty() {
        let prev = &klp.words[klp.words.len() - 1];
        let last = prev
            .morphs
            .last()
            .expect("word must have at least one morph");
        klp.gb_ini_tag = last.ch_tag;
        klp.gsch_ini_str = last.pyogi.clone();
        let ll = if prev.morphs.len() < 2 {
            if klp.words.len() >= 2 {
                klp.words[klp.words.len() - 2].morphs.last()
            } else {
                None
            }
        } else {
            prev.morphs.get(prev.morphs.len() - 2)
        };
        if let Some(ll) = ll {
            klp.gb_ini_ll_tag = ll.ch_tag;
            klp.gsch_ini_ll_str = ll.pyogi.clone();
        }
    }
    if n_index + 1 < infos.len() {
        klp.gb_end_tag = set_tag_from_attribute(infos[n_index + 1].f_char_type);
    } else {
        klp.gb_end_tag = b'l';
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn morph_cand_process(ma: &mut MorphAnalBuf, d: &KmaDicts) {
    ma.pch_pyogi_str = conv_uni_wan_to_pyogi(&ma.pw_korea_str);
    ma.n_morph_cand_cnt = -1;
    ma.b_focus_idx = -1;
    ma.w_pyogi_len = ma.pch_pyogi_str.len() as i16;
    ma.w_fin_pos = ma.w_pyogi_len * 6;
    ma.ps_pyogi_info = vec![PyogiInfo::new(); ma.w_pyogi_len as usize + 2];
    pyogi_info_set(&mut ma.ps_pyogi_info, &ma.pch_pyogi_str);
    set_irregular_predicate(ma, d);
    produce_morpheme_candidate(ma, d);
    set_morpheme_cand_info(ma, d);
}

fn pyogi_info_set(infos: &mut [PyogiInfo], pyogi: &[u8]) {
    let mut pos: i16 = 0;
    for (i, &c) in pyogi.iter().enumerate() {
        let e = &mut infos[i];
        e.w_self_pos = pos;
        pos += 6;
        e.w_next_pos = pos;
        e.ch_eng_pyogi = c;
        if b"ghqndlmbrsvfjzcktp".contains(&c) || b"wy".contains(&c) {
            e.b_start_able_flag = b'o';
            e.b_end_able_flag = b'x';
        } else if b"DJG*CPHQTK".contains(&c) {
            e.b_start_able_flag = b'x';
            e.b_end_able_flag = b'o';
        } else {
            e.b_start_able_flag = b'o';
            e.b_end_able_flag = b'o';
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
fn set_irregular_predicate(ma: &mut MorphAnalBuf, d: &KmaDicts) {
    let w_pyogi_len = ma.w_pyogi_len as usize;
    for e in &mut ma.ps_pyogi_info {
        e.sch_con_status = [b' '; 20];
    }
    for n_len in 1..=5usize {
        let i_var7 = n_len - 1;
        if w_pyogi_len <= i_var7 {
            continue;
        }
        let mut k = 0usize;
        loop {
            let mut stem = Vec::with_capacity(n_len);
            for j in 0..=i_var7 {
                stem.push(ma.ps_pyogi_info[k + j].ch_eng_pyogi);
            }
            if let Some(conds) = d.irr_pred_search(&stem) {
                let local_38 = (n_len + k) as i32;
                let local_3c = k as i32 - 1;
                for (ci, &b) in conds.iter().enumerate() {
                    if ci >= 3 || b == 0xff {
                        break;
                    }
                    let mut set_flag: Option<u8> = None;
                    match b {
                        0 => {
                            if is_back_consonant(local_38, ma) {
                                set_flag = Some(0);
                            }
                        }
                        1 => {
                            if is_not_consonant(local_3c, local_38, ma) {
                                set_flag = Some(1);
                            }
                        }
                        0x0f => {
                            if stem[ci] != b'y' || is_fore_consonant(local_3c, ma) {
                                set_flag = Some(0x0f);
                            }
                        }
                        0x10 | 0x11 => {
                            let mut bb = b;
                            if bb == 0x10 && stem[ci] == b'w' && !is_fore_consonant(local_3c, ma) {
                                bb = 0xff;
                            }
                            if is_back_consonant(local_38, ma) && bb != 0xff {
                                set_flag = Some(bb);
                            }
                        }
                        0x13 => {
                            if is_fore_vowel(local_3c, ma) {
                                set_flag = Some(0x13);
                            }
                        }
                        _ => set_flag = Some(b),
                    }
                    if let Some(f) = set_flag {
                        let entry = (local_38 + tables::GRAPHEME_OFFSET[f as usize] - 1) as usize;
                        if entry < ma.ps_pyogi_info.len() {
                            ma.ps_pyogi_info[entry].sch_con_status[f as usize] = b'o';
                        }
                    }
                }
            }
            k += 1;
            if k + i_var7 >= w_pyogi_len {
                break;
            }
        }
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn is_back_consonant(n_back_pos: i32, ma: &MorphAnalBuf) -> bool {
    if n_back_pos < i32::from(ma.w_pyogi_len) {
        let c = ma.ps_pyogi_info[n_back_pos as usize].ch_eng_pyogi;
        b"wioghqndlmbrsvfjzcktpVy".contains(&c)
    } else {
        true
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn is_fore_consonant(n_fore_pos: i32, ma: &MorphAnalBuf) -> bool {
    if n_fore_pos >= 0 {
        let c = ma.ps_pyogi_info[n_fore_pos as usize].ch_eng_pyogi;
        !b"ghqndlmbrsvfjzcktp".contains(&c)
    } else {
        true
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn is_not_consonant(n_fore_pos: i32, n_back_pos: i32, ma: &MorphAnalBuf) -> bool {
    if n_back_pos < i32::from(ma.w_pyogi_len) {
        if n_fore_pos >= 0 {
            let c = ma.ps_pyogi_info[n_fore_pos as usize].ch_eng_pyogi;
            if b"ghqndlmbrsvfjzcktp".contains(&c) {
                return false;
            }
        }
        let c = ma.ps_pyogi_info[n_back_pos as usize].ch_eng_pyogi;
        if b"NnBoL".contains(&c) {
            return true;
        }
        if n_back_pos + 1 < i32::from(ma.w_pyogi_len) && c == b's' {
            let c2 = ma.ps_pyogi_info[(n_back_pos + 1) as usize].ch_eng_pyogi;
            return b"iy9".contains(&c2);
        }
    }
    false
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn is_fore_vowel(n_fore_pos: i32, ma: &MorphAnalBuf) -> bool {
    if n_fore_pos >= 0 {
        let c = ma.ps_pyogi_info[n_fore_pos as usize].ch_eng_pyogi;
        b"aeoui_89".contains(&c)
    } else {
        true
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn produce_morpheme_candidate(ma: &mut MorphAnalBuf, d: &KmaDicts) {
    let s_var2 = get_first_start_pos_from_endof_word(ma.w_pyogi_len as usize, ma);
    focus_info_set(
        ma,
        s_var2,
        ma.w_fin_pos,
        ma.w_pyogi_len - 1,
        0,
        ma.w_fin_pos,
        b'R',
    );
    add_morph_cand(
        ma,
        ma.w_fin_pos,
        ma.w_fin_pos,
        ma.w_fin_pos,
        &[-1],
        b"",
        b'k',
        1.0,
    );
    let mut sch_word: Vec<u8> = Vec::new();
    let mut sch_pyogi: Vec<u8> = Vec::new();
    loop {
        get_morph_pyogi(&mut sch_pyogi, ma);
        if is_change_ium_to(&sch_pyogi) {
            let end = ma.ps_morph_cand[ma.n_morph_cand_cnt as usize].w_end_pos;
            get_unknoun_pyogi(&mut sch_word, ma, end);
            if sch_pyogi == b"k9" || is_closed_syllable_k_root(&sch_word) {
                process_special_unk_noun(&mut sch_word, ma, d);
            }
        }
        let n = search_kma_dict(ma, d, &sch_pyogi);
        add_process_morph_cand_list(ma, d, &sch_pyogi, n);
        if ma.s_focus_range[ma.b_focus_idx as usize].w_fin_pos == ma.w_fin_pos {
            let ini = ma.s_focus_range[ma.b_focus_idx as usize].w_ini_pos;
            let n_pos = get_phone_start_pos(ini, ma);
            irr_predicate_retrieve(ma, d, n_pos);
        }
        if !decide_new_focus_pos(b'G', ma) {
            break;
        }
    }
    process_pronoun(ma, d);
    let mut root: i16 = -1;
    let mut links = vec![0i16; 20];
    get_link_morph_idx(
        ma,
        d,
        ma.n_morph_cand_cnt,
        &mut root,
        &mut links,
        0,
        ma.gb_ini_tag,
        true,
    );
    add_morph_cand(ma, 0, 0, root, &links, b"", b'0', 1.0);
}

#[expect(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn get_first_start_pos_from_endof_word(n_len: usize, ma: &MorphAnalBuf) -> i16 {
    let mut i = n_len as isize - 1;
    if i < 0 {
        return 0;
    }
    loop {
        let info = &ma.ps_pyogi_info[i as usize];
        if info.b_start_able_flag == b'o' {
            return info.w_self_pos;
        }
        i -= 1;
        if i < 0 {
            return 0;
        }
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
const fn focus_info_set(
    ma: &mut MorphAnalBuf,
    w_start_pos: i16,
    w_fin_pos: i16,
    w_pyogi_idx: i16,
    w_word_start_pos: i16,
    w_word_end_pos: i16,
    ch_dic_search_type: u8,
) {
    let c1 = ma.b_focus_idx + 1;
    if c1 > 2 {
        return;
    }
    ma.b_focus_idx = c1;
    let f = &mut ma.s_focus_range[c1 as usize];
    f.w_ini_pos = w_start_pos;
    f.w_fin_pos = w_fin_pos;
    f.w_pyogi_cnt = w_pyogi_idx;
    f.m_w_word_start_pos = w_word_start_pos;
    f.m_w_word_end_pos = w_word_end_pos;
    f.f_search_direct = ch_dic_search_type;
}

#[expect(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn get_morph_pyogi(out: &mut Vec<u8>, ma: &MorphAnalBuf) {
    let b = ma.b_focus_idx as usize;
    let w_pyogi_cnt = ma.s_focus_range[b].w_pyogi_cnt as usize;
    out.clear();
    if ma.s_focus_range[b].f_search_direct == b'L' {
        let mut local_26 = ma.s_focus_range[b].w_fin_pos;
        loop {
            if local_26 == ma.s_focus_range[b].w_ini_pos {
                out.reverse();
                return;
            }
            let mut found: Option<usize> = None;
            let mut i = w_pyogi_cnt as isize;
            while i >= 0 {
                if ma.ps_pyogi_info[i as usize].w_next_pos == local_26 {
                    found = Some(i as usize);
                    break;
                }
                i -= 1;
            }
            let Some(i) = found else { return };
            out.push(ma.ps_pyogi_info[i].ch_eng_pyogi);
            local_26 = ma.ps_pyogi_info[i].w_self_pos;
        }
    }
    let mut s_var3 = ma.s_focus_range[b].w_ini_pos;
    let fin = ma.s_focus_range[b].w_fin_pos;
    while s_var3 != fin {
        let mut found: Option<usize> = None;
        let mut i = w_pyogi_cnt as isize;
        while i >= 0 {
            if ma.ps_pyogi_info[i as usize].w_self_pos == s_var3 {
                found = Some(i as usize);
                break;
            }
            i -= 1;
        }
        let Some(i) = found else { return };
        out.push(ma.ps_pyogi_info[i].ch_eng_pyogi);
        s_var3 = ma.ps_pyogi_info[i].w_next_pos;
    }
}

fn is_change_ium_to(s: &[u8]) -> bool {
    s == b"ci" || s == b"k9"
}

fn is_closed_syllable_k_root(s: &[u8]) -> bool {
    if s.is_empty() {
        return false;
    }
    let c = s[s.len() - 1];
    c.wrapping_add(0xbf) < 0x1a || c == b'*'
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn decide_new_focus_pos(ch_decide_type: u8, ma: &mut MorphAnalBuf) -> bool {
    let c = ma.b_focus_idx as usize;
    let mut local_2e = ma.s_focus_range[c].w_ini_pos;
    let word_start = ma.s_focus_range[c].m_w_word_start_pos;
    let word_end = ma.s_focus_range[c].m_w_word_end_pos;
    if local_2e == word_start && ma.s_focus_range[c].w_fin_pos == word_end {
        return false;
    }
    let mut s_var3: i16;
    if ch_decide_type == b'L' {
        s_var3 = new_morph_start_pos(ma, ma.s_focus_range[c].w_pyogi_cnt, local_2e, word_start);
        ma.s_focus_range[c].w_ini_pos = s_var3;
        return true;
    } else if ch_decide_type == b'R' {
        s_var3 = new_morph_end_pos(
            ma,
            ma.s_focus_range[c].w_pyogi_cnt,
            ma.s_focus_range[c].w_fin_pos,
            word_end,
        );
    } else if ch_decide_type == b'G' {
        let w_curr_pos = ma.s_focus_range[c].w_fin_pos;
        if w_curr_pos == word_end {
            local_2e =
                new_morph_start_pos(ma, ma.s_focus_range[c].w_pyogi_cnt, local_2e, word_start);
            s_var3 = new_morph_end_pos(ma, ma.s_focus_range[c].w_pyogi_cnt, local_2e, word_end);
        } else {
            s_var3 = new_morph_end_pos(ma, ma.s_focus_range[c].w_pyogi_cnt, w_curr_pos, word_end);
        }
    } else {
        s_var3 = i16::from(ch_decide_type);
        local_2e = s_var3;
    }
    loop {
        let local_1e = ma.s_focus_range[c].m_w_word_end_pos;
        if local_1e == s_var3 {
            ma.s_focus_range[c].w_ini_pos = local_2e;
            ma.s_focus_range[c].w_fin_pos = local_1e;
            return true;
        }
        let mut hit = false;
        let mut i = ma.n_morph_cand_cnt;
        while i >= 0 {
            let cand = &ma.ps_morph_cand[i as usize];
            if cand.w_start_pos == s_var3 && cand.w_root_pos == ma.w_fin_pos {
                hit = true;
                break;
            }
            i -= 1;
        }
        if hit {
            ma.s_focus_range[c].w_ini_pos = local_2e;
            ma.s_focus_range[c].w_fin_pos = s_var3;
            return true;
        }
        s_var3 = new_morph_end_pos(ma, ma.s_focus_range[c].w_pyogi_cnt, s_var3, local_1e);
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn new_morph_start_pos(
    ma: &MorphAnalBuf,
    w_pyogi_cnt: i16,
    mut w_curr_pos: i16,
    w_start_limit_pos: i16,
) -> i16 {
    loop {
        let mut found: Option<usize> = None;
        if w_pyogi_cnt >= 0 {
            let mut i = w_pyogi_cnt as isize;
            while i >= 0 {
                if ma.ps_pyogi_info[i as usize].w_next_pos == w_curr_pos {
                    found = Some(i as usize);
                    break;
                }
                i -= 1;
            }
            if let Some(i) = found {
                w_curr_pos = ma.ps_pyogi_info[i].w_self_pos;
            }
        }
        if w_curr_pos == w_start_limit_pos {
            return w_curr_pos;
        }
        if let Some(i) = found {
            if ma.ps_pyogi_info[i].b_start_able_flag == b'o' {
                return w_curr_pos;
            }
        } else {
            return w_curr_pos;
        }
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn new_morph_end_pos(
    ma: &MorphAnalBuf,
    w_pyogi_cnt: i16,
    mut w_curr_pos: i16,
    w_end_limit_pos: i16,
) -> i16 {
    loop {
        let mut found: Option<usize> = None;
        if w_pyogi_cnt >= 0 {
            let mut i = w_pyogi_cnt as isize;
            while i >= 0 {
                if ma.ps_pyogi_info[i as usize].w_self_pos == w_curr_pos {
                    found = Some(i as usize);
                    break;
                }
                i -= 1;
            }
            if let Some(i) = found {
                w_curr_pos = ma.ps_pyogi_info[i].w_next_pos;
            }
        }
        if w_curr_pos == w_end_limit_pos {
            return w_curr_pos;
        }
        if let Some(i) = found {
            if ma.ps_pyogi_info[i].b_end_able_flag == b'o' {
                return w_curr_pos;
            }
        } else {
            return w_curr_pos;
        }
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn get_phone_start_pos(w_pos: i16, ma: &MorphAnalBuf) -> i32 {
    let mut i = i32::from(ma.s_focus_range[ma.b_focus_idx as usize].w_pyogi_cnt);
    if i >= 0 && ma.ps_pyogi_info[i as usize].w_self_pos != w_pos {
        i -= 1;
        while i >= 0 && ma.ps_pyogi_info[i as usize].w_self_pos != w_pos {
            i -= 1;
        }
    }
    i
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "C port: index/math casts with wrap semantics"
)]
fn search_kma_dict(ma: &mut MorphAnalBuf, d: &KmaDicts, key: &[u8]) -> i32 {
    let v = d.search_kma_dict(key);
    let n = v.len() as i32;
    ma.gs_k_anal_info = v;
    n
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn add_process_morph_cand_list(ma: &mut MorphAnalBuf, d: &KmaDicts, pch_string: &[u8], n: i32) {
    ma.gsch_morph_cand_str = pch_string.to_vec();
    ma.gn_morph_cand_str = pch_string.len().saturating_sub(1);
    let b_focus = ma.b_focus_idx as usize;
    let w_fin_pos = ma.s_focus_range[b_focus].w_fin_pos;
    let w_ini_pos = ma.s_focus_range[b_focus].w_ini_pos;
    for i in 0..n {
        let info = ma.gs_k_anal_info[i as usize].clone();
        if info.irr_type == b'T' {
            let mut root: i16 = -1;
            let mut links = vec![0i16; 20];
            get_link_morph_idx(
                ma,
                d,
                ma.n_morph_cand_cnt,
                &mut root,
                &mut links,
                w_fin_pos,
                info.ch_pumsa,
                false,
            );
            add_morph_cand_proc(
                ma, w_ini_pos, w_fin_pos, root, &links, pch_string, &info, false,
            );
        } else if info.irr_type == b'R' {
            let mut c5 = ma.b_irr_number + 1;
            if c5 > 29 {
                c5 = ma.b_irr_number;
            }
            ma.b_irr_number = c5;
            let e = IregulerStr {
                pyogi: pch_string.to_vec(),
                irr_string: info.irr_string.clone(),
            };
            let idx = c5 as usize;
            if idx < ma.pps_ireguler.len() {
                ma.pps_ireguler[idx] = e;
            } else {
                ma.pps_ireguler.push(e);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn add_morph_cand(
    ma: &mut MorphAnalBuf,
    w_start: i16,
    w_end: i16,
    w_root_morph_pos: i16,
    pw_link_morph_idx: &[i16],
    pch_morph_str: &[u8],
    ch_tag: u8,
    dbl_prob: f64,
) {
    let info = KAnalInfo {
        irr_type: b'T',
        ch_pumsa: ch_tag,
        ch_con_type: b'0',
        d_part_prob: 0.0,
        d_word_prob: dbl_prob,
        un_to_info: 0,
        irr_string: Vec::new(),
    };
    let links: Vec<i16> = if pw_link_morph_idx.first() == Some(&-1) {
        Vec::new()
    } else {
        pw_link_morph_idx.to_vec()
    };
    add_morph_cand_proc(
        ma,
        w_start,
        w_end,
        w_root_morph_pos,
        &links,
        pch_morph_str,
        &info,
        true,
    );
}

#[allow(clippy::too_many_arguments)]
fn add_morph_cand_proc(
    ma: &mut MorphAnalBuf,
    w_start: i16,
    w_end: i16,
    w_root_morph_pos: i16,
    pw_link_morph_idx: &[i16],
    pch_morph_str: &[u8],
    ps_kanal_dic_info: &KAnalInfo,
    b_retrieved: bool,
) {
    let mut cand = MorphCand::new();
    cand.w_start_pos = w_start;
    cand.w_end_pos = w_end;
    cand.w_root_pos = w_root_morph_pos;
    cand.sw_link_morph_idx = pw_link_morph_idx.to_vec();
    cand.sch_morph_str = pch_morph_str.to_vec();
    cand.ch_pumsa = ps_kanal_dic_info.ch_pumsa;
    cand.d_probability = ps_kanal_dic_info.d_word_prob;
    cand.d_part_prob = ps_kanal_dic_info.d_part_prob;
    cand.b_retrieved = b_retrieved;
    if tables::is_to(cand.ch_pumsa) {
        cand.un_to_info = ps_kanal_dic_info.un_to_info;
    }
    ma.ps_morph_cand.push(cand);
    ma.n_morph_cand_cnt += 1;
}

#[allow(clippy::too_many_arguments)]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn get_link_morph_idx(
    ma: &mut MorphAnalBuf,
    d: &KmaDicts,
    n_curr_morph_cnt: i32,
    pw_root_morph_pos: &mut i16,
    pw_link_morph_idx: &mut Vec<i16>,
    w_new_morph_end: i16,
    ch_new_tag: u8,
    b_new: bool,
) {
    let mut local_52: i16 = -1;
    let mut n_count = 0usize;
    let mut links_out: Vec<i16> = Vec::new();
    let mut i = n_curr_morph_cnt;
    while i >= 0 {
        let cand = ma.ps_morph_cand[i as usize].clone();
        if cand.w_start_pos == w_new_morph_end {
            let d_var12 = d.pos_bigram(ch_new_tag, cand.ch_pumsa);
            if (d_var12 == 0.0 && ch_new_tag != b'j')
                || cand.w_root_pos == -1
                || !check_neighbour_morpheme(ch_new_tag, cand.ch_pumsa, &cand.sch_morph_str)
            {
                i -= 1;
                continue;
            }
            if cand.ch_pumsa == b'm' {
                let last = ma
                    .gsch_morph_cand_str
                    .get(ma.gn_morph_cand_str)
                    .copied()
                    .unwrap_or(0);
                if !b"NDLMBSJG*CPHQVTK".contains(&last) {
                    i -= 1;
                    continue;
                }
                if tables::is_k_voice_yong_yon(ch_new_tag) {
                    let mut keep: Vec<i16> = Vec::new();
                    for &l in &cand.sw_link_morph_idx {
                        if l == -1 {
                            break;
                        }
                        let tgt = &ma.ps_morph_cand[l as usize];
                        if !tables::is_k_yongon_to(tgt.ch_pumsa) {
                            continue;
                        }
                        let u = tgt.un_to_info;
                        let str_is_ebs = ma.gsch_morph_cand_str == b"eBS";
                        let str_is_iv = ma.gsch_morph_cand_str == b"iV";
                        let cond1 = (u & 0x100) != 0;
                        let cond2 = !str_is_ebs && !str_is_iv;
                        let ch_is_cd = matches!(ch_new_tag, b'C' | b'D');
                        let cond3 = !ch_is_cd;
                        let cond4 = str_is_ebs || u >= 0x80;
                        let cond5 = (ch_new_tag == b'B' || ch_new_tag == b'@')
                            && last != b'H'
                            && (u & 0x40) == 0;
                        if (cond1 || (cond2 && (cond3 || cond4))) && !cond5 {
                            keep.push(l);
                        }
                    }
                    if keep.is_empty() {
                        i -= 1;
                        continue;
                    }
                    keep.push(-1);
                    ma.ps_morph_cand[i as usize].sw_link_morph_idx = keep;
                }
            }
            if tables::is_k_cheon_part(ch_new_tag)
                && tables::is_to(cand.ch_pumsa)
                && !is_combinable_cheon_to(&ma.gsch_morph_cand_str, b_new, &cand)
            {
                i -= 1;
                continue;
            }
            if tables::is_k_voice_yong_yon(ch_new_tag)
                && tables::is_k_yongon_to(cand.ch_pumsa)
                && !is_combinable_yongon_to(
                    &ma.gsch_morph_cand_str,
                    ch_new_tag,
                    b_new,
                    &cand,
                    &ma.pch_pyogi_str,
                    d,
                )
            {
                i -= 1;
                continue;
            }
            if tables::is_k_yongon_to(cand.ch_pumsa)
                && ch_new_tag == b'b'
                && (cand.un_to_info & 0x200) == 0
            {
                i -= 1;
                continue;
            }
            if n_count > 0x12 {
                break;
            }
            links_out.push(i as i16);
            n_count += 1;
            if cand.w_root_pos > local_52 {
                local_52 = cand.w_root_pos;
            }
        }
        i -= 1;
    }
    links_out.push(-1);
    *pw_link_morph_idx = links_out;
    *pw_root_morph_pos = local_52;
}

#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
fn check_neighbour_morpheme(b_l_tag: u8, b_r_tag: u8, pch_r_str: &[u8]) -> bool {
    if tables::is_k_cheon_part(b_l_tag) {
        if b_r_tag == b'7' && bsearch_str(pch_r_str, &tables::CHEON_NO_LINKABLE) {
            return false;
        }
    } else if b_r_tag == b'7' && b_l_tag == b'_' {
        return !bsearch_str(pch_r_str, &tables::YONGYON_NO_LINKABLE);
    }
    if b_l_tag == b'c' && pch_r_str.starts_with(b"n_N") {
        return pch_r_str.get(3).copied().unwrap_or(0) != 0;
    }
    true
}

fn bsearch_str(key: &[u8], table: &[&[u8]]) -> bool {
    let mut lo = 0usize;
    let mut hi = table.len();
    while lo < hi {
        let mid = usize::midpoint(lo, hi);
        match key.cmp(table[mid]) {
            std::cmp::Ordering::Less => hi = mid,
            std::cmp::Ordering::Greater => lo = mid + 1,
            std::cmp::Ordering::Equal => return true,
        }
    }
    false
}

fn is_combinable_cheon_to(pch_word: &[u8], b_new: bool, cand: &MorphCand) -> bool {
    if b_new {
        return true;
    }
    let f_close = is_closed_syllable_k_root(pch_word);
    if tables::is_k_cheon_to(cand.ch_pumsa) {
        let u = cand.un_to_info;
        if u != 0 {
            if f_close && (u & 0x400) != 0 {
                if cand.ch_pumsa != b'Y' {
                    return false;
                }
                return pch_word.last().copied().unwrap_or(0) == b'L';
            }
            let xor = (u >> 0xb) & 1;
            return u32::from(!f_close) ^ xor != 0;
        }
        return true;
    }
    if tables::is_k_yongon_to(cand.ch_pumsa) {
        if (cand.un_to_info & 0x20) == 0 {
            return false;
        }
        if f_close {
            let c = cand.sch_morph_str.first().copied().unwrap_or(0);
            return !b"bcdfghjklmnpqrstvxz".contains(&c);
        }
    }
    true
}

#[allow(clippy::too_many_arguments)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
fn is_combinable_yongon_to(
    gsch: &[u8],
    ch_new_tag: u8,
    b_new: bool,
    cand: &MorphCand,
    pch_pyogi_str: &[u8],
    _d: &KmaDicts,
) -> bool {
    if b_new {
        return true;
    }
    let c_var1 = gsch.last().copied().unwrap_or(0);
    let f_close = is_closed_syllable_k_root(gsch);
    let mut u_var4 = cand.un_to_info;
    if (u_var4 & 0x100) == 0 && (gsch == b"eBS" || gsch == b"iV") {
        return false;
    }
    let local_180 = u_var4 as i8;
    if ((ch_new_tag == b'C' || ch_new_tag == b'D') && gsch != b"eBS" && local_180 >= 0)
        || ((ch_new_tag == b'B' || ch_new_tag == b'@') && c_var1 != b'H' && (u_var4 & 0x40) == 0)
    {
        return false;
    }
    let mut sw_kr = conv_pyogi_to_uni_wan(gsch);
    sw_kr.push(0xb2e4);
    let mut sb_y_num_array = [0u8; 2];
    let mut n_size = 0i32;
    if !get_yong_yon_num(&sw_kr, &mut sb_y_num_array, &mut n_size) {
        return true;
    }
    let b_var2 = if ch_new_tag != b'@' && (ch_new_tag != b'C' || n_size != 1) {
        sb_y_num_array[1]
    } else {
        sb_y_num_array[0]
    };
    u_var4 = cand.un_to_info;
    if (u_var4 & 1) != 0 {
        return true;
    }
    if (u_var4 & 2) != 0 {
        if is_bad_tim_to(cand.ch_pumsa, &cand.sch_morph_str) {
            if f_close && c_var1 != b'L' {
                if (b_var2.wrapping_sub(0x34)) < 8 && gsch == b"i" {
                    return gsch.get(3).copied().unwrap_or(0) == b'n';
                }
                return false;
            }
            return true;
        }
        if cand.sch_morph_str == b"n_Nda" || cand.sch_morph_str.first() == Some(&b's') {
            return f_close;
        }
        u_var4 = cand.un_to_info;
    }
    if (u_var4 & 4) == 0 {
        if (u_var4 & 8) == 0 {
            if (u_var4 & 0x10) != 0 {
                if !f_close {
                    return cand.sch_morph_str.first() != Some(&b'_');
                }
                if cand.sch_morph_str.first() != Some(&b'_') {
                    return b"LBH".contains(&c_var1);
                }
            }
            return true;
        }
        let mut sch_comp = gsch.to_vec();
        sch_comp.extend_from_slice(&cand.sch_morph_str);
        if ((b_var2.wrapping_sub(0x1b)) < 2 || b_var2 == b'7')
            && cand.sch_morph_str.starts_with(b"_N")
        {
            return !contains_subslice(pch_pyogi_str, &sch_comp);
        }
        if ((b_var2.wrapping_sub(0x1d)) < 2 || b_var2 == b'<') || b_var2 == b'=' {
            return !contains_subslice(pch_pyogi_str, &sch_comp);
        }
        if !f_close {
            return cand.sch_morph_str.first() != Some(&b'_');
        }
        if cand.sch_morph_str.first() != Some(&b'_') {
            return b"BH".contains(&c_var1);
        }
        return true;
    }
    match b_var2 {
        0x01 | 0x09 | 0x0a | 0x0d | 0x10 | 0x16 | 0x19 | 0x1b | 0x1d | b'$' | b'*' | b','
        | b'-' | b'/' | b'4' | b'7' | b'8' | b'9' | b':' | b';' | b'<' => {
            if cand.sch_morph_str.first() == Some(&b'e') {
                return false;
            }
            return !cand.sch_morph_str.starts_with(b"ye");
        }
        0x02 => {
            if cand.sch_morph_str.first() == Some(&b'a') || gsch == b"maJse" {
                return true;
            }
        }
        0x03 | 0x06 | 0x14 | b'!' | b'2' => {
            return cand.sch_morph_str.first() != Some(&b'a');
        }
        0x04 | 0x07 | 0x08 | 0x0f => {
            if cand.sch_morph_str.first() == Some(&b'a') {
                return false;
            }
            if cand.sch_morph_str.first() == Some(&b'j') {
                return cand.sch_morph_str.get(1) != Some(&b'i');
            }
        }
        0x05 | 0x0b | b'5' => {
            return !cand.sch_morph_str.starts_with(b"ye");
        }
        0x0c | 0x11 | 0x12 | 0x13 | 0x17 | 0x18 | 0x1a | 0x1c | 0x1e | 0x1f | b' ' | b'%'
        | b'+' | b'.' | b'0' | b'1' | b'6' | b'=' => {
            if cand.sch_morph_str.first() == Some(&b'a') {
                return false;
            }
            return !cand.sch_morph_str.starts_with(b"ye");
        }
        b'#' | b'&' | b'(' => {
            let c0 = cand.sch_morph_str.first().copied().unwrap_or(0);
            return c0 != b'e' && c0 != b'a';
        }
        _ => return true,
    }
    true
}

fn contains_subslice(hay: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    hay.windows(needle.len()).any(|w| w == needle)
}

fn is_bad_tim_to(b_tag: u8, pch_str: &[u8]) -> bool {
    if tables::is_to(b_tag) {
        let c = pch_str.first().copied().unwrap_or(0);
        return c.wrapping_add(0xbf) < 0x1a;
    }
    false
}

fn get_yong_yon_num(pw_vstr: &[u16], pb_array: &mut [u8; 2], pn_size: &mut i32) -> bool {
    if pw_vstr.len() < 2 {
        return false;
    }
    if except_yong_yon(pw_vstr, pb_array, pn_size) {
        return true;
    }
    let b1 = get_verb_num(pw_vstr, pb_array, pn_size);
    let b2 = get_hong_yong_sa_num(pw_vstr, pb_array, pn_size);
    b1 || b2
}

fn except_yong_yon(pw_vstr: &[u16], pb_array: &mut [u8; 2], pn_size: &mut i32) -> bool {
    if pw_vstr.len() < 2 {
        return false;
    }
    let w_var1 = pw_vstr[pw_vstr.len() - 2];
    match w_var1 {
        0xc54a => {
            yong_yon_add(pb_array, pn_size, 0x0d);
            yong_yon_add(pb_array, pn_size, b',');
            true
        }
        0xc788 => {
            yong_yon_add(pb_array, pn_size, 0x1f);
            true
        }
        0xc5c6 => {
            yong_yon_add(pb_array, pn_size, b' ');
            true
        }
        0xc8b8 => {
            yong_yon_add(pb_array, pn_size, b'*');
            true
        }
        0xd1b1 => {
            yong_yon_add(pb_array, pn_size, 0x17);
            true
        }
        _ => {
            if pw_vstr.len() > 2 {
                let w_var4 = pw_vstr[pw_vstr.len() - 3];
                if w_var4 == 0xacc4 && w_var1 == 0xc2dc {
                    yong_yon_add(pb_array, pn_size, b'!');
                    return true;
                }
            }
            false
        }
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
const fn yong_yon_add(pb_array: &mut [u8; 2], pn_size: &mut i32, b_byte: u8) -> bool {
    if *pn_size < 2 {
        pb_array[*pn_size as usize] = b_byte;
        *pn_size += 1;
    }
    true
}

#[allow(clippy::if_same_then_else)]
#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
#[expect(
    clippy::branches_sharing_code,
    reason = "C port: shared code in branches kept as-is (extraction would change control flow)"
)]
fn get_verb_num(pw_vstr: &[u16], pb_array: &mut [u8; 2], pn_size: &mut i32) -> bool {
    let s_var2 = pw_vstr.len();
    let (w_wan, w_cx1) = match s_var2 {
        0 | 1 => (0, 0),
        2 => (pw_vstr[0], 0),
        _ => (pw_vstr[s_var2 - 2], pw_vstr[s_var2 - 3]),
    };
    let kx0 = crate::code::conv_uni_code_to_cvc(w_wan);
    let kx1 = crate::code::conv_uni_code_to_cvc(w_cx1);
    let b_byte: Option<u8> = if kx0[2] == 0 {
        if kx0[1] == 1 {
            if w_wan == 0xac00 {
                Some(0x09)
            } else if w_wan == 0xd558 {
                Some(0x14)
            } else {
                Some(0x01)
            }
        } else if kx0[1] == 5 {
            if (w_cx1 == 0xadf8 && w_wan == 0xb7ec) || (w_cx1 == 0xc5b4 && w_wan == 0xca4c) {
                Some(0x15)
            } else {
                Some(0x02)
            }
        } else if matches!(kx0[1], 7 | 2 | 6 | 12) {
            Some(0x03)
        } else if (kx0[1].wrapping_sub(0x10)) < 2 || kx0[1] == 0x14 {
            Some(0x04)
        } else if kx0[1] == 9 {
            if w_wan == 0xc624 {
                Some(0x0a)
            } else {
                Some(0x05)
            }
        } else if kx0[1] == 0x12 {
            Some(0x06)
        } else if kx0[1] == 0x0e {
            if w_wan == 0xd478 {
                Some(0x13)
            } else {
                Some(0x07)
            }
        } else if kx0[1] == 0x15 {
            Some(0x08)
        } else if kx0[1] == 0x13 {
            if w_wan == 0xb974 {
                if w_cx1 == 0xc774 {
                    Some(0x12)
                } else if w_cx1 == 0xb530 {
                    Some(0x0e)
                } else if w_cx1 == 0xb7ec {
                    Some(0x0f)
                } else if w_cx1 == 0xce58 {
                    Some(0x07)
                } else if tables::check_chosong(kx1[1]) == 1 {
                    Some(0x10)
                } else {
                    Some(0x11)
                }
            } else if w_wan == 0xadf8 {
                Some(0x07)
            } else if w_wan == 0xc4f0 || s_var2 == 2 {
                Some(0x0f)
            } else if tables::check_chosong(kx1[1]) == 1 {
                Some(0x0e)
            } else if tables::check_chosong(kx1[1]) == 2 {
                Some(0x0f)
            } else {
                return false;
            }
        } else {
            return false;
        }
    } else if kx0[2] == 0x11 {
        if w_wan == 0xb3d5 {
            Some(0x16)
        } else if w_wan == 0xcb59 || w_wan == 0xbd59 || w_wan == 0xc635 {
            Some(0x18)
        } else if w_wan == 0xc90d {
            Some(0x0c)
        } else if w_wan == 0xaf3d {
            if w_cx1 == 0xb2c8 {
                Some(0x0b)
            } else {
                return false;
            }
        } else if matches!(kx0[0], 9 | 12 | 11 | 13) {
            Some(0x0b)
        } else if matches!(kx0[0], 3 | 1 | 4 | 6 | 7 | 8 | 10 | 15 | 16 | 18 | 14 | 2) {
            Some(0x17)
        } else {
            return false;
        }
    } else if kx0[2] == 7 {
        if w_wan == 0xb2eb {
            if w_cx1 == 0xae68 {
                Some(0x19)
            } else {
                yong_yon_add(pb_array, pn_size, 0x0b);
                Some(0x19)
            }
        } else if w_wan == 0xbb3b || w_wan == 0xac77 {
            yong_yon_add(pb_array, pn_size, 0x0c);
            Some(0x1a)
        } else if matches!(
            w_wan,
            0xb3cb | 0xad73 | 0xbc8b | 0xb51b | 0xbbff | 0xb72f | 0xbed7 | 0xbc1b | 0xc5bb | 0xc3df
        ) {
            Some(0x0b)
        } else if matches!(
            w_wan,
            0xae37 | 0xceeb | 0xb20b | 0xacaf | 0xbd87 | 0xb4e3 | 0xc2e3
        ) {
            if tables::check_chosong(kx0[1]) == 1 {
                Some(0x19)
            } else {
                Some(0x1a)
            }
        } else {
            return false;
        }
    } else if kx0[2] == 0x13 {
        if matches!(
            w_wan,
            0xbc27
                | 0xae43
                | 0xbe57
                | 0xbc97
                | 0xc2ef
                | 0xc19f
                | 0xcacf
                | 0xc53b
                | 0xb057
                | 0xaf3f
                | 0xc557
                | 0xbe8f
                | 0xc6c3
        ) {
            if tables::check_chosong(kx0[1]) == 1 {
                Some(0x0b)
            } else {
                Some(0x0c)
            }
        } else if matches!(
            w_wan,
            0xb0ab
                | 0xae0b
                | 0xbb47
                | 0xbd93
                | 0xc90f
                | 0xc22b
                | 0xc9d3
                | 0xc813
                | 0xc787
                | 0xc7a3
                | 0xbabb
        ) {
            if tables::check_chosong(kx0[1]) == 1 {
                Some(0x1b)
            } else {
                Some(0x1c)
            }
        } else {
            return false;
        }
    } else if kx0[2] == 8 {
        if tables::check_chosong(kx0[1]) == 1 {
            Some(0x1d)
        } else {
            Some(0x1e)
        }
    } else if kx0[2] == 0x14 {
        Some(0x0c)
    } else if kx0[2] == 6 && kx0[1] == 1 && (kx0[0] == 7 || kx0[0] == 15) {
        Some(0x0d)
    } else if w_wan == 0xbc49 {
        Some(0x0b)
    } else if tables::check_chosong(kx0[1]) == 1 {
        Some(0x0b)
    } else {
        Some(0x0c)
    };
    b_byte.is_some_and(|b| yong_yon_add(pb_array, pn_size, b))
}

#[allow(clippy::if_same_then_else)]
#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
fn get_hong_yong_sa_num(pw_vstr: &[u16], pb_array: &mut [u8; 2], pn_size: &mut i32) -> bool {
    let s_var2 = pw_vstr.len();
    let (w_cx0, w_wan) = match s_var2 {
        0 | 1 => (0, 0),
        2 => (pw_vstr[0], 0),
        _ => (pw_vstr[s_var2 - 2], pw_vstr[s_var2 - 3]),
    };
    let kx0 = crate::code::conv_uni_code_to_cvc(w_cx0);
    let kx1 = crate::code::conv_uni_code_to_cvc(w_wan);
    let b: Option<u8> = if kx0[2] == 0 {
        if kx0[1] == 1 {
            if w_cx0 == 0xd558 {
                Some(b'2')
            } else {
                Some(b'$')
            }
        } else if kx0[1] == 2 || kx0[1] == 6 {
            Some(b'%')
        } else if kx0[1] == 0x11 || kx0[1] == 0x14 {
            Some(b'&')
        } else if kx0[1] == 9 {
            Some(b'\'')
        } else if kx0[1] == 0x12 {
            Some(b'(')
        } else if kx0[1] == 0x15 {
            Some(b')')
        } else if kx0[1] == 0x13 {
            let cc = tables::check_chosong(kx1[1]);
            if w_cx0 == 0xb974 {
                if matches!(w_wan, 0xb178 | 0xd478 | 0xb204) {
                    Some(b'1')
                } else if cc == 1 {
                    if w_wan == 0 {
                        Some(b'/')
                    } else if w_wan == 0xc5b4 {
                        Some(b'0')
                    } else {
                        return false;
                    }
                } else if cc == 2 {
                    Some(b'0')
                } else if w_wan == 0 {
                    Some(b'/')
                } else {
                    return false;
                }
            } else if cc == 1 {
                if w_wan == 0 {
                    Some(b'-')
                } else {
                    return false;
                }
            } else if cc == 2 {
                Some(b'.')
            } else if w_wan == 0 {
                Some(b'.')
            } else {
                return false;
            }
        } else if w_wan == 0xc5b4 && w_cx0 == 0xca4c {
            Some(b'3')
        } else {
            return false;
        }
    } else if w_wan == 0xb0ab {
        Some(b'7')
    } else if kx0[2] == 0x1b {
        let cc = tables::check_chosong(kx0[1]);
        if cc == 1 {
            if kx0[1] == 3 { Some(b'9') } else { Some(b'8') }
        } else if kx0[1] == 7 {
            Some(b';')
        } else {
            Some(b':')
        }
    } else if kx0[2] == 8 {
        if tables::check_chosong(kx0[1]) == 1 {
            Some(b'<')
        } else {
            Some(b'=')
        }
    } else if kx0[2] == 6 && kx0[1] == 1 && (kx0[0] == 7 || kx0[0] == 15) {
        Some(b',')
    } else if w_wan == 0xacf1 {
        Some(b'4')
    } else if w_wan == 0xc5b4 {
        if w_cx0 == 0xc90d {
            Some(b'+')
        } else if matches!(kx0[0], 9 | 12 | 13) {
            if tables::check_chosong(kx0[1]) == 1 {
                Some(b'*')
            } else {
                Some(b'+')
            }
        } else if matches!(
            kx0[0],
            3 | 1 | 4 | 6 | 7 | 8 | 10 | 15 | 16 | 18 | 14 | 2 | 17 | 19
        ) {
            if tables::check_chosong(kx0[1]) == 1 {
                Some(b'5')
            } else {
                Some(b'+')
            }
        } else {
            return false;
        }
    } else if w_wan == 0xb2c8 {
        if w_cx0 == 0xaf3d {
            Some(b'4')
        } else if matches!(kx0[0], 9 | 12 | 13) {
            if tables::check_chosong(kx0[1]) == 1 {
                Some(b'*')
            } else {
                Some(b'+')
            }
        } else if matches!(
            kx0[0],
            3 | 1 | 4 | 6 | 7 | 8 | 10 | 15 | 16 | 18 | 14 | 2 | 17 | 19
        ) {
            if tables::check_chosong(kx0[1]) == 1 {
                Some(b'5')
            } else {
                Some(b'+')
            }
        } else {
            return false;
        }
    } else if w_wan == 0xc5fd {
        Some(b'6')
    } else if matches!(kx0[0], 9 | 12 | 13) {
        if tables::check_chosong(kx0[1]) == 1 {
            Some(b'*')
        } else {
            Some(b'+')
        }
    } else if matches!(
        kx0[0],
        3 | 1 | 4 | 6 | 7 | 8 | 10 | 15 | 16 | 18 | 14 | 2 | 17 | 19
    ) {
        if tables::check_chosong(kx0[1]) == 1 {
            Some(b'5')
        } else {
            Some(b'+')
        }
    } else {
        return false;
    };
    b.is_some_and(|b| yong_yon_add(pb_array, pn_size, b))
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn irr_predicate_retrieve(ma: &mut MorphAnalBuf, d: &KmaDicts, n_pos: i32) {
    if n_pos < 0 {
        return;
    }
    let mut i_var2 = 1usize;
    loop {
        let flag = i_var2 - 1;
        let c = ma.ps_pyogi_info[n_pos as usize].sch_con_status[flag];
        if c == b'o' {
            match flag {
                0 => original_retrieve_one(ma, d, n_pos, b'_', b'0'),
                1 => original_retrieve_one(ma, d, n_pos, b'L', b'0'),
                2 => original_retrieve_one(ma, d, n_pos, b'S', b's'),
                3 => original_retrieve_two(ma, d, n_pos, b'D', b'd'),
                4 => original_retrieve_two(ma, d, n_pos, b'B', b'b'),
                5 => original_retrieve_three(ma, d, n_pos),
                6 => original_retrieve_four(ma, d, n_pos),
                7 => original_retrieve_one(ma, d, n_pos, b'u', b'0'),
                8 => original_retrieve_five(ma, d, n_pos, b'x', b'a'),
                9 => original_retrieve_six(ma, d, n_pos),
                10 => original_retrieve_five(ma, d, n_pos, b'o', b'o'),
                11 => original_retrieve_seven(ma, d, n_pos),
                12 => original_retrieve_eight(ma, d, n_pos),
                13 => original_retrieve_nine(
                    ma,
                    d,
                    n_pos,
                    ma.ps_pyogi_info[n_pos as usize].ch_eng_pyogi,
                ),
                14 => original_retrieve_nine(ma, d, n_pos, b'a'),
                15 => original_retrieve_ten(ma, d, n_pos),
                16 => original_retrieve_eleven(ma, d, n_pos),
                17 => {
                    let ch = ma.ps_pyogi_info[n_pos as usize + 1].ch_eng_pyogi;
                    original_retrieve_one(ma, d, n_pos, ch, b'0');
                }
                18 => original_retrieve_twelve(ma, d, n_pos),
                19 => {
                    original_retrieve_thirteen(ma, d, n_pos);
                    return;
                }
                _ => {}
            }
        }
        i_var2 += 1;
        if i_var2 > 20 {
            return;
        }
    }
}

#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
fn store_pyogi_info(
    infos: &mut [PyogiInfo],
    idx: usize,
    w_phone_pos: i16,
    w_next: i16,
    chf_cv: u8,
    chf_cc: u8,
    ch_pyogi: u8,
) {
    let e = &mut infos[idx];
    e.w_self_pos = w_phone_pos;
    e.w_next_pos = w_next;
    e.b_start_able_flag = chf_cv;
    e.b_end_able_flag = chf_cc;
    e.ch_eng_pyogi = ch_pyogi;
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn original_retrieve(
    ma: &mut MorphAnalBuf,
    d: &KmaDicts,
    ch_cmp: u8,
    con_type: u8,
    ch_direct_flg: u8,
) {
    loop {
        let mut sch_pyogi: Vec<u8> = Vec::new();
        get_morph_pyogi(&mut sch_pyogi, ma);
        let n = search_kma_dict(ma, d, &sch_pyogi);
        let mut i = 0i32;
        while i < n {
            let info = ma.gs_k_anal_info[i as usize].clone();
            if info.irr_type == b'P' {
                add_process_morph_cand_list_one(ma, d, &sch_pyogi, &info);
                i += 1;
                continue;
            }
            if info.irr_type == b'T'
                && is_yongon_type(info.ch_pumsa, ch_cmp)
                && (info.ch_con_type == con_type || info.ch_con_type == b'P')
            {
                add_process_morph_cand_list_one(ma, d, &sch_pyogi, &info);
            }
            i += 1;
        }
        if !decide_new_focus_pos(ch_direct_flg, ma) {
            ma.b_focus_idx -= 1;
            return;
        }
    }
}

const fn is_yongon_type(ch_pumsa: u8, ch_cmp: u8) -> bool {
    if ch_cmp == b'd' {
        if ch_pumsa.wrapping_add(0x9f) < 3 {
            return true;
        }
    } else if ch_cmp == b'e' {
        if ch_pumsa.wrapping_add(0xa2) < 2 {
            return true;
        }
        return matches!(ch_pumsa, b'a' | b'd' | b'b' | b'g');
    } else if ch_cmp != b'p' {
        return false;
    }
    matches!(ch_pumsa, b'C' | b'@' | b'B' | b'f' | b'D')
}

#[allow(clippy::too_many_arguments)]
#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn original_retrieve_one(
    ma: &mut MorphAnalBuf,
    d: &KmaDicts,
    n_index: i32,
    ch_retrie_char: u8,
    ch_con_type: u8,
) {
    let n = n_index as usize;
    let w_self = ma.ps_pyogi_info[n].w_self_pos;
    let w_next = ma.ps_pyogi_info[n].w_next_pos;
    let mid = i16::midpoint(w_next, w_self);
    let ch = ma.ps_pyogi_info[n].ch_eng_pyogi;
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        w_self,
        mid,
        b'o',
        b'x',
        ch,
    );
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize + 1,
        mid,
        w_next,
        b'x',
        b'o',
        ch_retrie_char,
    );
    focus_info_set(ma, w_self, w_next, ma.w_pyogi_len + 1, 0, w_next, b'L');
    original_retrieve(ma, d, b'p', ch_con_type, b'L');
}

#[allow(clippy::too_many_arguments)]
#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn original_retrieve_two(
    ma: &mut MorphAnalBuf,
    d: &KmaDicts,
    n_index: i32,
    ch_retrie_char: u8,
    ch_con_type: u8,
) {
    let n = n_index as usize;
    let w_self = ma.ps_pyogi_info[n].w_self_pos;
    let w_next = ma.ps_pyogi_info[n].w_next_pos;
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        w_self,
        w_next,
        b'x',
        b'o',
        ch_retrie_char,
    );
    focus_info_set(ma, w_self, w_next, ma.w_pyogi_len, 0, w_next, b'L');
    original_retrieve(ma, d, b'p', ch_con_type, b'L');
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn original_retrieve_three(ma: &mut MorphAnalBuf, d: &KmaDicts, n_index: i32) {
    let n = n_index as usize;
    let w_self = ma.ps_pyogi_info[n].w_self_pos;
    let nxt_self = ma.ps_pyogi_info[n + 1].w_self_pos;
    let nxt_next = ma.ps_pyogi_info[n + 1].w_next_pos;
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        w_self,
        nxt_self,
        b'x',
        b'x',
        b'l',
    );
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize + 1,
        nxt_self,
        nxt_next,
        b'x',
        b'o',
        b'_',
    );
    focus_info_set(ma, w_self, nxt_next, ma.w_pyogi_len + 1, 0, nxt_next, b'L');
    original_retrieve(ma, d, b'p', b'l', b'L');
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn original_retrieve_four(ma: &mut MorphAnalBuf, d: &KmaDicts, n_index: i32) {
    let n = n_index as usize;
    let w_self = ma.ps_pyogi_info[n].w_self_pos;
    let nxt_self = ma.ps_pyogi_info[n + 1].w_self_pos;
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        w_self,
        nxt_self,
        b'x',
        b'o',
        b'_',
    );
    focus_info_set(ma, w_self, nxt_self, ma.w_pyogi_len, 0, nxt_self, b'L');
    original_retrieve(ma, d, b'p', b'L', b'L');
}

#[allow(clippy::too_many_arguments)]
#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn original_retrieve_five(
    ma: &mut MorphAnalBuf,
    d: &KmaDicts,
    n_index: i32,
    ch_bound_flg: u8,
    ch_eng_pyogi: u8,
) {
    let n = n_index as usize;
    let w_self = ma.ps_pyogi_info[n].w_self_pos;
    let nxt_next = ma.ps_pyogi_info[n + 1].w_next_pos;
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        w_self,
        nxt_next,
        ch_bound_flg,
        b'o',
        ch_eng_pyogi,
    );
    focus_info_set(ma, w_self, nxt_next, ma.w_pyogi_len, 0, nxt_next, b'L');
    original_retrieve(ma, d, b'p', b'0', b'L');
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn original_retrieve_six(ma: &mut MorphAnalBuf, d: &KmaDicts, n_index: i32) {
    let n = n_index as usize;
    let w_self = ma.ps_pyogi_info[n - 1].w_self_pos;
    let w_next = ma.ps_pyogi_info[n].w_next_pos;
    let ch = ma.ps_pyogi_info[n - 1].ch_eng_pyogi;
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        w_self,
        w_next,
        b'x',
        b'o',
        ch,
    );
    let fin = w_next;
    let ini = ma.ps_pyogi_info[n - 2].w_self_pos;
    focus_info_set(ma, ini, fin, ma.w_pyogi_len, 0, fin, b'L');
    original_retrieve(ma, d, b'p', b'0', b'L');
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn original_retrieve_seven(ma: &mut MorphAnalBuf, d: &KmaDicts, n_index: i32) {
    let n = n_index as usize;
    let w_self = ma.ps_pyogi_info[n].w_self_pos;
    let w_next = ma.ps_pyogi_info[n].w_next_pos;
    let mid = i16::midpoint(w_self, w_next);
    let ch = ma.ps_pyogi_info[n].ch_eng_pyogi;
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        w_self,
        mid,
        b'x',
        b'x',
        ch,
    );
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize + 1,
        mid,
        w_next,
        b'x',
        b'o',
        b'H',
    );
    let ini = ma.ps_pyogi_info[n - 1].w_self_pos;
    focus_info_set(ma, ini, w_next, ma.w_pyogi_len + 1, 0, w_next, b'L');
    original_retrieve(ma, d, b'p', b'h', b'L');
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn original_retrieve_eight(ma: &mut MorphAnalBuf, d: &KmaDicts, n_index: i32) {
    let n = n_index as usize;
    let w_self = ma.ps_pyogi_info[n].w_self_pos;
    let w_next = ma.ps_pyogi_info[n].w_next_pos;
    let mid = w_self + 4;
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        mid,
        w_next,
        b'o',
        b'o',
        b'a',
    );
    focus_info_set(ma, mid, w_next, ma.w_pyogi_len, mid, ma.w_fin_pos, b'R');
    original_retrieve(ma, d, b'e', b'0', b'R');
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        w_self,
        w_self + 2,
        b'x',
        b'x',
        b'a',
    );
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize + 1,
        w_self + 2,
        mid,
        b'x',
        b'o',
        b'H',
    );
    let ini = ma.ps_pyogi_info[n - 1].w_self_pos;
    focus_info_set(ma, ini, mid, ma.w_pyogi_len + 1, 0, mid, b'L');
    original_retrieve(ma, d, b'p', b'h', b'L');
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn original_retrieve_nine(ma: &mut MorphAnalBuf, d: &KmaDicts, n_index: i32, ch_retrie_char: u8) {
    let n = n_index as usize;
    let w_self = ma.ps_pyogi_info[n].w_self_pos;
    let w_next = ma.ps_pyogi_info[n].w_next_pos;
    let mid = i16::midpoint(w_next, w_self);
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        mid,
        w_next,
        b'o',
        b'o',
        b'e',
    );
    focus_info_set(ma, mid, w_next, ma.w_pyogi_len, mid, ma.w_fin_pos, b'R');
    original_retrieve(ma, d, b'e', b'0', b'R');
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        w_self,
        mid,
        b'x',
        b'o',
        ch_retrie_char,
    );
    let ini = ma.ps_pyogi_info[n - 1].w_self_pos;
    focus_info_set(ma, ini, mid, ma.w_pyogi_len, 0, mid, b'L');
    original_retrieve(ma, d, b'p', b'0', b'L');
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn original_retrieve_ten(ma: &mut MorphAnalBuf, d: &KmaDicts, n_index: i32) {
    let n = n_index as usize;
    let w_self = ma.ps_pyogi_info[n].w_self_pos;
    let w_next = ma.ps_pyogi_info[n].w_next_pos;
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        w_self,
        w_next,
        b'o',
        b'o',
        b'i',
    );
    focus_info_set(ma, w_self, w_next, ma.w_pyogi_len, 0, w_next, b'L');
    original_retrieve(ma, d, b'd', b'0', b'L');
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn original_retrieve_eleven(ma: &mut MorphAnalBuf, d: &KmaDicts, n_index: i32) {
    let n = n_index as usize;
    let w_self = ma.ps_pyogi_info[n].w_self_pos;
    let w_next = ma.ps_pyogi_info[n].w_next_pos;
    let ch = if ma.ps_pyogi_info[n + 1].ch_eng_pyogi == b'a' {
        b'o'
    } else {
        b'u'
    };
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        w_self,
        w_next,
        b'o',
        b'o',
        ch,
    );
    focus_info_set(ma, w_self, w_next, ma.w_pyogi_len, 0, w_next, b'L');
    original_retrieve(ma, d, b'p', b'0', b'L');
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn original_retrieve_twelve(ma: &mut MorphAnalBuf, d: &KmaDicts, n_index: i32) {
    let n = n_index as usize;
    let w_self = ma.ps_pyogi_info[n].w_self_pos;
    let w_next = ma.ps_pyogi_info[n].w_next_pos;
    let mid = i16::midpoint(w_next, w_self);
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        mid,
        w_next,
        b'o',
        b'o',
        b'e',
    );
    focus_info_set(ma, mid, w_next, ma.w_pyogi_len, mid, ma.w_fin_pos, b'R');
    original_retrieve(ma, d, b'e', b'0', b'R');
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        w_self,
        mid,
        b'x',
        b'o',
        b'i',
    );
    let ini = ma.ps_pyogi_info[n - 1].w_self_pos;
    focus_info_set(ma, ini, mid, ma.w_pyogi_len, 0, mid, b'L');
    original_retrieve(ma, d, b'p', b'0', b'L');
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn original_retrieve_thirteen(ma: &mut MorphAnalBuf, d: &KmaDicts, n_index: i32) {
    let n = n_index as usize;
    let w_start = ma.ps_pyogi_info[n].w_self_pos;
    let w_next = ma.ps_pyogi_info[n].w_next_pos;
    let w_phone_pos = w_start + 1;
    let ch = ma.ps_pyogi_info[n].ch_eng_pyogi;
    store_pyogi_info(
        &mut ma.ps_pyogi_info,
        ma.w_pyogi_len as usize,
        w_phone_pos,
        w_next,
        b'o',
        b'x',
        ch,
    );
    focus_info_set(
        ma,
        w_phone_pos,
        w_next,
        ma.w_pyogi_len,
        w_phone_pos,
        ma.w_fin_pos,
        b'R',
    );
    loop {
        let mut sch_pyogi: Vec<u8> = Vec::new();
        get_morph_pyogi(&mut sch_pyogi, ma);
        let n2 = search_kma_dict(ma, d, &sch_pyogi);
        if n2 > 0 {
            let mut i = 0i32;
            while i < n2 {
                let info = &ma.gs_k_anal_info[i as usize];
                let is_target = info.irr_type == b'T'
                    && (info.ch_pumsa.wrapping_add(0xa5) < 2
                        || info.ch_pumsa == b'`'
                        || info.ch_pumsa == b'_');
                if is_target {
                    add_process_morph_cand_list(ma, d, &sch_pyogi, 1);
                }
                i += 1;
            }
        }
        if !decide_new_focus_pos(b'R', ma) {
            ma.b_focus_idx -= 1;
            focus_info_set(
                ma,
                w_start,
                w_phone_pos,
                ma.w_pyogi_len,
                w_start,
                w_phone_pos,
                b'L',
            );
            let stale_part = ma.gs_k_anal_info.first().map_or(0.0, |x| x.d_part_prob);
            let stale_to = ma.gs_k_anal_info.first().map_or(0, |x| x.un_to_info);
            ma.gs_k_anal_info.clear();
            ma.gs_k_anal_info.push(KAnalInfo {
                irr_type: b'T',
                ch_pumsa: b'c',
                ch_con_type: 0,
                d_part_prob: stale_part,
                d_word_prob: 0.0,
                un_to_info: stale_to,
                irr_string: Vec::new(),
            });
            add_process_morph_cand_list(ma, d, b"i", 1);
            ma.b_focus_idx -= 1;
            return;
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_special_unk_noun(pch_str: &mut Vec<u8>, ma: &mut MorphAnalBuf, d: &KmaDicts) {
    if pch_str.len() <= 3 {
        return;
    }
    let tail = &pch_str[pch_str.len() - 2..];
    let is_k9 = tail == b"k9";
    let is_ci = tail == b"ci";
    if !is_k9 && !is_ci {
        return;
    }
    pch_str.truncate(pch_str.len() - 2);
    pch_str.extend_from_slice(b"ha");
    let n = search_kma_dict(ma, d, pch_str);
    if n <= 0 {
        return;
    }
    let mut cand_idx_list: Vec<i32> = Vec::new();
    for (i, info) in ma.gs_k_anal_info.iter().enumerate().take(n as usize) {
        if info.irr_type == b'T' && tables::is_k_voice_yong_yon(info.ch_pumsa) {
            cand_idx_list.push(i as i32);
        }
    }
    if cand_idx_list.is_empty() {
        return;
    }
    let (ch_new_tag, u_to_info) = if is_k9 {
        (b'`', 0x1e1u32)
    } else {
        (b'g', 0x3c1u32)
    };
    let sch_tail: Vec<u8> = if is_k9 {
        b"g9".to_vec()
    } else {
        b"ji".to_vec()
    };
    let mut root: i16 = -1;
    let mut links = vec![0i16; 20];
    let end = ma.ps_morph_cand[ma.n_morph_cand_cnt as usize].w_end_pos;
    get_unknown_link_morph_idx(
        ma,
        d,
        ma.n_morph_cand_cnt,
        &mut root,
        &mut links,
        end,
        ch_new_tag,
        &sch_tail,
    );
    let w_start = (pch_str.len() as i16) * 6 - 12;
    let w_end = (pch_str.len() as i16) * 6;
    let info = KAnalInfo {
        irr_type: b'T',
        ch_pumsa: ch_new_tag,
        ch_con_type: b'0',
        d_part_prob: 0.0,
        d_word_prob: 1.0,
        un_to_info: u_to_info,
        irr_string: Vec::new(),
    };
    add_morph_cand_proc(ma, w_start, w_end, root, &links, &sch_tail, &info, false);
    let saved_fin = ma.s_focus_range[0].w_fin_pos;
    let saved_ini = ma.s_focus_range[0].w_ini_pos;
    ma.s_focus_range[0].w_ini_pos = 0;
    ma.s_focus_range[0].w_fin_pos = w_start;
    for &ci in &cand_idx_list {
        let info = ma.gs_k_anal_info[ci as usize].clone();
        add_process_morph_cand_list_one(ma, d, pch_str, &info);
    }
    ma.s_focus_range[0].w_fin_pos = saved_fin;
    ma.s_focus_range[0].w_ini_pos = saved_ini;
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn add_process_morph_cand_list_one(
    ma: &mut MorphAnalBuf,
    d: &KmaDicts,
    pch_string: &[u8],
    info: &KAnalInfo,
) {
    ma.gsch_morph_cand_str = pch_string.to_vec();
    ma.gn_morph_cand_str = pch_string.len().saturating_sub(1);
    let b_focus = ma.b_focus_idx as usize;
    let w_fin_pos = ma.s_focus_range[b_focus].w_fin_pos;
    let w_ini_pos = ma.s_focus_range[b_focus].w_ini_pos;
    if info.irr_type == b'T' {
        let mut root: i16 = -1;
        let mut links = vec![0i16; 20];
        get_link_morph_idx(
            ma,
            d,
            ma.n_morph_cand_cnt,
            &mut root,
            &mut links,
            w_fin_pos,
            info.ch_pumsa,
            false,
        );
        add_morph_cand_proc(
            ma, w_ini_pos, w_fin_pos, root, &links, pch_string, info, true,
        );
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn get_unknoun_pyogi(out: &mut Vec<u8>, ma: &MorphAnalBuf, w_pos: i16) {
    out.clear();
    let n = ma.w_pyogi_len as usize;
    if n == 0 {
        return;
    }
    out.push(ma.ps_pyogi_info[0].ch_eng_pyogi);
    let mut i = 1usize;
    while i < n && ma.ps_pyogi_info[i - 1].w_next_pos != w_pos {
        out.push(ma.ps_pyogi_info[i].ch_eng_pyogi);
        i += 1;
    }
}

fn is_unknown_tail_morph(cand: &MorphCand) -> bool {
    tables::is_k_cheon_part(cand.ch_pumsa)
        || tables::is_k_cheon_to(cand.ch_pumsa)
        || cand.ch_pumsa == b'k'
}

#[expect(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_pronoun(ma: &mut MorphAnalBuf, d: &KmaDicts) {
    let n_cand = ma.n_morph_cand_cnt;
    if n_cand < 0 {
        return;
    }
    let mut i = 0i32;
    while i <= n_cand {
        let cand = ma.ps_morph_cand[i as usize].clone();
        let w_pos = cand.w_start_pos;
        if is_unknown_tail_morph(&cand) && w_pos != ma.gw_pre_tail_phone_start_pos {
            ma.gw_pre_tail_phone_start_pos = w_pos;
            if ma.w_pyogi_len > 0 {
                let mut found = false;
                for pi in ma.ps_pyogi_info.iter().take(ma.w_pyogi_len as usize) {
                    if pi.b_end_able_flag == b'o' && pi.w_next_pos == w_pos {
                        found = true;
                        break;
                    }
                }
                if found {
                    let mut sch_pyogi: Vec<u8> = Vec::new();
                    get_unknoun_pyogi(&mut sch_pyogi, ma, w_pos);
                    if let Some(tag) = proc_unknown_morph(&sch_pyogi, d) {
                        let mut root: i16 = -1;
                        let mut links = vec![0i16; 20];
                        get_unknown_link_morph_idx(
                            ma,
                            d,
                            ma.n_morph_cand_cnt,
                            &mut root,
                            &mut links,
                            w_pos,
                            tag,
                            &sch_pyogi,
                        );
                        let freq = f64::from(d.pos.uni.freq(tag));
                        let mut prob = if freq > 0.0 {
                            1.0 / (freq * 1000.0)
                        } else {
                            1.0
                        };
                        let n_char = crate::code::get_kchar_count(&sch_pyogi);
                        if n_char > 1 {
                            if n_char < 4 {
                                if tag == 0x3c {
                                    if !tables::is_k_cheon_part(cand.ch_pumsa) {
                                        prob *= 0.5;
                                    }
                                } else {
                                    prob /= (n_char * 10000) as f64;
                                }
                            } else {
                                prob /= (n_char * 10000) as f64;
                                if n_char > 6 {
                                    prob /= 10000.0;
                                }
                            }
                        }
                        let prob = prob.ln();
                        add_morph_cand(ma, 0, w_pos, root, &links, &sch_pyogi, tag, prob);
                    }
                }
            }
        }
        i += 1;
    }
}

fn proc_unknown_morph(pch_morph: &[u8], d: &KmaDicts) -> Option<u8> {
    let sw_korea = conv_pyogi_to_uni_wan(pch_morph);
    let len = sw_korea.len();
    if (2..=4).contains(&len) && proc_korean_name(&sw_korea, d) {
        return Some(b'<');
    }
    proc_foreign_name(&sw_korea, d)
}

fn proc_korean_name(pw_morph: &[u16], d: &KmaDicts) -> bool {
    let mut key = vec![0x4cu16];
    key.extend_from_slice(pw_morph);
    let matched_len = d.namegram_match_len(&key);
    if matched_len > 0 {
        let mut n_key = vec![0x4eu16];
        n_key.extend_from_slice(&pw_morph[matched_len - 1..]);
        if d.namegram(&n_key) {
            return true;
        }
    }
    false
}

#[expect(
    clippy::cast_precision_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn proc_foreign_name(pw_morph: &[u16], d: &KmaDicts) -> Option<u8> {
    let mut sw_str = vec![0x53u16];
    sw_str.extend_from_slice(pw_morph);
    sw_str.push(0x53);
    let n_total = sw_str.len();
    let mut d_probability = 0.0f64;
    let mut b_region_name = true;
    let mut b_foreign_name = true;
    let mut i_var5 = 1usize;
    while i_var5 < n_total {
        if b_foreign_name {
            let mut key = vec![0x46u16];
            key.push(sw_str[i_var5 - 1]);
            key.push(sw_str[i_var5]);
            b_foreign_name = d.namegram(&key);
        }
        if b_region_name {
            let mut key = vec![0x52u16];
            key.push(sw_str[i_var5 - 1]);
            key.push(sw_str[i_var5]);
            b_region_name = d.namegram(&key);
        }
        let ckey = [sw_str[i_var5 - 1], sw_str[i_var5]];
        match d.chargram(&ckey) {
            Some(p) => d_probability += f64::from(p),
            None => d_probability -= 10.35,
        }
        i_var5 += 1;
        if i_var5 >= n_total {
            break;
        }
    }
    if !b_foreign_name {
        if b_region_name {
            return Some(b'9');
        }
        let ratio = d_probability / ((n_total as f64) * (n_total as f64).ln());
        if ratio >= -3.5 {
            return None;
        }
        return Some(b'9');
    }
    Some(b'<')
}

#[allow(unused_assignments, unused_variables)]
#[allow(clippy::too_many_arguments)]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn get_unknown_link_morph_idx(
    ma: &MorphAnalBuf,
    d: &KmaDicts,
    n_curr_morph_cnt: i32,
    pw_root_morph_pos: &mut i16,
    pw_link_morph_idx: &mut Vec<i16>,
    w_new_morph_end: i16,
    ch_new_tag: u8,
    pch_word: &[u8],
) {
    let mut local_5e: i16 = -1;
    let mut n_count = 0usize;
    let mut links_out: Vec<i16> = Vec::new();
    let mut i = n_curr_morph_cnt;
    while i >= 0 {
        let cand = &ma.ps_morph_cand[i as usize];
        if cand.w_start_pos == w_new_morph_end {
            let d_var4 = d.pos_bigram(ch_new_tag, cand.ch_pumsa);
            let word_ok = is_unknown_tail_morph(cand) || pch_word == b"ji" || pch_word == b"k9";
            if d_var4 != 0.0 && word_ok && cand.w_root_pos != -1 {
                let cheon_ok = !tables::is_k_cheon_part(ch_new_tag)
                    || !tables::is_to(cand.ch_pumsa)
                    || is_combinable_cheon_to(pch_word, false, cand);
                let yong_ok = !tables::is_k_voice_yong_yon(ch_new_tag)
                    || !tables::is_k_yongon_to(cand.ch_pumsa)
                    || is_combinable_yongon_to(
                        &ma.gsch_morph_cand_str,
                        ch_new_tag,
                        false,
                        cand,
                        &ma.pch_pyogi_str,
                        d,
                    );
                if cheon_ok && yong_ok {
                    links_out.push(i as i16);
                    n_count += 1;
                    if cand.w_root_pos > local_5e {
                        local_5e = cand.w_root_pos;
                    }
                }
            }
        }
        i -= 1;
    }
    links_out.push(-1);
    *pw_link_morph_idx = links_out;
    *pw_root_morph_pos = local_5e;
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn sequence(ma: &mut MorphAnalBuf, d: &KmaDicts) -> i32 {
    let n_bos = ma.n_morph_cand_cnt;
    if n_bos < 0 {
        return -1;
    }
    let mut link_idx: Vec<i16> = Vec::new();
    let bos_links = ma.ps_morph_cand[n_bos as usize].sw_link_morph_idx.clone();
    for &l in &bos_links {
        if l == -1 {
            break;
        }
        add_idx_in_link_idx(&mut link_idx, l);
    }
    let mut qi = 0usize;
    while qi < link_idx.len() {
        let cand_idx = link_idx[qi];
        qi += 1;
        let links = ma.ps_morph_cand[cand_idx as usize]
            .sw_link_morph_idx
            .clone();
        let mut n_links = 0;
        for &l in &links {
            if l == -1 {
                break;
            }
            add_idx_in_link_idx(&mut link_idx, l);
            n_links += 1;
        }
        if n_links == 0 && link_idx.len() == 1 {
            return -1;
        }
    }
    if link_idx.is_empty() {
        if process_last_unknown(ma, d) {
            return sequence(ma, d);
        }
        return -1;
    }
    link_idx.sort_by(|a, b| b.cmp(a));
    ma.pw_link_idx_cache = link_idx;
    ma.pw_link_idx_cache.len() as i32
}

fn add_idx_in_link_idx(buf: &mut Vec<i16>, w_value: i16) {
    if w_value == 0 {
        return;
    }
    if !buf.contains(&w_value) {
        buf.push(w_value);
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_last_unknown(ma: &mut MorphAnalBuf, d: &KmaDicts) -> bool {
    let mut i = ma.n_morph_cand_cnt;
    while i >= 0 {
        let cand = ma.ps_morph_cand[i as usize].clone();
        if !cand.sw_link_morph_idx.is_empty()
            && cand.sw_link_morph_idx[0] != -1
            && ma.w_pyogi_len > 0
        {
            let w_pos = cand.sw_link_morph_idx[0];
            for info in &ma.ps_pyogi_info {
                if info.b_end_able_flag == b'o' && cand.w_start_pos == info.w_next_pos {
                    if is_unknown_tail_morph(&cand) {
                        let mut sch_pyogi: Vec<u8> = Vec::new();
                        get_unknoun_pyogi(&mut sch_pyogi, ma, w_pos);
                        if let Some(tag) = proc_unknown_morph(&sch_pyogi, d) {
                            let mut root: i16 = -1;
                            let mut links = vec![0i16; 20];
                            get_unknown_link_morph_idx(
                                ma,
                                d,
                                ma.n_morph_cand_cnt,
                                &mut root,
                                &mut links,
                                w_pos,
                                tag,
                                &sch_pyogi,
                            );
                            add_morph_cand(ma, 0, w_pos, root, &links, &sch_pyogi, tag, 1.0);
                            let nxt = ma.n_morph_cand_cnt;
                            add_morph_cand(
                                ma,
                                0,
                                0,
                                cand.w_root_pos,
                                &[nxt as i16, -1],
                                &sch_pyogi,
                                tag,
                                1.0,
                            );
                            return true;
                        }
                    }
                    break;
                }
            }
        }
        i -= 1;
    }
    false
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn set_morpheme_cand_info(ma: &mut MorphAnalBuf, d: &KmaDicts) {
    let n = sequence(ma, d);
    if n < 0 {
        let i = ma.n_morpheme_cnt as usize;
        let mut node = MorphNode::new();
        node.ch_tag = b':';
        node.pyogi.clone_from(&ma.pch_pyogi_str);
        node.w_start_pos = 0;
        node.w_end_pos = (ma.pch_pyogi_str.len() * 6) as i16;
        node.d_self_prob = tables::PROB_ALL_ONE_NODE;
        node.b_next_morph_flag = b'o';
        node.child = vec![i as i16 + 1, -1];
        node.parent = vec![0, -1];
        node.w_route_idx = 0;
        ma.ps_morpheme.push(node);
        ma.n_morpheme_cnt += 1;
        let mut fin = MorphNode::new();
        fin.pyogi = b"FIN".to_vec();
        fin.w_route_idx = -1;
        fin.child = vec![-1];
        fin.parent = vec![-1];
        fin.b_next_morph_flag = b'o';
        fin.d_self_prob = 0.0;
        fin.ch_tag = ma.gb_end_tag;
        ma.ps_morpheme.push(fin);
        ma.n_morpheme_cnt += 1;
        marking_parent(ma);
        return;
    }
    let n_cand = n as usize;
    let base = ma.n_morpheme_cnt as usize;
    let mut children: Vec<Vec<i16>> = Vec::with_capacity(n_cand);
    let mut next_flags: Vec<u8> = Vec::with_capacity(n_cand);
    for &cand_idx in &ma.pw_link_idx_cache {
        let cand = &ma.ps_morph_cand[cand_idx as usize];
        let mut ch: Vec<i16> = Vec::new();
        let mut last_link = -1i16;
        for &l in &cand.sw_link_morph_idx {
            if l == -1 {
                break;
            }
            last_link = l;
            ch.push(curr_idx_in_link_idx(&ma.pw_link_idx_cache, l) as i16);
        }
        ch.push(-1);
        children.push(ch);
        next_flags.push(if last_link == 0 { b'o' } else { b'x' });
    }
    for (i, &cand_idx) in ma.pw_link_idx_cache.iter().enumerate() {
        let cand = &ma.ps_morph_cand[cand_idx as usize];
        let mut node = MorphNode::new();
        node.ch_tag = cand.ch_pumsa;
        node.pyogi.clone_from(&cand.sch_morph_str);
        node.w_start_pos = cand.w_start_pos;
        node.w_end_pos = cand.w_end_pos;
        node.d_self_prob = cand.d_probability;
        node.b_retrieved = cand.b_retrieved;
        node.w_route_idx = -1;
        node.b_next_morph_flag = next_flags[i];
        node.child = children[i]
            .iter()
            .map(|&c| if c == -1 { -1 } else { base as i16 + c })
            .collect();
        node.parent = vec![-1];
        ma.ps_morpheme.push(node);
    }
    ma.n_morpheme_cnt = (base + n_cand) as i32;
    if ma.ps_morpheme[0].ch_tag == ma.gb_ini_tag {
        let bos_cand = &ma.ps_morph_cand[ma.n_morph_cand_cnt as usize];
        let mut ch: Vec<i16> = Vec::new();
        for &l in &bos_cand.sw_link_morph_idx {
            if l == -1 {
                break;
            }
            ch.push(base as i16 + curr_idx_in_link_idx(&ma.pw_link_idx_cache, l) as i16);
        }
        ch.push(-1);
        ma.ps_morpheme[0].child = ch;
    }
    let fin_idx = ma.n_morpheme_cnt as usize;
    let mut fin = MorphNode::new();
    fin.pyogi = b"FIN".to_vec();
    fin.w_route_idx = -1;
    fin.child = vec![-1];
    fin.parent = vec![-1];
    fin.b_next_morph_flag = b'o';
    fin.d_self_prob = 0.0;
    fin.ch_tag = ma.gb_end_tag;
    ma.ps_morpheme.push(fin);
    ma.n_morpheme_cnt += 1;
    let _ = fin_idx;
    marking_parent(ma);
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "C port: index/math casts with wrap semantics"
)]
fn curr_idx_in_link_idx(buf: &[i16], w_value: i16) -> i32 {
    if w_value == 0 {
        return buf.len() as i32;
    }
    buf.iter().position(|&x| x == w_value).unwrap_or(buf.len()) as i32
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn marking_parent(ma: &mut MorphAnalBuf) {
    let n = ma.n_morpheme_cnt as usize;
    for i in 0..n {
        let children = ma.ps_morpheme[i].child.clone();
        for &c in &children {
            if c == -1 {
                break;
            }
            let c = c as usize;
            if c >= ma.ps_morpheme.len() {
                continue;
            }
            let par = &mut ma.ps_morpheme[c].parent;
            if let Some(pos) = par.iter().position(|&x| x == -1) {
                par[pos] = i as i16;
                if pos + 1 == par.len() {
                    par.push(-1);
                }
            } else {
                par.push(i as i16);
                par.push(-1);
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
fn viterbi_search(n_word_idx: i32, ma: &mut MorphAnalBuf, d: &KmaDicts, out: &mut [i32]) -> usize {
    let n = ma.n_morpheme_cnt as usize;
    if n > 1 {
        for i in 1..n {
            let parents = ma.ps_morpheme[i].parent.clone();
            let mut best = f64::NEG_INFINITY;
            let mut best_parent = -1i16;
            for &p in &parents {
                if p == -1 {
                    break;
                }
                let ll = ma.ps_morpheme[p as usize].w_route_idx;
                let v = ma.ps_morpheme[p as usize].d_accum_prob
                    + get_com_trans_prob(n_word_idx, ll, p, i as i16, ma, d);
                if v > best {
                    best = v;
                    best_parent = p;
                }
            }
            ma.ps_morpheme[i].d_accum_prob = best;
            ma.ps_morpheme[i].w_route_idx = best_parent;
        }
    }
    get_rout(out, ma)
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn get_rout(out: &mut [i32], ma: &MorphAnalBuf) -> usize {
    let n = ma.n_morpheme_cnt as usize;
    let mut i = i32::from(ma.ps_morpheme[n - 1].w_route_idx);
    if i <= 0 {
        return 0;
    }
    let mut cnt = 0usize;
    out[cnt] = i;
    cnt += 1;
    while i > 0 {
        i = i32::from(ma.ps_morpheme[i as usize].w_route_idx);
        if i <= 0 {
            return cnt;
        }
        out[cnt] = i;
        cnt += 1;
    }
    cnt
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
fn get_com_trans_prob(
    n_word_idx: i32,
    n_ll_ix: i16,
    n_l_ix: i16,
    n_r_ix: i16,
    ma: &mut MorphAnalBuf,
    d: &KmaDicts,
) -> f64 {
    let ll: Option<usize> = if n_ll_ix >= 0 {
        Some(n_ll_ix as usize)
    } else {
        None
    };
    let l = n_l_ix as usize;
    let r = n_r_ix as usize;
    let mut ch_r_tag = ma.ps_morpheme[r].ch_tag;
    let mut local_cc = ma.ps_morpheme[r].d_self_prob;
    if tables::is_k_yongon_to(ch_r_tag) && !ma.ps_morpheme[r].b_standard_to_flag {
        let mut sch_to = ma.ps_morpheme[r].pyogi.clone();
        let mut tag = ch_r_tag;
        let changed = standardize_to(&mut sch_to, &mut tag);
        if changed {
            let n = search_kma_dict(ma, d, &sch_to);
            if n > 0
                && let Some(info) = ma.gs_k_anal_info.iter().find(|x| x.ch_pumsa == ch_r_tag)
            {
                local_cc = info.d_word_prob;
                ma.ps_morpheme[r].d_self_prob = local_cc;
                ma.ps_morpheme[r].b_standard_to_flag = true;
            }
        } else {
            ma.ps_morpheme[r].b_standard_to_flag = true;
        }
    }
    let mut c_var3 = ll.map_or(ma.gb_ini_ll_tag, |x| ma.ps_morpheme[x].ch_tag);
    if c_var3 == b'j' && n_word_idx > 1 {
        c_var3 = b'k';
    }
    let mut b_pumsa = ma.ps_morpheme[l].ch_tag;
    if b_pumsa == b'j' && n_word_idx > 0 {
        b_pumsa = b'k';
    }
    if ch_r_tag == b'l' && n_word_idx < ma.gn_total_str_count - 1 {
        ch_r_tag = b'k';
    }
    let b_pumsa_00 = ch_r_tag;
    let mut d_all_prob;
    if b_pumsa == b'j' {
        d_all_prob = local_cc + d.pos_bigram(b'j', ch_r_tag);
    } else {
        let key = [c_var3, b_pumsa, ch_r_tag];
        if let Some(tri) = d.trigram(&key) {
            d_all_prob = local_cc + tri;
        } else {
            let bi = d.pos_bigram(b_pumsa, ch_r_tag);
            if bi == 0.0 {
                d_all_prob = local_cc - tables::PENALTY_BIGRAM_ZERO;
            } else {
                d_all_prob = local_cc + bi;
            }
            if tables::is_k_char_root_part(ch_r_tag) {
                d_all_prob -= tables::PENALTY_CHAR_ROOT;
            }
        }
    }
    if tables::is_k_cheon_part(b_pumsa_00) {
        if (b_pumsa.wrapping_add(0xba)) < 2
            && crate::code::get_kchar_count(&ma.ps_morpheme[r].pyogi) > 1
        {
            d_all_prob -= tables::PENALTY_CHEON;
        }
        if tables::is_k_voice_jarib_cheon(b_pumsa)
            && tables::is_k_voice_jarib_cheon(b_pumsa_00)
            && crate::code::get_kchar_count(&ma.ps_morpheme[l].pyogi) == 1
            && crate::code::get_kchar_count(&ma.ps_morpheme[r].pyogi) == 1
        {
            d_all_prob -= tables::PENALTY_VOICE_CHEON;
        }
    }
    let r_pyogi = ma.ps_morpheme[r].pyogi.clone();
    let l_pyogi = ma.ps_morpheme[l].pyogi.clone();
    let off = d
        .wordgram_lookup(&r_pyogi, b'3')
        .or_else(|| d.wordgram_lookup(&l_pyogi, b'2'))
        .or_else(|| d.wordgram_lookup(&l_pyogi, b'1'));
    if let Some(off) = off {
        let ims = [
            ll.map(|x| &ma.ps_morpheme[x]),
            Some(&ma.ps_morpheme[l]),
            Some(&ma.ps_morpheme[r]),
        ];
        d_all_prob += d.search_pattern(
            off,
            ims,
            &ma.gsch_ini_ll_str,
            &ma.gsch_ini_str,
            ma.gb_ini_ll_tag,
            ma.gb_ini_tag,
        );
    }
    d_all_prob
}

#[allow(unused_assignments)]
fn standardize_to(pch_to: &mut Vec<u8>, pch_tag: &mut u8) -> bool {
    let is_cheon = tables::is_k_cheon_to(*pch_tag);
    let is_bagum = tables::is_bagum_yi(*pch_tag);
    if !is_cheon || is_bagum {
        if !tables::is_k_yongon_to(*pch_tag) {
            return true;
        }
        let mut b_var6 = true;
        if pch_to == b"_L" {
            *pch_to = b"L".to_vec();
        } else if pch_to == b"_N" {
            *pch_to = b"N".to_vec();
            b_var6 = true;
        } else if (pch_to.starts_with(b"e") || pch_to.starts_with(b"ye")) && *pch_tag == b'g' {
            if pch_to.first() == Some(&b'e') {
                pch_to[0] = b'a';
                b_var6 = true;
            } else {
                b_var6 = true;
                pch_to.drain(0..1);
                pch_to[0] = b'a';
            }
        } else if pch_to == b"ese" || pch_to.starts_with(b"yese") {
            *pch_to = b"ase".to_vec();
            b_var6 = true;
        } else if pch_to.starts_with(b"yeV") || pch_to.starts_with(b"yeS") {
            b_var6 = true;
            pch_to.drain(0..2);
        } else if pch_to.starts_with(b"eV")
            || pch_to.starts_with(b"eS")
            || pch_to.starts_with(b"aV")
            || pch_to.starts_with(b"aS")
        {
            b_var6 = true;
            pch_to.drain(0..1);
        } else if pch_to.starts_with(b"aVeV") {
            b_var6 = true;
            pch_to.drain(0..3);
            return true;
        } else {
            b_var6 = false;
        }
        if pch_to.len() >= 3 && pch_to.starts_with(b"_") {
            pch_to.drain(0..1);
        }
        return b_var6;
    }
    if pch_to == b"i" || pch_to.starts_with(b"q9se") {
        *pch_to = b"ga".to_vec();
        return true;
    }
    if pch_to == b"q9sen_N" {
        *pch_to = b"n_N".to_vec();
        *pch_tag = b']';
        return true;
    }
    if pch_to.len() > 2 && pch_to.starts_with(b"i") {
        pch_to.drain(0..1);
        return true;
    }
    if pch_to == b"_L" {
        *pch_to = b"l_L".to_vec();
        return true;
    }
    if pch_to == b"_N" {
        *pch_to = b"n_N".to_vec();
        return true;
    }
    if pch_to == b"wa" {
        *pch_to = b"gwa".to_vec();
        return true;
    }
    if pch_to.len() < 3 {
        return false;
    }
    if pch_to.starts_with(b"_") {
        pch_to.drain(0..1);
        return true;
    }
    false
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn morph_source_cvc(ma: &MorphAnalBuf, node: &MorphNode) -> Vec<u8> {
    if let Some(e) = ma.pps_ireguler.iter().find(|e| e.pyogi == node.pyogi) {
        return e.irr_string.clone();
    }
    let s = (node.w_start_pos as usize) / 6;
    let e = (node.w_end_pos as usize) / 6;
    if node.b_retrieved
        && !tables::is_k_yongon_to(node.ch_tag)
        && e > s
        && e <= ma.pch_pyogi_str.len()
        && e - s == node.pyogi.len()
    {
        return conv_pyogi_to_cvc(&ma.pch_pyogi_str[s..e]);
    }
    conv_pyogi_to_cvc(&node.pyogi)
}

fn surface_syllable_spans(pyogi: &[u8]) -> Vec<(usize, usize)> {
    use crate::tables::{SSCH_CHO, SSCH_JONG, SSCH_JUNG};
    let n = pyogi.len();
    if n == 0 {
        return Vec::new();
    }
    let mut ptype: Vec<u8> = Vec::new();
    let mut clen: Vec<usize> = Vec::new();
    let mut i = 0usize;
    let mut ok = true;
    while i < n {
        loop {
            let mut matched = false;
            for e in SSCH_JUNG.iter().rev() {
                if !e.is_empty() && pyogi[i..].starts_with(e) {
                    ptype.push(2);
                    clen.push(e.len());
                    i += e.len();
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }
            for e in SSCH_JONG.iter().rev() {
                if !e.is_empty() && pyogi[i..].starts_with(e) {
                    ptype.push(3);
                    clen.push(e.len());
                    i += e.len();
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }
            break;
        }
        if i >= n {
            break;
        }
        let mut matched = false;
        for e in SSCH_CHO.iter().rev() {
            if !e.is_empty() && pyogi[i..].starts_with(e) {
                ptype.push(0);
                clen.push(e.len());
                i += e.len();
                matched = true;
                break;
            }
        }
        if !matched {
            ok = false;
            break;
        }
    }
    if !ok || ptype.is_empty() {
        return Vec::new();
    }
    let mut spans = Vec::new();
    let mut j = 0usize;
    let mut i2 = 1usize;
    loop {
        let emit = if i2 < ptype.len() {
            let prev_t = ptype[i2 - 1];
            let cur_t = ptype[i2];
            !((prev_t == 0 && cur_t == 2) || (prev_t == 2 && cur_t == 3))
        } else {
            true
        };
        if emit {
            let start: usize = clen[..j].iter().sum();
            let end: usize = clen[..i2].iter().sum();
            spans.push((start, end));
            j = i2;
        }
        if i2 >= ptype.len() {
            break;
        }
        i2 += 1;
    }
    spans
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn merge_contracted_syllables(ma: &MorphAnalBuf, morphs: &mut [MaMorph], pairs: &[(i32, usize)]) {
    if morphs.len() < 2 {
        return;
    }
    let wlen = ma.w_pyogi_len as usize;
    if wlen > ma.pch_pyogi_str.len() {
        return;
    }
    let pyogi = &ma.pch_pyogi_str[..wlen];
    let spans = surface_syllable_spans(pyogi);
    if spans.is_empty() {
        return;
    }
    for k in 1..pairs.len() {
        let node = &ma.ps_morpheme[pairs[k].0 as usize];
        if !tables::is_k_yongon_to(node.ch_tag) {
            continue;
        }
        let s = (node.w_start_pos as usize) / 6;
        let e = (node.w_end_pos as usize) / 6;
        if s >= e || e > pyogi.len() {
            continue;
        }
        let vowel_first = node
            .pyogi
            .first()
            .is_some_and(|&c| tables::SSCH_JUNG.iter().any(|t| !t.is_empty() && t[0] == c));
        if !vowel_first {
            continue;
        }
        let Some(&(syl_start, syl_end)) = spans.iter().find(|(a, b)| *a <= s && s < *b) else {
            continue;
        };
        if syl_start == s {
            continue;
        }
        for &(node_idx, mi) in pairs {
            let n2 = &ma.ps_morpheme[node_idx as usize];
            let s2 = (n2.w_start_pos as usize) / 6;
            let e2 = (n2.w_end_pos as usize) / 6;
            if e2 <= syl_start || s2 >= syl_end {
                continue;
            }
            if s2 > syl_start && s2 < syl_end {
                let rest_start = syl_end.min(e2);
                if rest_start < e2 && rest_start <= pyogi.len() {
                    morphs[mi].cvc = conv_pyogi_to_cvc(&pyogi[rest_start..e2]);
                } else {
                    morphs[mi].cvc = Vec::new();
                    morphs[mi].b_merged = true;
                }
            } else {
                let e_eff = if syl_start >= s2 && syl_start < e2 {
                    syl_end.max(e2)
                } else {
                    e2
                };
                if s2 < e_eff && e_eff <= pyogi.len() {
                    morphs[mi].cvc = conv_pyogi_to_cvc(&pyogi[s2..e_eff]);
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
fn format_ma_result(ma: &MorphAnalBuf, klp: &KlpState, route: &[i32], d: &KmaDicts) -> MaWord {
    let mut word = MaWord {
        source: ma.pw_korea_str.clone(),
        morphs: Vec::new(),
        b_str_type: 1,
        ireguler: ma.pps_ireguler.clone(),
        b_sentence_end: false,
    };
    let n_morph_cnt = route.len();
    if n_morph_cnt == 0 {
        return word;
    }
    if n_morph_cnt == 1
        && ma.ps_morpheme[route[0] as usize].ch_tag == b':'
        && let Some((tag, n_cmp)) = process_to_pronoun(ma, d)
    {
        let mut m1 = MaMorph {
            ch_tag: b'9',
            pyogi: ma.pch_pyogi_str[..ma.w_pyogi_len as usize - n_cmp].to_vec(),
            cvc: Vec::new(),
            prob: ma.ps_morpheme[route[0] as usize].d_accum_prob,
            b_merged: false,
        };
        m1.cvc = conv_pyogi_to_cvc(&m1.pyogi);
        let mut m2 = MaMorph {
            ch_tag: tag,
            pyogi: ma.pch_pyogi_str[ma.w_pyogi_len as usize - n_cmp..].to_vec(),
            cvc: Vec::new(),
            prob: ma.ps_morpheme[route[0] as usize].d_accum_prob,
            b_merged: false,
        };
        m2.cvc = conv_pyogi_to_cvc(&m2.pyogi);
        word.morphs = vec![m1, m2];
        return word;
    }
    let mut pairs: Vec<(i32, usize)> = Vec::with_capacity(route.len());
    let mut i = route.len() as isize - 1;
    loop {
        let node_idx = route[i as usize] as usize;
        let node = &ma.ps_morpheme[node_idx];
        let mut m = MaMorph {
            ch_tag: node.ch_tag,
            pyogi: node.pyogi.clone(),
            cvc: Vec::new(),
            prob: node.d_accum_prob,
            b_merged: false,
        };
        m.cvc = morph_source_cvc(ma, node);
        let is_end = node.b_next_morph_flag == b'o';
        pairs.push((node_idx as i32, word.morphs.len()));
        word.morphs.push(m);
        if is_end {
            break;
        }
        i -= 1;
        if i < 0 {
            break;
        }
    }
    apply_copula_bnida_assimilation(&mut word.morphs);
    merge_contracted_syllables(ma, &mut word.morphs, &pairs);
    let _ = klp;
    word
}

fn apply_copula_bnida_assimilation(morphs: &mut [MaMorph]) {
    let n = morphs.len();
    if n < 2 {
        return;
    }
    for i in 0..n - 1 {
        let a = &morphs[i];
        let b = &morphs[i + 1];
        if a.ch_tag == b'@' && a.pyogi == b"iL" && b.ch_tag == b'^' && b.pyogi == b"Bnida" {
            let cvc_a = &mut morphs[i].cvc;
            let a_len = cvc_a.len();
            if a_len >= 3 && cvc_a[a_len - 1] == 9 {
                cvc_a[a_len - 1] = 17;
            }
            let cvc_b = &mut morphs[i + 1].cvc;
            if cvc_b.len() >= 3 && cvc_b[..3] == [1, 1, 19] {
                cvc_b.drain(0..3);
            }
        }
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_to_pronoun(ma: &MorphAnalBuf, d: &KmaDicts) -> Option<(u8, usize)> {
    let w_pyogi_len = ma.w_pyogi_len as usize;
    let mut n_cmp = 12usize;
    while n_cmp > 0 {
        if w_pyogi_len <= n_cmp {
            n_cmp -= 1;
            continue;
        }
        let start = w_pyogi_len - n_cmp;
        let c = ma.pch_pyogi_str[start.saturating_sub(1)];
        if c != 0 && !b"gqndflmbrsvjzcktph".contains(&c) {
            let pos = d.to_struct_search(&ma.pch_pyogi_str[start..], n_cmp);
            if pos != 0 {
                return Some((pos, n_cmp));
            }
        }
        n_cmp -= 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn morph_anal_buf_copies_gb_end_tag_from_klp() {
        let mut klp = KlpState {
            words: Vec::new(),
            gb_ini_tag: b'j',
            gb_ini_ll_tag: b'j',
            gb_end_tag: b'L',
            gsch_ini_str: Vec::new(),
            gsch_ini_ll_str: Vec::new(),
            gn_total_str_count: 1,
        };
        let ma = MorphAnalBuf::new(&klp);
        assert_eq!(ma.gb_end_tag, b'L');
        klp.gb_end_tag = b'l';
        let ma2 = MorphAnalBuf::new(&klp);
        assert_eq!(ma2.gb_end_tag, b'l');
    }

    #[test]
    fn nugu_analyze_tags_match_oracle() {
        let dir = std::path::PathBuf::from(
            std::env::var("KTTSDB_DIR")
                .expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
        )
        .join("KLangDic");
        if !dir.join("KMPADict").join("kmorph.dic").exists() {
            eprintln!("skip: no dictionary ({})", dir.display());
            return;
        }
        let ctx = crate::load_kma_dicts(&dir).expect("failed to load KMA dictionary");
        let words = crate::analyze(&ctx, "누구세요?").expect("analysis");
        assert_eq!(words.len(), 1);
        let pos: Vec<u8> = words[0].morphs.iter().map(|m| m.pos[0]).collect();
        assert_eq!(pos, vec![b'F', b'H', b'^']);
    }
}
