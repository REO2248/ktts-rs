use crate::consts::{
    CAND_MAX, CEP_DIV, CTX_INIT, ENG_SAT, ENG_W_HIGH, ENG_W_LOW, GROUP_TYPE_LIMIT, HALF,
    LEN_HIGH_RATIO, LEN_LOW_RATIO, LEN_SCALE, MIN_PITCH, PANALTY, PENALTY_5, PENALTY_10,
    PHONE_EDGE_MISMATCH, PHONE_MISMATCH, PITCH_W_END, PITCH_W_HIGH, PITCH_W_LOW, PITCH_W_START,
    SCORE_LIMIT,
};
use crate::context::{Letter, Phrase};
use crate::tables::{
    GSCH_CHO_STATUS, GSCH_PHONE_GROUP, GSW_CENTER_INDEX, GSW_CHO_NEXT_INDEX, GSW_CHO_PREV_INDEX,
    GSW_GROUP_ADDRESS, GSW_HUBO_INDEX, GSW_JONG_CENTER_INDEX, GSW_JONG_NEXT_INDEX,
    GSW_JONG_PREV_INDEX, GSW_JUNG_NEXT_INDEX, GSW_JUNG_PREV_INDEX,
};
use ktts_dict::synthdb::{PhoneDict, SynthGroupIdx, SynthIdx};

#[derive(Debug, Clone, Copy)]
pub struct TriInfo {
    pub w_tri_phone_no: u16,
    pub w_type_no: u16,
    pub score: f32,
    pub f_type_index: u8,
}

#[derive(Debug, Clone)]
pub struct TriHubo {
    pub inno: Vec<TriInfo>,
    pub sort: Vec<u16>,
    pub pcm_length: u16,
}

impl TriHubo {
    const fn new() -> Self {
        Self {
            inno: Vec::new(),
            sort: Vec::new(),
            pcm_length: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SelfInfo {
    pub f_exist_pos: u8,
    pub ch_word_pos: u8,
    pub w_letter_pos: u16,
    pub w_phone_pos: u16,
    pub f_index_pos: u8,
    pub f_chosong_type: u8,
    pub f_dict_type: u8,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PhoneResult {
    pub r_length_value: f32,
    pub b_accent_flag: u8,
    pub ch_start_pitch: u8,
    pub ch_end_pitch: u8,
    pub ch_ave_pitch: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct BestPhone {
    pub unit_no: u16,
    pub type_no: u16,
    pub f_type_index: u8,
}

#[inline]
const fn is_special(x: u8) -> bool {
    (x.wrapping_add(0xa0)) < 2
}

#[inline]
const fn is_special2(x: u8) -> bool {
    (x.wrapping_add(0x9e)) < 2
}

#[must_use]
pub const fn chosong_type(w1: u16, w2: u16) -> i8 {
    if w1 < 0x15 {
        let u = 1u32 << (w1 & 0x1f);
        if (u & 0x10_4224) == 0 {
            if (u & 0x448) != 0 {
                return 2;
            }
            if (u & 0xf9800) != 0 {
                return 1;
            }
        } else if w2 > 0x5f {
            return 1;
        } else if w1 != 0x14 {
            return 2;
        }
    }
    0
}

#[derive(Debug, Clone, Copy)]
pub struct DictRec<'a> {
    pub unit_no: u16,
    pub rec: &'a PhoneDict,
}

#[derive(Debug)]
pub struct Selection {
    pub best: Vec<Vec<Option<BestPhone>>>,
}

fn seven_of(letter: &Letter, pos: u16) -> [i16; 7] {
    let arr = match pos {
        0 => &letter.sch_cho,
        1 => &letter.sch_jung,
        _ => &letter.sch_jong,
    };
    let mut out = [0i16; 7];
    for i in 0..7 {
        out[i] = i16::from(arr[i]);
    }
    out
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
pub fn center_phone_index(seven: &[i16; 7], out: &mut [i16; 2], flag: bool) -> bool {
    let sw = seven_phone_copy(seven);
    let c = sw[3];
    if c < 0x20 {
        if (GSW_CENTER_INDEX[c as usize] as i16) > -1
            && (GSW_CHO_PREV_INDEX[sw[2] as usize] as i16) > -1
            && (GSW_CHO_NEXT_INDEX[sw[4] as usize] as i16) > -1
        {
            let mut v = i32::from(GSW_CENTER_INDEX[c as usize]) * 0x1a9;
            v += i32::from(GSW_CHO_PREV_INDEX[sw[2] as usize]) * 0x11;
            v += i32::from(GSW_CHO_NEXT_INDEX[sw[4] as usize]);
            out[0] = v as i16;
            return true;
        }
    } else if c < 0x40 {
        let mut sw2 = sw;
        let s1 = sw2[1];
        if s1 < 0x60 && s1 != 0x48 && s1 != 0x42 && s1 != 0x53 {
            let p = sw2[2];
            if p == 5 || p == 2 || p == 9 || p == 0xe {
                sw2[2] = p + 1;
            }
        }
        let ci = GSW_CENTER_INDEX[c as usize] as i16;
        if ci > -1
            && (GSW_JUNG_PREV_INDEX[sw2[2] as usize] as i16) > -1
            && (GSW_JUNG_NEXT_INDEX[sw2[4] as usize] as i16) > -1
        {
            if flag {
                out[1] = (i32::from(ci) * 0x24
                    + 0x2fd
                    + i32::from(GSW_JUNG_NEXT_INDEX[sw2[4] as usize]))
                    as i16;
                out[0] =
                    (i32::from(ci) * 0x2d + i32::from(GSW_JUNG_PREV_INDEX[sw2[2] as usize])) as i16;
            } else {
                let mut v = i32::from(ci) * 0x654 + 0x2134;
                v += i32::from(GSW_JUNG_PREV_INDEX[sw2[2] as usize]) * 0x24;
                v += i32::from(GSW_JUNG_NEXT_INDEX[sw2[4] as usize]);
                out[0] = v as i16;
            }
            return true;
        }
    } else if c < 0x60 {
        let cj = GSW_JONG_CENTER_INDEX[(c - 0x40) as usize] as i16;
        if cj > -1
            && (GSW_JONG_PREV_INDEX[sw[2] as usize] as i16) > -1
            && (GSW_JONG_NEXT_INDEX[sw[4] as usize] as i16) > -1
        {
            let mut v = i32::from(GSW_CENTER_INDEX[c as usize]) * 0x286 - 0x7338;
            v += i32::from(GSW_JONG_PREV_INDEX[sw[2] as usize]) * 0x26;
            v += i32::from(GSW_JONG_NEXT_INDEX[sw[4] as usize]);
            out[0] = v as i16;
            return true;
        }
    }
    false
}

#[expect(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn center_phone_index_unchecked(seven: &[i16; 7]) -> i64 {
    let c = seven[3];
    if c < 0x20 {
        let mut v = i64::from(GSW_CENTER_INDEX[c as usize] as i16) * 0x1a9;
        v += i64::from(GSW_CHO_PREV_INDEX[seven[2] as usize] as i16) * 0x11;
        v += i64::from(GSW_CHO_NEXT_INDEX[seven[4] as usize] as i16);
        v
    } else if c < 0x40 {
        let mut sw = *seven;
        let s1 = sw[3];
        if s1 < 0x60 && s1 != 0x48 && s1 != 0x42 && s1 != 0x53 {
            let p = sw[2];
            if p == 5 || p == 2 || p == 9 || p == 0xe {
                sw[2] = p + 1;
            }
        }
        let mut v = i64::from(GSW_CENTER_INDEX[c as usize] as i16) * 0x654 + 0x2134;
        v += i64::from(GSW_JUNG_PREV_INDEX[sw[2] as usize] as i16) * 0x24;
        v += i64::from(GSW_JUNG_NEXT_INDEX[sw[4] as usize] as i16);
        v
    } else if c < 0x60 {
        let mut v = i64::from(GSW_CENTER_INDEX[c as usize] as i16) * 0x286 - 0x7338;
        v += i64::from(GSW_JONG_PREV_INDEX[seven[2] as usize] as i16) * 0x26;
        v += i64::from(GSW_JONG_NEXT_INDEX[seven[4] as usize] as i16);
        v
    } else {
        0x96de
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn seven_phone_copy(src: &[i16; 7]) -> [i16; 7] {
    let mut dst = [0i16; 7];
    for i in 0..7 {
        let s = src[i];
        if i > 0 && s == 7 && src[i - 1] != 0x49 && src[i - 1] != 0x60 {
            dst[i] = 0x1b;
            continue;
        }
        if i < 6 && s == 0x57 && src[i + 1] > 0x1f && src[i + 1] < 0x60 {
            dst[i] = 0x0d;
        } else {
            dst[i] = s;
        }
    }
    let center = 7 / 2;
    if (dst[center - 1].wrapping_sub(0x62) as u16) < 2 && center - 1 > 0 {
        let mut i = center - 1;
        while i > 0 {
            dst[i] = dst[i - 1];
            i -= 1;
        }
    }
    if (dst[center + 1].wrapping_sub(0x62) as u16) < 2 && center + 1 < 6 {
        let mut i = center + 1;
        while i < 6 {
            dst[i] = dst[i + 1];
            i += 1;
        }
    }
    dst
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn dict_phone_index(rec: &PhoneDict, seven: &[i16; 7]) -> bool {
    let mode = rec.ch_phone_mode;
    let w1 = seven[4];
    let w2 = seven[5];
    if mode == (mode / 10) * 10 {
        if w1 == 0x48 || w1 == 0x42 || w1 == 0x53 {
            if w1 != i16::from(rec.phones[3]) {
                return false;
            }
            if (w2.wrapping_sub(0x60) as u16) < 2 || w2 == 100 {
                if is_special2(rec.phones[4]) {
                    return true;
                }
                if w2 != i16::from(rec.phones[4]) {
                    return false;
                }
            }
            if w2 == 0x65 && rec.phones[4] != b'e' {
                return false;
            }
            if (w2.wrapping_sub(0x62) as u16) < 2 {
                return true;
            }
            return true;
        }
    } else if w1 == 0x48 || w1 == 0x42 || w1 == 0x53 {
        if mode > 9 && seven[2] < 0x62 {
            return false;
        }
        if w1 != i16::from(rec.phones[3]) {
            return false;
        }
        if (w2.wrapping_sub(0x62) as u16) > 1 {
            return false;
        }
        return true;
    }
    if mode == 0 {
        if seven[2] < 0x62
            && w1 != 0x62
            && w1 != 99
            && (((w1.wrapping_sub(0x60) as u16) > 1 && w1 != 100) || w1 == i16::from(rec.phones[3]))
        {
            if w1 != 0x65 {
                return true;
            }
            return rec.phones[3] == b'e';
        }
    } else if mode == 1 || mode == 2 {
        return seven[4] > 0x61;
    } else if mode == 0x14 || mode == 0x0a {
        return seven[2] > 0x61;
    } else if (mode == 0x0b || mode == 0x0c || mode == 0x15 || mode == 0x16) && seven[2] > 0x61 {
        return seven[4] > 0x61;
    }
    false
}

fn dict_phone_search(recs: &[PhoneDict], seven: &[i16; 7], mask: &mut [u8]) -> (usize, bool) {
    let mut n = 0usize;
    for (i, r) in recs.iter().enumerate() {
        mask[i] = 0;
        if dict_phone_index(r, seven) {
            n += 1;
            mask[i] = 1;
        }
    }
    (n, n >= CAND_MAX)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
pub fn phone_group_search(seven: &[i16; 7], tri: &mut i16) -> bool {
    let seven = seven_phone_copy(seven);
    let i3 = 3;
    let s4 = seven[i3];
    if s4 > 0x1f {
        if s4 > 0x3f && s4 < 0x60 {
            let mut v = i32::from(GSW_GROUP_ADDRESS[0x1f]);
            *tri = v as i16;
            if seven[i3 + 1] < 0x60 {
                v += 4;
            }
            v += i32::from(GSW_CENTER_INDEX[seven[i3] as usize]);
            *tri = v as i16;
            return v != -1;
        }
        *tri = -1;
        return false;
    }
    let s2 = i32::from(GSW_GROUP_ADDRESS[s4 as usize]);
    *tri = s2 as i16;
    if (s4 - 2) > 0x19 {
        *tri = -1;
        return false;
    }
    if s4 < 2 {
        *tri = -1;
        return false;
    }
    let c1 = GSCH_CHO_STATUS[(s4 - 2) as usize];
    let hubo = |next: i16| -> i32 { i32::from(GSW_HUBO_INDEX[next as usize]) };
    let v = match c1 {
        1 => hubo(seven[i3 + 1]),
        2 => {
            *tri = -1;
            return false;
        }
        0 => {
            if seven[i3 - 1] > 0x5f {
                hubo(seven[i3 + 1])
            } else {
                hubo(seven[i3 + 1]) + 7
            }
        }
        _ => {
            return s2 != -1;
        }
    };
    if v != -1 {
        *tri = (v + s2) as i16;
        return v + s2 != -1;
    }
    *tri = -1;
    false
}

fn unit_records(idx: &SynthIdx, unit_no: u16) -> Option<&[PhoneDict]> {
    idx.units
        .get(unit_no as usize)
        .map(|u| u.records.as_slice())
}

#[allow(clippy::needless_range_loop)]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn phone_type_hubo_search(
    ctx: &SynthCtx,
    hubo: &mut TriHubo,
    seven: &[i16; 7],
    f_mode: bool,
) -> bool {
    let mut w_value = [0i16; 2];
    if !center_phone_index(seven, &mut w_value, false) {
        return false;
    }
    let unit_no = w_value[0] as u16;
    let Some(recs) = unit_records(ctx.idx, unit_no) else {
        return false;
    };
    if recs.is_empty() {
        return false;
    }
    let size = recs.len();
    let mut mask = vec![0u8; size];
    let (n, found_enough) = dict_phone_search(recs, seven, &mut mask);
    if !found_enough && !f_mode {
        return false;
    }
    for i in 0..size {
        if mask[i] != 0 {
            hubo.inno.push(TriInfo {
                w_tri_phone_no: unit_no,
                w_type_no: i as u16,
                score: 0.0,
                f_type_index: 0,
            });
        }
    }
    let _ = n;
    true
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn phone_type_search(ctx: &SynthCtx, hubo: &mut TriHubo, seven: &[i16; 7], f_mode: bool) -> bool {
    let found = phone_type_hubo_search(ctx, hubo, seven, f_mode);
    if !found && !f_mode {
        let mut sw = *seven;
        let s1 = sw[4];
        if (sw[2].wrapping_sub(0x62) as u16) < 2 || s1 == 99 || s1 == 0x62 {
            if (sw[2].wrapping_sub(0x62) as u16) < 2 {
                sw[2] = 0x61;
            }
            if (sw[4].wrapping_sub(0x62) as u16) < 2 {
                sw[4] = 0x61;
            }
            if (sw[5].wrapping_sub(0x62) as u16) < 2
                && (sw[4] == 0x48 || sw[4] == 0x42 || sw[4] == 0x53)
            {
                sw[5] = 0x61;
            }
            phone_type_hubo_search(ctx, hubo, &sw, false);
        } else if s1 == 100 {
            let mut sw2 = *seven;
            sw2[4] = 0x60;
            sw2[5] = seven[5];
            phone_type_hubo_search(ctx, hubo, &sw2, false);
        }
    }
    hubo.inno.len() > CAND_MAX
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
fn hubo_phone_search(ctx: &SynthCtx, hubo: &mut TriHubo, mut seven: [i16; 7]) -> usize {
    if phone_type_search(ctx, hubo, &seven, false) {
        return 1;
    }
    if seven[2] == 0x61 {
        seven[2] = 0x60;
        if seven[4] == 0x61 {
            seven[4] = 0x60;
        }
        if phone_type_search(ctx, hubo, &seven, false) {
            return 1;
        }
    } else if seven[4] == 0x61 {
        seven[4] = 0x60;
        if phone_type_search(ctx, hubo, &seven, false) {
            return 1;
        }
    } else if seven[5] == 0x61 && (seven[4] == 0x48 || seven[4] == 0x42 || seven[4] == 0x53) {
        seven[5] = 0x60;
        if phone_type_search(ctx, hubo, &seven, false) {
            return 1;
        }
    }
    if (seven[3] as u16).wrapping_sub(0x20) < 0x20 {
        let mut sw = seven;
        phone_type_search(ctx, hubo, &sw, true);
        let g = GSCH_PHONE_GROUP[sw[2] as usize];
        let orig2 = sw[2];
        let mut i = 0usize;
        while i < 0x66 {
            if GSCH_PHONE_GROUP[i] == g && (GSCH_PHONE_GROUP[i] as i8) >= 0 && i != orig2 as usize {
                sw[2] = i as i16;
                i += 1;
                phone_type_search(ctx, hubo, &sw, true);
                if i == 0x66 {
                    break;
                }
            } else {
                i += 1;
            }
        }
        if hubo.inno.len() > CAND_MAX {
            return 1;
        }
        if sw[5] < 0x20 {
            hubo.inno.clear();
            let mut i = 0usize;
            while i < 0x66 {
                if (GSCH_PHONE_GROUP[i] as i8) == (g as i8) && (GSCH_PHONE_GROUP[i] as i8) >= 0 {
                    let mut j = 0usize;
                    sw[2] = i as i16;
                    let g2 = GSCH_PHONE_GROUP[sw[5] as usize] as i8;
                    loop {
                        while j < 0x20 && (GSCH_PHONE_GROUP[j] as i8) == g2 && g2 >= 0 {
                            sw[5] = j as i16;
                            j += 1;
                            phone_type_search(ctx, hubo, &sw, true);
                        }
                        j += 1;
                        if j >= 0x20 {
                            break;
                        }
                    }
                }
                i += 1;
            }
        }
        if !hubo.inno.is_empty() {
            return 1;
        }
    }
    let mut tri = 0i16;
    if phone_group_search(&seven, &mut tri) {
        let gi = tri as usize;
        let Some(group) = ctx.groups.groups.get(gi) else {
            return 1;
        };
        let total: u32 = group
            .phones
            .iter()
            .map(|&u| {
                ctx.idx
                    .units
                    .get(u as usize)
                    .map_or(0, |x| x.records.len() as u32)
            })
            .sum();
        if total != 0 {
            hubo.inno.clear();
            let mut count = 0usize;
            for &u in &group.phones {
                let Some(recs) = unit_records(ctx.idx, u) else {
                    continue;
                };
                if recs.is_empty() {
                    continue;
                }
                if total < 1001 {
                    for (ti, _) in recs.iter().enumerate() {
                        if count >= GROUP_TYPE_LIMIT {
                            break;
                        }
                        hubo.inno.push(TriInfo {
                            w_tri_phone_no: u,
                            w_type_no: ti as u16,
                            score: 0.0,
                            f_type_index: 0,
                        });
                        count += 1;
                    }
                } else {
                    let mut mask = vec![0u8; recs.len()];
                    let _ = dict_phone_search(recs, &seven, &mut mask);
                    for (ti, m) in mask.iter().enumerate() {
                        if *m != 0 {
                            if count >= GROUP_TYPE_LIMIT {
                                break;
                            }
                            hubo.inno.push(TriInfo {
                                w_tri_phone_no: u,
                                w_type_no: ti as u16,
                                score: 0.0,
                                f_type_index: 0,
                            });
                            count += 1;
                        }
                    }
                }
            }
        }
        if !hubo.inno.is_empty() {
            return 1;
        }
    }
    if seven[2] == 99 || seven[4] == 99 {
        let mut sw = seven;
        if seven[2] == 99 {
            sw[2] = seven[1];
            sw[1] = seven[0];
        }
        if seven[4] == 99 {
            sw[4] = seven[5];
            sw[5] = seven[6];
        }
        if phone_type_search(ctx, hubo, &sw, false) {
            return 1;
        }
    }
    let s2 = seven[2];
    if s2 < 0x62 {
        let mut sw = seven;
        let s4 = seven[4];
        if s4 < 0x62 {
            if (s4.wrapping_sub(0x60) as u16) < 2 {
                sw[4] = if s4 == 0x61 { 0x60 } else { 0x61 };
                if phone_type_search(ctx, hubo, &sw, false) {
                    return 1;
                }
            }
        } else {
            sw[4] = 0x61;
            if phone_type_search(ctx, hubo, &sw, false) {
                return 1;
            }
        }
        sw = seven;
        if s2 >= 0x60 {
            sw[2] = if s2 == 0x61 { 0x60 } else { 0x61 };
            if phone_type_search(ctx, hubo, &sw, false) {
                return 1;
            }
        }
        if s2 == 0x61 {
            seven[2] = 0x60;
        }
    } else {
        seven[2] = 0x61;
        if seven[4] > 0x61 {
            seven[4] = 0x61;
        }
        if phone_type_search(ctx, hubo, &seven, false) {
            return 1;
        }
        if seven[2] == 0x61 {
            seven[2] = 0x60;
        }
    }
    if seven[4] == 0x61 {
        seven[4] = 0x60;
    }
    phone_type_search(ctx, hubo, &seven, false);
    let mut sw_value = [0i16; 2];
    let center = if center_phone_index(&seven, &mut sw_value, false) {
        i64::from(sw_value[0] as u16)
    } else {
        i64::from(center_phone_index_unchecked(&seven) as u16)
    };
    let mut lo = center - 5;
    let mut hi = center + 5;
    if hi > 0x96de {
        hi = 0x96de;
    }
    let mut l6c: i64 = 0;
    loop {
        let mut count = 0usize;
        if lo < hi {
            hubo.inno.clear();
            for u in lo.max(0)..hi {
                let u = u as u16;
                let Some(recs) = unit_records(ctx.idx, u) else {
                    continue;
                };
                if recs.is_empty() {
                    continue;
                }
                for (ti, _) in recs.iter().enumerate() {
                    if count >= GROUP_TYPE_LIMIT {
                        break;
                    }
                    hubo.inno.push(TriInfo {
                        w_tri_phone_no: u,
                        w_type_no: ti as u16,
                        score: 0.0,
                        f_type_index: 0,
                    });
                    count += 1;
                }
            }
        } else {
            count = 0;
        }
        if count > CAND_MAX {
            break;
        }
        lo = (center - 5) + l6c;
        l6c -= 5;
        if center + l6c < 0 {
            break;
        }
        hi = center - l6c;
        if hi > 0x96de {
            hi = 0x96de;
        }
    }
    1
}

#[expect(clippy::float_cmp, reason = "C port: exact cost tie comparison")]
fn token_heapify(
    arr: &mut [u16],
    scores: &[f32],
    pick_larger: bool,
    tie_right: bool,
    stop_elem_ge: bool,
) {
    let n = arr.len();
    let mut n_root = n / 2;
    while n_root > 0 {
        let s_var3 = arr[n_root - 1];
        if 2 * n_root <= n {
            let mut n_parent = n_root;
            let mut i_var8 = 2 * n_root;
            let pos;
            loop {
                let (chosen, i_var9) = if i_var8 < n {
                    let rl = scores[arr[i_var8 - 1] as usize];
                    let rr = scores[arr[i_var8] as usize];
                    let want = if pick_larger { rr > rl } else { rr < rl };
                    if want || (rr == rl && tie_right) {
                        (arr[i_var8], i_var8 + 1)
                    } else {
                        (arr[i_var8 - 1], i_var8)
                    }
                } else {
                    (arr[i_var8 - 1], i_var8)
                };
                let cr = scores[chosen as usize];
                let stop = if stop_elem_ge {
                    scores[s_var3 as usize] >= cr
                } else {
                    scores[s_var3 as usize] <= cr
                };
                if stop {
                    pos = n_parent - 1;
                    break;
                }
                arr[n_parent - 1] = chosen;
                i_var8 = i_var9 * 2;
                n_parent = i_var9;
                if i_var8 > n {
                    pos = n_parent - 1;
                    break;
                }
            }
            arr[pos] = s_var3;
        }
        n_root -= 1;
    }
}

#[expect(clippy::float_cmp, reason = "C port: exact cost tie comparison")]
fn token_heap_extract(
    arr: &mut [u16],
    scores: &[f32],
    n_need: usize,
    pick_larger: bool,
    tie_right: bool,
    stop_chosen_le: bool,
) {
    let n_total = arr.len();
    let mut n = n_total;
    let mut local_20 = n_total - 1;
    while n_total - n_need < n {
        let s_var3 = arr[local_20];
        arr[local_20] = arr[0];
        n -= 1;
        if 1 < n {
            let mut i_var8 = 2;
            let mut n_parent = 1;
            let f_var1 = scores[s_var3 as usize];
            let pos;
            loop {
                let (chosen, i_var9) = if i_var8 < n {
                    let rl = scores[arr[i_var8 - 1] as usize];
                    let rr = scores[arr[i_var8] as usize];
                    let want = if pick_larger { rr > rl } else { rr < rl };
                    if want || (rr == rl && tie_right) {
                        (arr[i_var8], i_var8 + 1)
                    } else {
                        (arr[i_var8 - 1], i_var8)
                    }
                } else {
                    (arr[i_var8 - 1], i_var8)
                };
                let cr = scores[chosen as usize];
                let stop = if stop_chosen_le {
                    cr <= f_var1
                } else {
                    cr >= f_var1
                };
                if stop {
                    pos = n_parent - 1;
                    break;
                }
                arr[n_parent - 1] = chosen;
                i_var8 = i_var9 * 2;
                n_parent = i_var9;
                if i_var8 > n {
                    pos = n_parent - 1;
                    break;
                }
            }
            arr[pos] = s_var3;
        }
        local_20 -= 1;
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn token_process(hubo: &mut TriHubo) {
    let n = hubo.inno.len();
    let scores: Vec<f32> = hubo.inno.iter().map(|e| e.score).collect();
    let mut order: Vec<u16> = (0..n as u16).collect();
    let (start, stop): (i64, i64) = if n < 31 {
        (n as i64 - 1, 0)
    } else if n > 60 {
        token_heapify(&mut order, &scores, true, true, true);
        token_heap_extract(&mut order, &scores, 30, true, false, true);
        (n as i64 - 1, n as i64 - 30)
    } else {
        token_heapify(&mut order, &scores, false, true, false);
        token_heap_extract(&mut order, &scores, n - 30, false, false, false);
        (29, 0)
    };
    let mut out: Vec<u16> = Vec::new();
    let mut i = start;
    while i >= stop {
        let ci = order[i as usize] as usize;
        let score = -hubo.inno[ci].score;
        if score <= SCORE_LIMIT {
            hubo.inno[ci].score = score;
            out.push(order[i as usize]);
        } else {
            break;
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    hubo.sort = out;
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn get_pitch_and_length(
    letter: &Letter,
    w_phone_pos: u16,
    f_index_pos: u8,
    result: &mut PhoneResult,
    pcm_length: &mut u16,
    n_flag: u8,
) -> u8 {
    let mut r_length = f32::from(letter.ave_length[w_phone_pos as usize]);
    if r_length == 0.0 {
    } else if w_phone_pos == 1 && f_index_pos > 0 {
        r_length *= HALF;
        *pcm_length = crate::context::round(r_length) as u16;
    } else {
        *pcm_length = crate::context::round(r_length) as u16;
    }
    result.r_length_value = r_length;
    let mut l0 = i32::from(letter.ave_length[0]);
    let center = letter.sch_cho[3];
    if center < 0x20 && l0 > 0 && center != 7 && center != 4 && center != 13 && center != 8 {
        l0 = 0;
    }
    let l1 = i32::from(letter.ave_length[1]);
    let l2 = i32::from(letter.ave_length[2]);
    let s14 = l0 + l1;
    let s8 = l2 + s14;
    let (w_start, w_end) = match w_phone_pos {
        0 => (0i32, l0),
        1 => match f_index_pos {
            1 => (l0, l1 / 2 + l0),
            2 => (l1 / 2 + l0, s14),
            _ => (l0, s14),
        },
        _ => (s14, s8),
    };
    if w_start == w_end {
        result.b_accent_flag = 1;
        return 0;
    }
    result.b_accent_flag = 0;
    let i9 = (w_start * 12) / s8;
    let i10 = (w_end * 12) / s8 - 1;
    let i11 = i9.max(i10);
    match n_flag {
        0 => {
            let v = i32::from(letter.ave_pitch[i9 as usize] as i8) - i32::from(MIN_PITCH);
            v as u8
        }
        2 => {
            let v = i32::from(letter.ave_pitch[i11 as usize] as i8) - i32::from(MIN_PITCH);
            v as u8
        }
        _ => {
            let count = (i11 - i9) + 1;
            if count <= 0 {
                return 0;
            }
            let sum: i32 = (i9..=i11)
                .map(|k| i32::from(letter.ave_pitch[k as usize]))
                .sum();
            let avg = (sum / count) as i16;
            let v = i32::from(avg as i8) - i32::from(MIN_PITCH);
            v as u8
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[must_use]
#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
#[expect(
    clippy::suboptimal_flops,
    reason = "C port: float-op order kept bit-exact (mul_add/FMA would change rounding)"
)]
pub fn get_inno_score(
    ctx: &SynthCtx,
    cand: &PhoneDict,
    one: &SelfInfo,
    result: PhoneResult,
    letter: &Letter,
) -> f32 {
    let mode = cand.ch_phone_mode;
    let seven: &[u8; 7] = match one.w_phone_pos {
        1 => &letter.sch_jung,
        2 => &letter.sch_jong,
        _ => &letter.sch_cho,
    };
    let (mut f4, local_2e, local_30, local_5d, local_2f, state_machine) =
        mode_state_costs(cand, *seven, mode, one.w_phone_pos);

    if state_machine {
        phone_class_state_machine(&mut f4, cand, *seven, local_2f, local_5d, mode);
    }

    extra_penalties(&mut f4, one, cand, mode, local_2e, local_30);

    let (f8, _, _) = length_cost(one, result, cand);

    let f9 = pitch_cost(ctx, result, cand);

    let ct = chosong_type(u16::from(seven[3]), u16::from(seven[2]));
    let (f_cho_w, f_pitch_w) = if ct < 1 {
        (1.0f32, PENALTY_10)
    } else if cand.phones[3] == seven[4] {
        (HALF, 1.0)
    } else {
        (1.0, 1.0)
    };

    f4 * f_cho_w * PANALTY[3] + f8 * LEN_SCALE * PANALTY[2] + f_pitch_w * f9 * PANALTY[4]
}
#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
fn mode_state_costs(
    cand: &PhoneDict,
    seven: [u8; 7],
    mode: i8,
    w_phone_pos: u16,
) -> (f32, u8, u8, u8, u8, bool) {
    let mut f4 = CTX_INIT;
    let (local_2e, local_30, local_5d, local_2f);
    let default_case = if w_phone_pos == 0 {
        mode >= 0x15
    } else {
        mode > 0x14
    };
    let mut state_machine = true;
    if default_case {
        let c3 = cand.phones[2];
        let t3 = seven[3];
        if c3 == t3 {
            f4 = 0.0;
        }
        if seven[2] < 0x62 {
            f4 += PHONE_MISMATCH;
        }
        if cand.phones[1] != seven[1] {
            f4 += PHONE_MISMATCH;
        }
        if seven[4] < 0x62 {
            f4 += PHONE_MISMATCH;
        }
        if cand.phones[3] != seven[5] {
            f4 += PHONE_MISMATCH;
        }
        if cand.phones[0] != seven[0] {
            f4 += PHONE_EDGE_MISMATCH;
        }
        local_2e = c3;
        local_30 = t3;
        local_5d = cand.phones[1];
        local_2f = cand.phones[3];
        if cand.phones[4] != seven[6] {
            f4 += PHONE_EDGE_MISMATCH;
        }
        if cand.phones[2] < 0x20 || cand.phones[2] >= 0x40 {
            state_machine = false;
        }
    } else {
        match mode {
            0 => {
                let c3 = cand.phones[2];
                let t3 = seven[3];
                if c3 == t3 {
                    f4 = 0.0;
                }
                if cand.phones[1] != seven[2] {
                    f4 += PHONE_MISMATCH;
                }
                if seven[4] < 0x60 && seven[4] != cand.phones[3] {
                    f4 += PHONE_MISMATCH;
                }
                if cand.phones[0] != seven[1] {
                    f4 += PHONE_EDGE_MISMATCH;
                }
                local_2e = c3;
                local_30 = t3;
                local_5d = cand.phones[1];
                local_2f = cand.phones[3];
                if cand.phones[4] != seven[5] {
                    f4 += PHONE_EDGE_MISMATCH;
                }
                if cand.phones[2] < 0x20 || cand.phones[2] >= 0x40 {
                    state_machine = false;
                }
            }
            1 | 2 => {
                let c3 = cand.phones[2];
                let t3 = seven[3];
                if c3 == t3 {
                    f4 = 0.0;
                }
                if cand.phones[1] != seven[2] {
                    f4 += PHONE_MISMATCH;
                }
                if seven[4] < 0x62 {
                    f4 += PHONE_MISMATCH;
                }
                if cand.phones[3] != seven[5] {
                    f4 += PHONE_MISMATCH;
                }
                if cand.phones[0] != seven[1] {
                    f4 += PHONE_EDGE_MISMATCH;
                }
                local_2e = c3;
                local_30 = t3;
                local_5d = cand.phones[1];
                local_2f = cand.phones[3];
                if cand.phones[4] != seven[6] {
                    f4 += PHONE_EDGE_MISMATCH;
                }
                if cand.phones[2] < 0x20 || cand.phones[2] >= 0x40 {
                    state_machine = false;
                }
            }
            10 | 0x14 => {
                let c3 = cand.phones[2];
                let t3 = seven[3];
                if c3 == t3 {
                    f4 = 0.0;
                }
                if cand.phones[3] != seven[4] {
                    f4 += PHONE_MISMATCH;
                }
                if seven[2] < 0x62 {
                    f4 += PHONE_MISMATCH;
                }
                if cand.phones[1] != seven[1] {
                    f4 += PHONE_MISMATCH;
                }
                if cand.phones[0] != seven[0] {
                    f4 += PHONE_EDGE_MISMATCH;
                }
                local_2e = c3;
                local_30 = t3;
                local_5d = cand.phones[1];
                local_2f = cand.phones[3];
                if cand.phones[4] != seven[5] {
                    f4 += PHONE_EDGE_MISMATCH;
                }
                if cand.phones[2] < 0x20 || cand.phones[2] >= 0x40 {
                    state_machine = false;
                }
            }
            _ => {
                let c3 = cand.phones[2];
                let t3 = seven[3];
                if c3 == t3 {
                    f4 = 0.0;
                }
                if seven[2] < 0x62 {
                    f4 += PHONE_MISMATCH;
                }
                if cand.phones[1] != seven[1] {
                    f4 += PHONE_MISMATCH;
                }
                if seven[4] < 0x62 {
                    f4 += PHONE_MISMATCH;
                }
                if cand.phones[3] != seven[5] {
                    f4 += PHONE_MISMATCH;
                }
                if cand.phones[0] != seven[0] {
                    f4 += PHONE_EDGE_MISMATCH;
                }
                local_2e = c3;
                local_30 = t3;
                local_5d = cand.phones[1];
                local_2f = cand.phones[3];
                if cand.phones[4] != seven[6] {
                    f4 += PHONE_EDGE_MISMATCH;
                }
                if cand.phones[2] < 0x20 || cand.phones[2] >= 0x40 {
                    state_machine = false;
                }
            }
        }
    }
    (f4, local_2e, local_30, local_5d, local_2f, state_machine)
}
#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
#[expect(
    clippy::branches_sharing_code,
    reason = "C port: shared code in branches kept as-is (extraction would change control flow)"
)]
fn phone_class_state_machine(
    f4: &mut f32,
    cand: &PhoneDict,
    seven: [u8; 7],
    local_2f: u8,
    local_5d: u8,
    mode: i8,
) {
    let c6 = seven[4];
    let local_2f_v = local_2f;
    if (c6.wrapping_sub(0x20) >= 0x40) && local_2f_v > 0x3f && local_2f_v < 0x60 {
        *f4 += PENALTY_10;
    }
    if c6.wrapping_sub(0x40) < 0x20 {
        if (local_2f_v.wrapping_sub(0x20) < 0x20) || local_2f_v > 0x5f {
            *f4 += PENALTY_5;
        }
        if (c6 == b'H' || c6 == b'B' || c6 == b'S')
            && local_2f_v != b'H'
            && local_2f_v != b'B'
            && local_2f_v != b'S'
        {
            *f4 += PENALTY_5;
        }
    }
    let c2 = seven[2];
    let n_real_prev: i32;
    let n_dict_prev: i32;
    let n_real_next: i32;
    let mut ivar7: i32;
    let mut l_813c3 = false;
    if is_special(c2) || (is_special(seven[1]) && c2 < 0x20) {
        n_real_prev = 1;
        if is_special(local_5d) {
            n_dict_prev = 1;
            if is_special(c6) {
                n_real_next = 1;
                if is_special(local_2f_v) {
                    ivar7 = 1;
                } else {
                    l_813c3 = true;
                    ivar7 = 0;
                }
            } else {
                let c2b = seven[5];
                if is_special(c2b) && c6 > 0x3f {
                    n_real_next = 1;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c6) || c6 == b'e' {
                    n_real_next = 2;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c2b) {
                    if c6 > 0x3f {
                        n_real_next = 2;
                    } else {
                        n_real_next = 0;
                    }
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if c6 == b'd' || (c2b == b'd' && c6 > 0x3f) {
                    n_real_next = 10;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else {
                    n_real_next = 0;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                }
            }
        } else if local_5d == b'e' {
            n_dict_prev = 2;
            if is_special(c6) {
                n_real_next = 1;
                if is_special(local_2f_v) {
                    ivar7 = 1;
                } else {
                    l_813c3 = true;
                    ivar7 = 0;
                }
            } else {
                let c2b = seven[5];
                if is_special(c2b) && c6 > 0x3f {
                    n_real_next = 1;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c6) || c6 == b'e' {
                    n_real_next = 2;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c2b) {
                    n_real_next = if c6 > 0x3f { 2 } else { 0 };
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if c6 == b'd' || (c2b == b'd' && c6 > 0x3f) {
                    n_real_next = 10;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else {
                    n_real_next = 0;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                }
            }
        } else if is_special(cand.phones[0]) && local_5d < 0x20 {
            n_dict_prev = 1;
            if is_special(c6) {
                n_real_next = 1;
                if is_special(local_2f_v) {
                    ivar7 = 1;
                } else {
                    l_813c3 = true;
                    ivar7 = 0;
                }
            } else {
                let c2b = seven[5];
                if is_special(c2b) && c6 > 0x3f {
                    n_real_next = 1;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c6) || c6 == b'e' {
                    n_real_next = 2;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c2b) {
                    n_real_next = if c6 > 0x3f { 2 } else { 0 };
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if c6 == b'd' || (c2b == b'd' && c6 > 0x3f) {
                    n_real_next = 10;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else {
                    n_real_next = 0;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                }
            }
        } else if mode > 9
            || is_special2(cand.phones[0])
            || (cand.phones[0] == b'e' && local_5d < 0x20)
        {
            n_dict_prev = 2;
            if is_special(c6) {
                n_real_next = 1;
                if is_special(local_2f_v) {
                    ivar7 = 1;
                } else {
                    l_813c3 = true;
                    ivar7 = 0;
                }
            } else {
                let c2b = seven[5];
                if is_special(c2b) && c6 > 0x3f {
                    n_real_next = 1;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c6) || c6 == b'e' {
                    n_real_next = 2;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c2b) {
                    n_real_next = if c6 > 0x3f { 2 } else { 0 };
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if c6 == b'd' || (c2b == b'd' && c6 > 0x3f) {
                    n_real_next = 10;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else {
                    n_real_next = 0;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                }
            }
        } else {
            n_dict_prev = 0;
            if is_special(c6) {
                n_real_next = 1;
                if is_special(local_2f_v) {
                    ivar7 = 1;
                } else {
                    l_813c3 = true;
                    ivar7 = 0;
                }
            } else {
                let c2b = seven[5];
                if is_special(c2b) && c6 > 0x3f {
                    n_real_next = 1;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c6) || c6 == b'e' {
                    n_real_next = 2;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c2b) {
                    n_real_next = if c6 > 0x3f { 2 } else { 0 };
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if c6 == b'd' || (c2b == b'd' && c6 > 0x3f) {
                    n_real_next = 10;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else {
                    n_real_next = 0;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                }
            }
        }
    } else {
        if c2 < 0x62 && (seven[1] < 0x62 || c2 > 0x1f) {
            n_real_prev = 0;
        } else {
            n_real_prev = 2;
        }
        if is_special(local_5d) {
            n_dict_prev = 1;
            if is_special(c6) {
                n_real_next = 1;
                if is_special(local_2f_v) {
                    ivar7 = 1;
                } else {
                    l_813c3 = true;
                    ivar7 = 0;
                }
            } else {
                let c2b = seven[5];
                if is_special(c2b) && c6 > 0x3f {
                    n_real_next = 1;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c6) || c6 == b'e' {
                    n_real_next = 2;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c2b) {
                    n_real_next = if c6 > 0x3f { 2 } else { 0 };
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if c6 == b'd' || (c2b == b'd' && c6 > 0x3f) {
                    n_real_next = 10;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else {
                    n_real_next = 0;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                }
            }
        } else if local_5d == b'e' {
            n_dict_prev = 2;
            if is_special(c6) {
                n_real_next = 1;
                if is_special(local_2f_v) {
                    ivar7 = 1;
                } else {
                    l_813c3 = true;
                    ivar7 = 0;
                }
            } else {
                let c2b = seven[5];
                if is_special(c2b) && c6 > 0x3f {
                    n_real_next = 1;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c6) || c6 == b'e' {
                    n_real_next = 2;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c2b) {
                    n_real_next = if c6 > 0x3f { 2 } else { 0 };
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if c6 == b'd' || (c2b == b'd' && c6 > 0x3f) {
                    n_real_next = 10;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else {
                    n_real_next = 0;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                }
            }
        } else if is_special(cand.phones[0]) && local_5d < 0x20 {
            n_dict_prev = 1;
            if is_special(c6) {
                n_real_next = 1;
                if is_special(local_2f_v) {
                    ivar7 = 1;
                } else {
                    l_813c3 = true;
                    ivar7 = 0;
                }
            } else {
                let c2b = seven[5];
                if is_special(c2b) && c6 > 0x3f {
                    n_real_next = 1;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c6) || c6 == b'e' {
                    n_real_next = 2;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c2b) {
                    n_real_next = if c6 > 0x3f { 2 } else { 0 };
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if c6 == b'd' || (c2b == b'd' && c6 > 0x3f) {
                    n_real_next = 10;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else {
                    n_real_next = 0;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                }
            }
        } else if mode > 9
            || is_special2(cand.phones[0])
            || (cand.phones[0] == b'e' && local_5d < 0x20)
        {
            n_dict_prev = 2;
            if is_special(c6) {
                n_real_next = 1;
                if is_special(local_2f_v) {
                    ivar7 = 1;
                } else {
                    l_813c3 = true;
                    ivar7 = 0;
                }
            } else {
                let c2b = seven[5];
                if is_special(c2b) && c6 > 0x3f {
                    n_real_next = 1;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c6) || c6 == b'e' {
                    n_real_next = 2;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c2b) {
                    n_real_next = if c6 > 0x3f { 2 } else { 0 };
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if c6 == b'd' || (c2b == b'd' && c6 > 0x3f) {
                    n_real_next = 10;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else {
                    n_real_next = 0;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                }
            }
        } else {
            n_dict_prev = 0;
            if is_special(c6) {
                n_real_next = 1;
                if is_special(local_2f_v) {
                    ivar7 = 1;
                } else {
                    l_813c3 = true;
                    ivar7 = 0;
                }
            } else {
                let c2b = seven[5];
                if is_special(c2b) && c6 > 0x3f {
                    n_real_next = 1;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c6) || c6 == b'e' {
                    n_real_next = 2;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if is_special2(c2b) {
                    n_real_next = if c6 > 0x3f { 2 } else { 0 };
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else if c6 == b'd' || (c2b == b'd' && c6 > 0x3f) {
                    n_real_next = 10;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                } else {
                    n_real_next = 0;
                    if is_special(local_2f_v) {
                        ivar7 = 1;
                    } else {
                        l_813c3 = true;
                        ivar7 = 0;
                    }
                }
            }
        }
    }
    if l_813c3 {
        let l2d = cand.phones[4];
        if is_special(l2d) && local_2f_v > 0x3f {
            ivar7 = 1;
        } else if mode % 10 == 0 && local_2f_v != b'e' {
            if is_special2(l2d) || l2d == b'e' {
                ivar7 = if local_2f_v > 0x3f { 2 } else { 0 };
            } else if local_2f_v == b'd' {
                ivar7 = 10;
            } else if l2d == b'd' {
                ivar7 = if local_2f_v > 0x3f { 10 } else { 0 };
            } else {
                ivar7 = 0;
            }
        } else {
            ivar7 = 2;
        }
    }
    state_mismatch_penalty(f4, n_real_next, ivar7, n_real_prev, n_dict_prev);
}

#[expect(
    clippy::unnested_or_patterns,
    reason = "C port: irreducible pair patterns"
)]
fn state_mismatch_penalty(
    f4: &mut f32,
    n_real_next: i32,
    ivar7: i32,
    n_real_prev: i32,
    n_dict_prev: i32,
) {
    if n_real_next != ivar7 {
        if n_real_prev != n_dict_prev {
            match (n_real_prev, n_dict_prev) {
                (1 | 2, 0) => *f4 += PENALTY_10,
                (1, 2) | (2, 1) | (0, 2) => *f4 += PHONE_MISMATCH,
                (0, 1) => *f4 += PENALTY_5,
                _ => {}
            }
        }
        let mut jumped = false;
        if n_real_next == 1 || n_real_next == 0 {
            if ivar7 == 2 {
                *f4 += PHONE_MISMATCH;
                jumped = true;
            }
        } else if n_real_next == 2 {
            if ivar7 == 0 {
                *f4 += PHONE_MISMATCH;
            } else if ivar7 == 1 {
                *f4 += PHONE_MISMATCH;
                jumped = true;
            }
        }
        if !jumped {
            *f4 += PENALTY_10;
        }
    }
}

#[expect(
    clippy::cast_possible_wrap,
    reason = "C port: index/math casts with wrap semantics"
)]
#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
fn extra_penalties(
    f4: &mut f32,
    one: &SelfInfo,
    cand: &PhoneDict,
    mode: i8,
    local_2e: u8,
    local_30: u8,
) {
    if one.f_dict_type >= 2 && one.f_dict_type <= 3 {
        if mode % 10 != 0 {
            *f4 += PENALTY_10;
        }
        if one.f_dict_type as i8 != cand.ch_dict_file_id + 1 {
            *f4 += PENALTY_10;
        }
    }
    if local_2e != local_30 {
        *f4 += PENALTY_10;
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn length_cost(one: &SelfInfo, result: PhoneResult, cand: &PhoneDict) -> (f32, f32, f32) {
    let unit_len = (i32::from(cand.w_pcm_size)
        - i32::from(cand.energy_pitch[3])
        - i32::from(MIN_PITCH)) as f32;
    let target = result.r_length_value;
    let unit_len_f = f64::from(unit_len);
    let target_f = f64::from(target);
    let f8;
    if one.f_dict_type >= 2 && one.f_dict_type <= 3 {
        if unit_len_f <= LEN_HIGH_RATIO * target_f {
            let d = (target - unit_len) / target;
            f8 = d * d;
        } else {
            f8 = PENALTY_10;
        }
    } else if f64::from(LEN_LOW_RATIO) * target_f <= unit_len_f
        && unit_len_f <= LEN_HIGH_RATIO * target_f
    {
        let d = (target - unit_len) / target;
        f8 = d * d;
    } else {
        f8 = PENALTY_10;
    }
    (f8, unit_len, target)
}

fn pitch_cost(ctx: &SynthCtx, result: PhoneResult, cand: &PhoneDict) -> f32 {
    let mut f9 = 0.0f32;
    if result.ch_start_pitch != 0 {
        f9 += pitch_distance(ctx, result.ch_start_pitch, cand.energy_pitch[2]);
    }
    if result.ch_end_pitch != 0 {
        f9 += pitch_distance(ctx, result.ch_end_pitch, cand.energy_pitch[3]);
    }
    f9 *= HALF;
    if result.ch_ave_pitch != 0 {
        f9 += pitch_distance(ctx, result.ch_ave_pitch, cand.energy_pitch[4]);
    }
    f9
}

#[must_use]
pub fn pitch_distance(ctx: &SynthCtx, b1: u8, b2: u8) -> f32 {
    if b2 <= b1 {
        get_pitch_table_value(ctx, b1, b2)
    } else {
        get_pitch_table_value(ctx, b2, b1)
    }
}

fn get_pitch_table_value(ctx: &SynthCtx, b1: u8, b2: u8) -> f32 {
    let mut r1 = 199usize;
    let mut r2 = 199usize;
    let s1 = ((u16::from(b1).wrapping_mul(0xcd)) >> 8) as u8 & 0xfc;
    if s1 < 199 {
        r1 = s1 as usize;
    }
    let s2 = ((u16::from(b2).wrapping_mul(0xcd)) >> 8) as u8 & 0xfc;
    if s2 < 199 {
        r2 = s2 as usize;
    }
    ctx.pitch_tbl.at(r1, r2).unwrap_or(0.0)
}

fn energy_distance(ctx: &SynthCtx, b1: u8, b2: u8) -> f32 {
    if b2 <= b1 {
        ctx.eng_tbl.at(b1 as usize, b2 as usize).unwrap_or(0.0)
    } else {
        ctx.eng_tbl.at(b2 as usize, b1 as usize).unwrap_or(0.0)
    }
}

#[expect(
    clippy::suboptimal_flops,
    reason = "C port: float-op order kept bit-exact (mul_add/FMA would change rounding)"
)]
fn euclidean_distance(a: &[f32; 12], b: &[f32; 12]) -> f32 {
    let mut s = 0.0f32;
    for i in 0..12 {
        let d = a[i] - b[i];
        s += d * d;
    }
    s.sqrt()
}

fn get_min_score(buf: &[f32]) -> usize {
    let mut best = 0usize;
    let mut v = buf[0];
    for (i, &x) in buf.iter().enumerate().skip(1) {
        if x < v {
            v = x;
            best = i;
        }
    }
    best
}

#[derive(Debug)]
pub struct SynthCtx<'a> {
    pub idx: &'a SynthIdx,
    pub groups: &'a SynthGroupIdx,
    pub pitch_tbl: &'a ktts_dict::synthdb::TriangularTable,
    pub eng_tbl: &'a ktts_dict::synthdb::TriangularTable,
}

const fn is_phrase_first_phone(letter: &Letter, one: &SelfInfo) -> bool {
    if !letter.is_phrase_head {
        return false;
    }
    let first_pos = if letter.f_cho >= 0 {
        0
    } else if letter.f_jung >= 0 {
        1
    } else {
        2
    };
    one.w_phone_pos == first_pos
}

fn best_hubo_search(
    ctx: &SynthCtx,
    phrase: &Phrase,
    hubo: &mut TriHubo,
    one: &SelfInfo,
    n_phone_pos: usize,
    n_total: usize,
    ones: &[SelfInfo],
) {
    let mut result = PhoneResult::default();
    let mut pcm_length = 0u16;
    let cur_letter = &phrase.letters[one.w_letter_pos as usize];
    if n_phone_pos == 0 || is_phrase_first_phone(cur_letter, one) {
        result.ch_start_pitch = 0;
    } else {
        let prev = &ones[n_phone_pos - 1];
        let prev_letter = &phrase.letters[prev.w_letter_pos as usize];
        let c = prev_letter.sch_cho[3];
        if c == 4 || prev.w_phone_pos != 0 || c == 8 || c == 7 || c == 13 {
            result.ch_start_pitch = get_pitch_and_length(
                prev_letter,
                prev.w_phone_pos,
                prev.f_index_pos,
                &mut result,
                &mut pcm_length,
                1,
            );
        } else if n_phone_pos == 1 || is_phrase_first_phone(prev_letter, prev) {
            result.ch_start_pitch = 0;
        } else {
            let pp = &ones[n_phone_pos - 2];
            let pp_letter = &phrase.letters[pp.w_letter_pos as usize];
            result.ch_start_pitch = get_pitch_and_length(
                pp_letter,
                pp.w_phone_pos,
                pp.f_index_pos,
                &mut result,
                &mut pcm_length,
                1,
            );
        }
    }
    if n_phone_pos != n_total - 1 {
        let next = &ones[n_phone_pos + 1];
        let next_letter = &phrase.letters[next.w_letter_pos as usize];
        if is_phrase_first_phone(next_letter, next) {
            result.ch_end_pitch = 0;
        } else {
            let c = next_letter.sch_cho[3];
            if c == 4 || next.w_phone_pos != 0 || c == 8 || c == 7 || c == 13 {
                result.ch_end_pitch = get_pitch_and_length(
                    next_letter,
                    next.w_phone_pos,
                    next.f_index_pos,
                    &mut result,
                    &mut pcm_length,
                    1,
                );
            } else if n_phone_pos != n_total - 2 {
                let nn = &ones[n_phone_pos + 2];
                let nn_letter = &phrase.letters[nn.w_letter_pos as usize];
                if is_phrase_first_phone(nn_letter, nn) {
                    result.ch_end_pitch = 0;
                } else {
                    result.ch_end_pitch = get_pitch_and_length(
                        nn_letter,
                        nn.w_phone_pos,
                        nn.f_index_pos,
                        &mut result,
                        &mut pcm_length,
                        1,
                    );
                }
            }
        }
    }
    let my = &ones[n_phone_pos];
    let my_letter = &phrase.letters[my.w_letter_pos as usize];
    result.ch_ave_pitch = get_pitch_and_length(
        my_letter,
        my.w_phone_pos,
        my.f_index_pos,
        &mut result,
        &mut pcm_length,
        1,
    );
    for e in &mut hubo.inno {
        let Some(rec) =
            unit_records(ctx.idx, e.w_tri_phone_no).and_then(|r| r.get(e.w_type_no as usize))
        else {
            e.score = f32::NEG_INFINITY;
            continue;
        };
        let s = get_inno_score(ctx, rec, one, result, my_letter);
        e.score = -s;
    }
    token_process(hubo);
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn phone_dict_find(
    ctx: &SynthCtx,
    phrase: &Phrase,
    hubos: &mut Vec<TriHubo>,
    ones: &mut Vec<SelfInfo>,
    exist_pos: &mut [u8],
) {
    for (li, letter) in phrase.letters.iter().enumerate() {
        let mut f_exist: u8 = exist_pos[li];
        for pos in 0u16..3 {
            let f_cmp: i8 = match pos {
                0 => letter.f_cho,
                1 => letter.f_jung,
                _ => letter.f_jong,
            };
            if f_cmp < 0 {
                continue;
            }
            let seven = seven_of(letter, pos);
            let mut hubo = TriHubo::new();
            hubo_phone_search(ctx, &mut hubo, seven);
            let dict_type = letter.dict_type;
            let f_chosong = chosong_type(seven[3] as u16, seven[2] as u16) as u8;
            ones.push(SelfInfo {
                f_exist_pos: f_exist,
                ch_word_pos: letter.word_idx as u8,
                w_letter_pos: li as u16,
                w_phone_pos: pos,
                f_index_pos: 0,
                f_chosong_type: f_chosong,
                f_dict_type: dict_type as u8,
            });
            f_exist = f_exist.wrapping_add(1);
            hubos.push(hubo);
        }
        exist_pos[li] = f_exist;
    }
}

#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
#[expect(
    clippy::suboptimal_flops,
    reason = "C port: float-op order kept bit-exact (mul_add/FMA would change rounding)"
)]
fn best_hubo_find(
    ctx: &SynthCtx,
    phrase: &Phrase,
    hubos: &mut [TriHubo],
    ones: &[SelfInfo],
) -> Vec<Vec<Option<BestPhone>>> {
    let n = hubos.len();
    let mut best: Vec<Vec<Option<BestPhone>>> = vec![Vec::new(); phrase.letter_num()];
    if n == 0 {
        return best;
    }
    let mut acc: Vec<Vec<f32>> = Vec::with_capacity(n);
    let mut back: Vec<Vec<usize>> = Vec::with_capacity(n);
    best_hubo_search(ctx, phrase, &mut hubos[0], &ones[0], 0, n, ones);
    acc.push(
        hubos[0]
            .sort
            .iter()
            .map(|&k| hubos[0].inno[k as usize].score)
            .collect(),
    );
    back.push(Vec::new());
    for n1 in 1..n {
        best_hubo_search(ctx, phrase, &mut hubos[n1], &ones[n1], n1, n, ones);
        let cur_n = hubos[n1].sort.len();
        let prev_n = hubos[n1 - 1].sort.len();
        let mut acc_row = vec![0f32; cur_n];
        let mut back_row = vec![0usize; cur_n];
        let cur_one = &ones[n1];
        let cur_letter = &phrase.letters[cur_one.w_letter_pos as usize];
        if is_phrase_first_phone(cur_letter, cur_one) {
            for (j, &cj) in hubos[n1].sort.iter().enumerate() {
                acc_row[j] = hubos[n1].inno[cj as usize].score;
            }
            if prev_n > 0 {
                let prev_best = get_min_score(&acc[n1 - 1]);
                back_row.fill(prev_best);
            }
        } else if prev_n > 0 && cur_n > 0 {
            for (j, &cj) in hubos[n1].sort.iter().enumerate() {
                let cur = &hubos[n1].inno[cj as usize];
                let cur_rec = unit_records(ctx.idx, cur.w_tri_phone_no)
                    .and_then(|r| r.get(cur.w_type_no as usize));
                let mut buf = vec![0f32; prev_n];
                let prev_hubo = &hubos[n1 - 1];
                let prev_one = &ones[n1 - 1];
                let cur_one = &ones[n1];
                for (i, &pi) in prev_hubo.sort.iter().enumerate() {
                    let prev = &prev_hubo.inno[pi as usize];
                    let prev_rec = unit_records(ctx.idx, prev.w_tri_phone_no)
                        .and_then(|r| r.get(prev.w_type_no as usize));
                    let (Some(pr), Some(cr)) = (prev_rec, cur_rec) else {
                        buf[i] = f32::INFINITY;
                        continue;
                    };
                    let low = cur_one.f_chosong_type == 0 && prev_one.f_chosong_type != 0;
                    let (eng_w, pitch_w) = if low {
                        (ENG_W_LOW, PITCH_W_LOW)
                    } else {
                        (ENG_W_HIGH, PITCH_W_HIGH)
                    };
                    let cep = euclidean_distance(&pr.sr_end_cepstrum, &cr.sr_start_cepstrum);
                    let prev_third = pr.phones[2];
                    let cur_third = cr.phones[2];
                    let prev_special =
                        prev_third == 4 || prev_third > 0x1f || prev_third == 7 || prev_third == 8;
                    let cur_unvoiced =
                        cur_third != 4 && cur_third < 0x20 && cur_third != 7 && cur_third != 8;
                    let pitch_dist = if prev_special && cur_unvoiced {
                        let f21 = pitch_distance(ctx, pr.energy_pitch[3], cr.energy_pitch[3]);
                        let f22 = pitch_distance(ctx, pr.energy_pitch[4], cr.energy_pitch[2]);
                        f21 * PITCH_W_END + f22 * PITCH_W_START
                    } else if prev_special {
                        let f21 = pitch_distance(ctx, pr.energy_pitch[3], cr.energy_pitch[4]);
                        let f22 = pitch_distance(ctx, pr.energy_pitch[4], cr.energy_pitch[2]);
                        f21 * PITCH_W_END + f22 * PITCH_W_START
                    } else {
                        let f21 = pitch_distance(ctx, pr.energy_pitch[3], cr.energy_pitch[4]);
                        let f22 = pitch_distance(ctx, pr.energy_pitch[2], cr.energy_pitch[2]);
                        f21 * PITCH_W_END + f22 * PITCH_W_START
                    };
                    let mut eng = energy_distance(ctx, pr.energy_pitch[1], cr.energy_pitch[0]);
                    if eng > ENG_SAT {
                        eng = SCORE_LIMIT;
                    }
                    let prev_acc = acc[n1 - 1][i];
                    let f = 2.0
                        * ((cep / CEP_DIV) * PANALTY[7]
                            + eng_w * PANALTY[5] * eng
                            + pitch_w * PANALTY[6] * pitch_dist)
                        + cur.score
                        + prev_acc;
                    buf[i] = f;
                }
                let b = get_min_score(&buf);
                back_row[j] = b;
                acc_row[j] = buf[b];
            }
        }
        acc.push(acc_row);
        back.push(back_row);
    }
    let mut last = n - 1;
    while last > 0 && acc[last].is_empty() {
        last -= 1;
    }
    if acc[last].is_empty() {
        return best;
    }
    let mut b = get_min_score(&acc[last]);
    let mut path = vec![usize::MAX; n];
    path[last] = b;
    for k in (0..last).rev() {
        if back[k + 1].is_empty() {
            b = usize::MAX;
        } else {
            b = back[k + 1][b.min(back[k + 1].len() - 1)];
        }
        path[k] = b;
    }
    for (n_i, &pi) in path.iter().enumerate() {
        if pi == usize::MAX || hubos[n_i].sort.is_empty() {
            continue;
        }
        let one = &ones[n_i];
        let entry = &hubos[n_i].inno[hubos[n_i].sort[pi.min(hubos[n_i].sort.len() - 1)] as usize];
        best[one.w_letter_pos as usize].resize(one.f_exist_pos as usize + 1, None);
        best[one.w_letter_pos as usize][one.f_exist_pos as usize] = Some(BestPhone {
            unit_no: entry.w_tri_phone_no,
            type_no: entry.w_type_no,
            f_type_index: entry.f_type_index,
        });
    }
    best
}

#[must_use]
pub fn select_units(ctx: &SynthCtx, phrase: &Phrase) -> Selection {
    let letter_num = phrase.letter_num();
    let mut hubos: Vec<TriHubo> = Vec::with_capacity(letter_num * 3);
    let mut ones: Vec<SelfInfo> = Vec::with_capacity(letter_num * 3);
    let mut exist_pos = vec![0u8; letter_num];
    phone_dict_find(ctx, phrase, &mut hubos, &mut ones, &mut exist_pos);
    let best = best_hubo_find(ctx, phrase, &mut hubos, &ones);

    Selection { best }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chosong_type_cases() {
        assert_eq!(chosong_type(0, 0), 0);
        assert_eq!(chosong_type(2, 0x20), 2);
        assert_eq!(chosong_type(2, 0x60), 1);
        assert_eq!(chosong_type(0x14, 0x60), 1);
        assert_eq!(chosong_type(0x14, 0x20), 0);
        assert_eq!(chosong_type(3, 0), 2);
        assert_eq!(chosong_type(0x0b, 0), 1);
        assert_eq!(chosong_type(0x15, 0), 0);
    }

    #[test]
    fn pitch_table_scale() {
        let s = |b: u8| ((u16::from(b).wrapping_mul(0xcd)) >> 8) as u8 & 0xfc;
        assert_eq!(s(0), 0);
        assert_eq!(s(255), 204);
        assert_eq!(s(200), 160);
        assert!(s(255) >= 199);
        assert!(s(249) < 199);
    }
}
