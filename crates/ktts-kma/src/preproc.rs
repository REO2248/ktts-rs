use crate::code;
use crate::dict::KmaDicts;
use crate::digits;
use crate::ma::{IregulerStr, KlpState, MaMorph, MaWord};
use crate::tables;

fn word_pyogi(w: &MaWord) -> Vec<u8> {
    let mut out = Vec::new();
    for m in &w.morphs {
        out.extend_from_slice(&m.pyogi);
    }
    out
}

fn word_pyogi_u16(w: &MaWord) -> Vec<u16> {
    word_pyogi(w).iter().map(|&b| u16::from(b)).collect()
}

fn morph_pyogi_u16(m: &MaMorph) -> Vec<u16> {
    m.pyogi.iter().map(|&b| u16::from(b)).collect()
}

fn wcscmp_c(w: &MaWord, s: &str) -> bool {
    word_pyogi(w) == s.as_bytes()
}

fn is_pumsa_array(words: &[MaWord], tag: &[u8]) -> bool {
    if words.len() < tag.len() {
        return false;
    }
    for (i, &t) in tag.iter().enumerate() {
        if t == b'*' {
            if words[i].b_str_type != 1 {
                return false;
            }
        } else if words[i].morphs.first().map(|m| m.ch_tag) != Some(t) {
            return false;
        }
    }
    true
}

fn is_symbol_str(w: &MaWord, ch: u8) -> bool {
    w.source.len() == 1 && w.source[0] == u16::from(ch)
}

fn is_digit_str(w: &MaWord) -> Option<i32> {
    if w.b_str_type != 0 {
        return None;
    }
    Some(digits::wtoi(&w.source))
}

fn is_length_str(w: &MaWord, n_cmp: usize) -> Option<i32> {
    if w.source.len() != n_cmp {
        return None;
    }
    is_digit_str(w)
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
const fn is_year(n: i32) -> bool {
    (n as u32).wrapping_sub(0x709) < 0x4b0
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
const fn is_month(n: i32) -> bool {
    (n as u32).wrapping_sub(1) < 0xc
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
const fn is_day(n: i32) -> bool {
    (n as u32).wrapping_sub(1) < 0x1f
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
const fn is_hour(n: i32) -> bool {
    (n as u32) < 0x19
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
const fn is_minute_sec(n: i32) -> bool {
    (n as u32) < 0x3d
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
const fn is_link_day(_n_year: i32, _n_month: i32, n_day: i32) -> bool {
    (n_day as u32).wrapping_sub(1) < 0x1f
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
const fn is_link_month(_n_month: i32, n_day: i32) -> bool {
    (n_day as u32).wrapping_sub(1) < 0x1f
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
const fn is_juche_year(n: i32) -> bool {
    (n as u32).wrapping_sub(1) < 200
}

fn is_year_str(w: &MaWord) -> Option<i32> {
    let n = is_digit_str(w)?;
    is_year(n).then_some(n)
}

fn is_month_str(w: &MaWord) -> Option<i32> {
    let n = is_digit_str(w)?;
    is_month(n).then_some(n)
}

fn is_day_str(w: &MaWord) -> Option<i32> {
    let n = is_digit_str(w)?;
    is_day(n).then_some(n)
}

fn is_hour_str(w: &MaWord) -> Option<i32> {
    let n = is_digit_str(w)?;
    is_hour(n).then_some(n)
}

fn is_minute_sec_str(w: &MaWord) -> Option<i32> {
    let n = is_digit_str(w)?;
    is_minute_sec(n).then_some(n)
}

fn is_juche_year_str(w: &MaWord) -> Option<i32> {
    let n = is_digit_str(w)?;
    is_juche_year(n).then_some(n)
}

fn is_digit_value(w: &MaWord) -> Option<i32> {
    let n = is_digit_str(w)?;
    (n < 100).then_some(n)
}

fn is_year_date(w: &MaWord) -> Option<(i32, i32, i32)> {
    if w.morphs.len() != 5 {
        return None;
    }
    let year = digits::wtoi(&morph_pyogi_u16(&w.morphs[0]));
    let month = digits::wtoi(&morph_pyogi_u16(&w.morphs[2]));
    let day = digits::wtoi(&morph_pyogi_u16(&w.morphs[4]));
    if !is_year(year) || !is_month(month) || !is_day(day) || !is_link_day(year, month, day) {
        return None;
    }
    Some((year, month, day))
}

fn is_month_date(w: &MaWord) -> Option<(i32, i32)> {
    if w.morphs.len() != 3 {
        return None;
    }
    let month = digits::wtoi(&morph_pyogi_u16(&w.morphs[0]));
    let day = digits::wtoi(&morph_pyogi_u16(&w.morphs[2]));
    if !is_month(month) || !is_day(day) || !is_link_month(month, day) {
        return None;
    }
    Some((month, day))
}

fn is_ym_date(w: &MaWord) -> Option<(i32, i32)> {
    if w.morphs.len() != 3 {
        return None;
    }
    let year = digits::wtoi(&morph_pyogi_u16(&w.morphs[0]));
    let month = digits::wtoi(&morph_pyogi_u16(&w.morphs[2]));
    if !is_year(year) || !is_month(month) {
        return None;
    }
    Some((year, month))
}

fn is_juche_date(w: &MaWord) -> Option<(i32, i32, i32)> {
    if w.morphs.len() != 5 {
        return None;
    }
    let year = digits::wtoi(&morph_pyogi_u16(&w.morphs[0]));
    let month = digits::wtoi(&morph_pyogi_u16(&w.morphs[2]));
    let day = digits::wtoi(&morph_pyogi_u16(&w.morphs[4]));
    if !is_juche_year(year)
        || !is_month(month)
        || !is_day(day)
        || !is_link_day(year + 0x777, month, day)
    {
        return None;
    }
    Some((year, month, day))
}

fn is_16_only_digit(pw: &[u16]) -> bool {
    !pw.is_empty()
        && pw.iter().all(|&c| {
            let d = c.wrapping_sub(0x61);
            let u = c.wrapping_sub(0x41);
            d <= 5 || u <= 5
        })
}

fn is_16_digit(d: &KmaDicts, w: &MaWord) -> bool {
    if w.morphs.len() != 1 {
        return false;
    }
    let tag = w.morphs[0].ch_tag;
    if tag == b'J' {
        is_16_only_digit(&w.source)
    } else if tag == b'I' {
        is_digit_str(w).is_some()
    } else {
        let _ = d;
        false
    }
}

fn is_pre_type_code(d: &KmaDicts, w: &MaWord, ch_type: u8) -> bool {
    let key: Vec<u16> = w
        .morphs
        .first()
        .map(|m| m.pyogi.iter().map(|&b| u16::from(b)).collect())
        .unwrap_or_default();
    d.prepron.get(&key) == Some(&ch_type)
}

fn is_pre_type_code_str(d: &KmaDicts, key: &[u16], ch_type: u8) -> bool {
    d.prepron.get(key) == Some(&ch_type)
}

fn get_symbol_type_code(d: &KmaDicts, w_code: u16, w_type: u16) -> Option<Vec<u8>> {
    d.unipron.get(&vec![w_type + 0x30, w_code]).cloned()
}

fn get_str_type_code(d: &KmaDicts, pw_code: &[u16], w_type: u16) -> Option<Vec<u8>> {
    let mut key = vec![w_type + 0x30];
    key.extend_from_slice(pw_code);
    d.strpron.get(&key).cloned()
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
fn is_remove_symbol(pw: &[u16]) -> bool {
    let s: Vec<u8> = pw.iter().map(|&c| c as u8).collect();
    s.starts_with(b"**")
        || s.starts_with(b"//")
        || s.starts_with(b"/*")
        || s.starts_with(b"*/")
        || s == b"-"
        || s == b":"
}

const fn is_sub_split(w: &MaWord) -> bool {
    w.b_str_type == 2
}

fn morph_cvc_from_pyogi(pyogi: &[u8]) -> Vec<u8> {
    let c = code::conv_pyogi_to_cvc(pyogi);
    if c.is_empty() { pyogi.to_vec() } else { c }
}

fn insert_anal(klp: &mut KlpState, n_index: usize, pch_str: &str, ch_pumsa: u8) {
    let pyogi = pch_str.as_bytes().to_vec();
    let cvc = morph_cvc_from_pyogi(&pyogi);
    klp.words.insert(
        n_index,
        MaWord {
            source: pch_str.encode_utf16().collect(),
            morphs: vec![MaMorph {
                ch_tag: ch_pumsa,
                pyogi,
                cvc,
                prob: 0.0,
                b_merged: false,
            }],
            b_str_type: 3,
            ireguler: Vec::new(),
            b_sentence_end: false,
        },
    );
}

fn remove_anal(klp: &mut KlpState, n_index: usize, n_len: usize) {
    let end = (n_index + n_len).min(klp.words.len());
    if n_index < end {
        klp.words.drain(n_index..end);
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn insert_digit_pron(klp: &mut KlpState, pn_index: &mut isize, pch_digit: &str) {
    let items: Vec<&str> = pch_digit.split('\t').filter(|s| !s.is_empty()).collect();
    if items.is_empty() {
        return;
    }
    for (n, item) in items.iter().enumerate() {
        let morph_pyogis: Vec<&str> = item.split(' ').filter(|s| !s.is_empty()).collect();
        let word_str: String = morph_pyogis.concat();
        let word = MaWord {
            source: word_str.encode_utf16().collect(),
            morphs: morph_pyogis
                .iter()
                .map(|p| MaMorph {
                    ch_tag: b'H',
                    pyogi: p.as_bytes().to_vec(),
                    cvc: code::conv_pyogi_to_cvc(p.as_bytes()),
                    prob: 0.0,
                    b_merged: false,
                })
                .collect(),
            b_str_type: 3,
            ireguler: Vec::new(),
            b_sentence_end: false,
        };
        klp.words.insert(*pn_index as usize, word);
        *pn_index += 1;
        if n < items.len() - 1 {
            insert_anal(klp, *pn_index as usize, ",", b'M');
            *pn_index += 1;
        }
    }
}

fn convert_anal(word: &mut MaWord, pch_str: &str, ch_pumsa: u8) {
    let pyogi = pch_str.as_bytes().to_vec();
    let cvc = morph_cvc_from_pyogi(&pyogi);
    if let Some(m) = word.morphs.first_mut() {
        m.ch_tag = ch_pumsa;
        m.pyogi = pyogi;
        m.cvc = cvc;
    }
    word.source = pch_str.encode_utf16().collect();
    word.b_str_type = 3;
}

fn process_pyogi_to_wansong(klp: &mut KlpState) {
    let mut n_count = 0usize;
    let mut i = 0usize;
    while i < klp.words.len() {
        let bound = klp.words[i]
            .morphs
            .first()
            .is_some_and(|m| tables::is_bound_pumsa(m.ch_tag));
        if bound {
            if n_count != i {
                klp.words[n_count] = klp.words[i].clone();
            }
            n_count += 1;
            i += 1;
            continue;
        }
        if klp.words[i].b_str_type != 0 {
            let src = word_pyogi(&klp.words[i]);
            let cvc = code::conv_pyogi_to_cvc(&src);
            let wan = code::conv_cvc_to_uni_wan(&cvc);
            if !wan.is_empty() {
                klp.words[i].source = wan;
            }
            if n_count != i {
                klp.words[n_count] = klp.words[i].clone();
            }
            n_count += 1;
        } else if klp.words[i]
            .morphs
            .first()
            .is_some_and(|m| m.ch_tag == b'J')
        {
            if n_count != i {
                klp.words[n_count] = klp.words[i].clone();
            }
            n_count += 1;
        }
        i += 1;
    }
    klp.words.truncate(n_count);
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_engilish(klp: &mut KlpState, d: &KmaDicts, pn_index: &mut isize) -> bool {
    let idx = *pn_index as usize;
    if idx >= klp.words.len() {
        return false;
    }
    let w0 = klp.words[idx].clone();
    let tag = w0.morphs.first().map_or(0, |m| m.ch_tag);
    if tag != b'J' || w0.b_str_type == 1 {
        return false;
    }
    let word_bytes: Vec<u8> = w0.source.iter().map(|&c| c as u8).collect();
    remove_anal(klp, idx, 1);
    if let Some(p) = crate::english::english_word_to_pyogi(&d.eng, &word_bytes) {
        let parts: Vec<&str> = p.split(' ').filter(|s| !s.is_empty()).collect();
        if parts.len() == 1 {
            insert_anal(klp, *pn_index as usize, parts[0], b'J');
        } else {
            for part in &parts {
                insert_anal(klp, *pn_index as usize, part, b'J');
                *pn_index += 1;
                insert_anal(klp, *pn_index as usize, " ", b'k');
                *pn_index += 1;
            }
            *pn_index -= 1;
        }
        return true;
    }
    true
}

fn process_del_space_symbol(klp: &mut KlpState) {
    if klp.words.is_empty() {
        return;
    }
    let mut i5 = 0usize;
    let mut i6 = 1usize;
    loop {
        if i5 + 2 >= klp.words.len() {
            i5 += 1;
            i6 += 1;
            if i6 >= klp.words.len() {
                return;
            }
            continue;
        }
        if i5 + 3 < klp.words.len() && is_pumsa_array(&klp.words[i5..], b"ILkI") {
            if klp.words[i5 + 2].source.len() == 1 {
                klp.words.remove(i5 + 2);
            }
            i5 += 1;
            i6 += 1;
            if i6 >= klp.words.len() {
                return;
            }
            continue;
        }
        if is_pumsa_array(&klp.words[i5..], b"IkI") && klp.words[i6].source.len() == 1 {
            convert_anal(&mut klp.words[i6], ",", b'M');
            klp.words[i6].b_str_type = 0;
        }
        i5 += 1;
        i6 += 1;
        if i6 >= klp.words.len() {
            return;
        }
    }
}

fn digit_merge_process(klp: &mut KlpState, i: usize, n_len: usize) -> bool {
    let mut n_word_len = 0usize;
    for w in &klp.words[i..i + n_len] {
        n_word_len += w.source.len();
    }
    if n_word_len > 99 {
        return false;
    }
    let mut source: Vec<u16> = Vec::new();
    let mut morphs: Vec<MaMorph> = Vec::new();
    let mut ireguler: Vec<IregulerStr> = Vec::new();
    for w in &klp.words[i..i + n_len] {
        source.extend_from_slice(&w.source);
        morphs.extend(w.morphs.iter().cloned());
        ireguler.extend(w.ireguler.iter().cloned());
    }
    klp.words[i] = MaWord {
        source,
        morphs,
        b_str_type: 0,
        ireguler,
        b_sentence_end: false,
    };
    true
}

fn process_digit_merge(klp: &mut KlpState) {
    if klp.words.len() < 3 {
        return;
    }
    let mut i = 0usize;
    loop {
        while is_pumsa_array(&klp.words[i..], b"ILI") && digit_merge_process(klp, i, 3) {
            remove_anal(klp, i + 1, 2);
            klp.words[i].b_str_type = 2;
            if klp.words.len() <= i + 2 {
                return;
            }
        }
        i += 1;
        if klp.words.len() <= i + 2 {
            return;
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
fn digit_sum_process(klp: &mut KlpState, i5: usize, n_len: usize) {
    let mut sw: Vec<u16> = Vec::new();
    let mut j = 0usize;
    while j < n_len {
        sw.extend_from_slice(&klp.words[i5 + j].source);
        j += 2;
    }
    klp.words[i5].source.clone_from(&sw);
    if let Some(m) = klp.words[i5].morphs.first_mut() {
        m.pyogi = sw.iter().map(|&c| c as u8).collect();
        m.cvc = Vec::new();
    }
}

fn process_digit_seperate(klp: &mut KlpState) {
    if klp.words.len() < 3 {
        return;
    }
    let mut i5 = 0usize;
    loop {
        while klp.words[i5].b_str_type != 0 || klp.words[i5].source.len() > 3 {
            i5 += 1;
            if klp.words.len() <= i5 + 2 {
                return;
            }
        }
        if !is_pumsa_array(&klp.words[i5..], b"IMI")
            || (i5 > 0 && klp.words[i5 - 1].morphs.first().map(|m| m.ch_tag) == Some(b'M'))
        {
            i5 += 1;
            if klp.words.len() <= i5 + 2 {
                return;
            }
            continue;
        }
        let pps_anal_i5 = i5;
        let n_index = i5 + 1;
        let mut i4 = 0usize;
        if n_index < klp.words.len() - 1 {
            let mut n_count = 0usize;
            i4 = n_index;
            loop {
                if !is_pumsa_array(&klp.words[i4..], b"MI")
                    || klp.words[i4].b_str_type != 0
                    || klp.words[i4 + 1].source.len() != 3
                {
                    break;
                }
                i4 += 2;
                n_count += 1;
                if i4 >= klp.words.len() - 1 {
                    break;
                }
            }
            if n_count > 8 || n_count == 0 {
                i4 = n_count * 2;
                i5 = n_index + i4;
                if klp.words.len() <= i5 + 2 {
                    return;
                }
                continue;
            }
            let merge = klp.words.len() <= i4 || {
                i5 = n_index + n_count * 2;
                klp.words[i4].morphs.first().map(|m| m.ch_tag) != Some(b'M')
            };
            if merge {
                digit_sum_process(klp, pps_anal_i5, n_count * 2 + 1);
                remove_anal(klp, n_index, n_count * 2);
                i5 = n_index;
            }
            if klp.words.len() <= i5 + 2 {
                return;
            }
            continue;
        }
        i5 = n_index + i4;
        if klp.words.len() <= i5 + 2 {
            return;
        }
    }
}

fn process_del_sp_symbol(klp: &mut KlpState) {
    if klp.words.is_empty() {
        return;
    }
    let mut i2 = klp.words.len();
    let mut n_index = 0usize;
    let mut n_index_00 = 1usize;
    loop {
        while i2 <= n_index + 1 {
            n_index += 1;
            let b = i2 <= n_index_00;
            n_index_00 += 1;
            if b {
                return;
            }
        }
        if is_pumsa_array(&klp.words[n_index..], b"kM") {
            remove_anal(klp, n_index, 1);
            i2 = klp.words.len();
            let b = i2 <= n_index_00;
            n_index += 1;
            n_index_00 += 1;
            if b {
                return;
            }
            continue;
        }
        if is_pumsa_array(&klp.words[n_index..], b"Mk") {
            remove_anal(klp, n_index_00, 1);
            i2 = klp.words.len();
            n_index += 1;
            let b = i2 <= n_index_00;
            n_index_00 += 1;
            if b {
                return;
            }
            continue;
        }
        if is_pumsa_array(&klp.words[n_index..], b"kk") {
            remove_anal(klp, n_index, 1);
        }
        i2 = klp.words.len();
        n_index += 1;
        let b = i2 <= n_index_00;
        n_index_00 += 1;
        if b {
            return;
        }
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn is_juche_nyeon(klp: &KlpState, n_index: isize) -> bool {
    if n_index < 0 || (n_index as usize) + 5 >= klp.words.len() {
        return false;
    }
    let i = n_index as usize;
    if !is_pumsa_array(&klp.words[i..], b"*INIO*") {
        return false;
    }
    if !wcscmp_c(&klp.words[i], "juc9") || !wcscmp_c(&klp.words[i + 5], "nyeN") {
        return false;
    }
    let Some(j) = is_juche_year_str(&klp.words[i + 1]) else {
        return false;
    };
    let Some(y) = is_year_str(&klp.words[i + 3]) else {
        return false;
    };
    j + 0x777 == y
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn is_year_parentheses(klp: &KlpState, n_index: isize) -> bool {
    if n_index < 0 || (n_index as usize) + 2 >= klp.words.len() {
        return false;
    }
    let i = n_index as usize;
    is_pumsa_array(&klp.words[i..], b"NIO") && is_year_str(&klp.words[i + 1]).is_some()
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn is_double_parentheses(klp: &KlpState, n_index: isize) -> bool {
    if n_index < 0 || (n_index as usize) + 2 >= klp.words.len() {
        return false;
    }
    let i = n_index as usize;
    is_pumsa_array(&klp.words[i..], b"NIO") && is_digit_value(&klp.words[i]).is_some()
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn is_parentheses(klp: &KlpState, n_index: isize) -> bool {
    if n_index < 0 || (n_index as usize) + 1 >= klp.words.len() {
        return false;
    }
    let i = n_index as usize;
    is_pumsa_array(&klp.words[i..], b"IO") && is_digit_value(&klp.words[i]).is_some()
}

#[expect(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
fn process_ma_repeat(klp: &mut KlpState) {
    if klp.words.is_empty() {
        return;
    }
    let mut local_20 = 0usize;
    'outer: loop {
        let mut n_index_01 = local_20;
        let mut found_o = false;
        while n_index_01 < klp.words.len() {
            if klp.words[n_index_01].morphs.first().map(|m| m.ch_tag) == Some(b'O') {
                found_o = true;
                break;
            }
            n_index_01 += 1;
        }
        if !found_o {
            local_20 = n_index_01 + 1;
            if klp.words.len() <= local_20 {
                return;
            }
            continue 'outer;
        }
        let n_index = n_index_01 as isize - 1;
        if n_index < local_20 as isize {
            if is_parentheses(klp, n_index) {
                convert_anal(&mut klp.words[n_index_01], ",", b'M');
                insert_anal(klp, n_index as usize, "baNgwaLho", b'0');
                local_20 = n_index_01 + 1;
                if klp.words.len() <= local_20 {
                    return;
                }
                continue 'outer;
            }
            remove_anal(klp, n_index_01, 1);
            local_20 = n_index_01;
            if klp.words.len() <= local_20 {
                return;
            }
            continue 'outer;
        }
        let mut n_index_00 = n_index;
        let mut p_svar4 = n_index_01 - 1;
        let mut found_n = false;
        loop {
            if klp.words[p_svar4].morphs.first().map(|m| m.ch_tag) == Some(b'N') {
                found_n = true;
                break;
            }
            n_index_00 -= 1;
            if n_index_00 < local_20 as isize {
                break;
            }
            p_svar4 -= 1;
        }
        if !found_n {
            if is_parentheses(klp, n_index) {
                convert_anal(&mut klp.words[n_index_01], ",", b'M');
                insert_anal(klp, n_index as usize, "baNgwaLho", b'0');
                local_20 = n_index_01 + 1;
                if klp.words.len() <= local_20 {
                    return;
                }
                continue 'outer;
            }
            remove_anal(klp, n_index_01, 1);
            local_20 = n_index_01;
            if klp.words.len() <= local_20 {
                return;
            }
            continue 'outer;
        }
        let close_is_paren = wcscmp_c(&klp.words[n_index_01], ")");
        let open_is_paren = wcscmp_c(&klp.words[p_svar4], "(");
        if close_is_paren && open_is_paren {
            if is_juche_nyeon(klp, n_index_00 - 2) {
                remove_anal(klp, n_index_01, 1);
                convert_anal(&mut klp.words[n_index_00 as usize], ",", b'M');
                local_20 = n_index_01 + 1;
                if klp.words.len() <= local_20 {
                    return;
                }
                continue 'outer;
            }
            if n_index_01 as isize - n_index_00 == 2 {
                if is_year_parentheses(klp, n_index_00) {
                    remove_anal(klp, n_index_01, 1);
                    convert_anal(&mut klp.words[n_index_00 as usize], " ", b'k');
                    local_20 = n_index_01 + 1;
                    if klp.words.len() <= local_20 {
                        return;
                    }
                    continue 'outer;
                }
                if is_double_parentheses(klp, n_index_00) {
                    convert_anal(&mut klp.words[n_index_00 as usize], "va*gwaLho", b'0');
                    convert_anal(&mut klp.words[n_index_01], ",", b'M');
                    local_20 = n_index_01 + 1;
                    if klp.words.len() <= local_20 {
                        return;
                    }
                    continue 'outer;
                }
            }
            if n_index_00 > 0 && n_index_01 + 1 < klp.words.len() {
                if klp.words[n_index_01 + 1].morphs.first().map(|m| m.ch_tag) == Some(b'k') {
                    remove_anal(klp, n_index_01, 1);
                    convert_anal(&mut klp.words[n_index_00 as usize], ",", b'M');
                    local_20 = n_index_01 + 1;
                    if klp.words.len() <= local_20 {
                        return;
                    }
                    continue 'outer;
                }
                remove_anal(
                    klp,
                    n_index_00 as usize,
                    (n_index_01 - n_index_00 as usize) + 1,
                );
                local_20 = n_index_00 as usize;
                if klp.words.len() <= local_20 {
                    return;
                }
                continue 'outer;
            }
        }
        convert_anal(&mut klp.words[n_index_00 as usize], " ", b'k');
        remove_anal(klp, n_index_01, 1);
        local_20 = n_index_01;
        if klp.words.len() <= local_20 {
            return;
        }
    }
}

fn process_modify_ma_result_to(klp: &mut KlpState) {
    for w in &mut klp.words {
        for m in &mut w.morphs {
            if m.ch_tag == b':' {
                m.ch_tag = b'9';
            }
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn conv_uni_wan_to_cvc_special_pre(w: u16) -> [u8; 3] {
    if w.wrapping_add(0xcecf) < 0x33 {
        let idx = (w as usize).wrapping_sub(0x3131);
        if idx < tables::KS_JOHAB.len() {
            let j = tables::KS_JOHAB[idx];
            return [
                ((j >> 10) & 0x1f) as u8,
                ((j >> 5) & 0x1f) as u8,
                (j & 0x1f) as u8,
            ];
        }
        return [0, 0, 0];
    }
    let t = i32::from(w) - 0xac00;
    if !(0..=0x2bff).contains(&t) {
        return [0, 0, 0];
    }
    let r = t % 0x24c;
    [
        (t / 0x24c + 2) as u8,
        tables::UNI_JUNG_ID[(r / 0x1c) as usize],
        tables::UNI_JONG_ID[(r % 0x1c) as usize],
    ]
}

fn set_word_cvc_table(pw_str: &[u16]) -> Vec<i16> {
    let n = pw_str.len();
    let mut table = vec![0i16; n * 3 + 2];
    let mut acc: i16 = 0;
    for (i, &w) in pw_str.iter().enumerate() {
        let cvc = conv_uni_wan_to_cvc_special_pre(w);
        let cho = tables::CHO_POS_TBL[cvc[0] as usize];
        table[i * 3] = acc + cho;
        acc += cho + tables::JUNG_POS_TBL[cvc[1] as usize];
        table[i * 3 + 1] = acc;
        acc += tables::JONG_POS_TBL[cvc[2] as usize];
        table[i * 3 + 2] = acc;
    }
    table[n * 3] = 0;
    table[n * 3 + 1] = 0;
    table
}

fn process_last_cut(klp: &mut KlpState) {
    let mut n = 0usize;
    while n < klp.words.len() {
        let w = &klp.words[n];
        if w.morphs.len() > 10 || w.source.len() > 0x13 {
            process_last_cut_one(klp, n);
        }
        n += 1;
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_last_cut_one(klp: &mut KlpState, n: usize) {
    let w = &klp.words[n];
    let table = set_word_cvc_table(&w.source);
    let i_var5 = w.source.len() * 3;
    let mut i_var7 = 0usize;
    let mut local_32: i32 = 0;
    let w_morph_cnt = w.morphs.len();
    let mut n1 = 0usize;
    while n1 < w_morph_cnt {
        let ascii_len = klp.words[n].morphs[n1].pyogi.len() as i32;
        let s_var3 = local_32 + ascii_len * 6;
        let mut s_var1 = i32::from(table[i_var7]);
        if s_var1 <= s_var3 {
            while i_var7 < i_var5 {
                if (n1 > 9 && i_var7.is_multiple_of(3)) || i_var7 > 0x3b {
                    let n_len = i_var7 / 3;
                    let n_morph_start = (s_var1 - local_32) / 6 - 1;
                    two_cut_anal(klp, n, n1, n_len, n_morph_start);
                    return;
                }
                s_var1 = i32::from(table[i_var7 + 1]);
                i_var7 += 1;
                if s_var3 < s_var1 {
                    break;
                }
            }
        }
        n1 += 1;
        local_32 = s_var3;
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn two_cut_anal(
    klp: &mut KlpState,
    n_index: usize,
    n_start: usize,
    n_len: usize,
    n_morph_start: i32,
) {
    let orig = klp.words[n_index].clone();
    let mut first = orig.clone();
    first.source.truncate(n_len);
    first.morphs.truncate(n_start);
    let mut second = MaWord {
        source: orig.source[n_len..].to_vec(),
        morphs: orig.morphs[n_start..].to_vec(),
        b_str_type: 3,
        ireguler: Vec::new(),
        b_sentence_end: false,
    };
    if n_morph_start != 0 {
        let prefix_len = n_morph_start as usize;
        let split = &orig.morphs[n_start];
        let mut prefix = split.clone();
        prefix.pyogi.truncate(prefix_len);
        prefix.cvc = code::conv_pyogi_to_cvc(&prefix.pyogi);
        first.morphs.push(prefix);
        let mut suffix = MaMorph {
            ch_tag: split.ch_tag,
            pyogi: split.pyogi[prefix_len..].to_vec(),
            cvc: Vec::new(),
            prob: split.prob,
            b_merged: false,
        };
        suffix.cvc = code::conv_pyogi_to_cvc(&suffix.pyogi);
        second.morphs[0] = suffix;
    }
    klp.words[n_index] = first;
    klp.words.insert(n_index + 1, second);
}

fn process_last_merge(klp: &mut KlpState) {
    let mut i = 0usize;
    let mut n_start = 0usize;
    loop {
        while i < klp.words.len()
            && klp.words[i]
                .morphs
                .first()
                .is_some_and(|m| tables::is_bound_pumsa(m.ch_tag))
        {
            i = merge_process(klp, n_start, i) + 1;
            n_start = i;
            if klp.words.len() <= i {
                break;
            }
        }
        if klp.words.len() <= i {
            break;
        }
        i += 1;
    }
    if !klp.words.is_empty() {
        merge_process(klp, n_start, klp.words.len() - 1);
    }
    let mut out: Vec<MaWord> = Vec::new();
    for w in klp.words.drain(..) {
        if !w
            .morphs
            .first()
            .is_some_and(|m| tables::is_bound_pumsa(m.ch_tag))
        {
            out.push(w);
        }
    }
    klp.words = out;
}

fn merge_process(klp: &mut KlpState, n_start: usize, n_end: usize) -> usize {
    if n_end <= n_start {
        return n_end;
    }
    let mut n1 = n_start;
    let mut end = n_end;
    let mut n_ret = 0usize;
    if klp.words[end].morphs.first().map(|m| m.ch_tag) == Some(b'k') {
        end -= 1;
        n_ret = 1;
        if end < n_start {
            let ret = merge_process_range(klp, n1, n_start);
            return (end + n_ret) - ret;
        }
    }
    loop {
        let mut n_morph = 0usize;
        let mut n_len = 0usize;
        let mut start = n1;
        loop {
            let w = &klp.words[start];
            n_morph += w.morphs.len();
            n_len += w.source.len();
            if n_len > 0x13 || n_morph > 10 {
                break;
            }
            start += 1;
            if end < start {
                let ret = merge_process_range(klp, n1, start);
                return (end + n_ret) - ret;
            }
        }
        let ret = merge_process_range(klp, n1, start);
        let new_start = n1 + 1;
        end -= ret;
        n1 = new_start;
        if end < new_start {
            let ret2 = merge_process_range(klp, n1, new_start);
            return (end + n_ret) - ret2;
        }
    }
}

fn merge_process_range(klp: &mut KlpState, n_start: usize, n_end: usize) -> usize {
    if n_end <= n_start {
        return 0;
    }
    let mut morphs: Vec<MaMorph> = Vec::new();
    let mut source: Vec<u16> = Vec::new();
    let mut ireguler: Vec<IregulerStr> = Vec::new();
    for w in &klp.words[n_start..n_end] {
        morphs.extend(w.morphs.iter().cloned());
        source.extend_from_slice(&w.source);
        ireguler.extend(w.ireguler.iter().cloned());
    }
    let merged = MaWord {
        source,
        morphs,
        b_str_type: 3,
        ireguler,
        b_sentence_end: false,
    };
    let n = n_end - n_start;
    klp.words.drain(n_start..n_end);
    klp.words.insert(n_start, merged);
    n - 1
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_juche_date_link(klp: &mut KlpState, _d: &KmaDicts, pn_index: &mut isize) -> bool {
    let idx = *pn_index;
    if (idx + 3) as usize >= klp.words.len() {
        return false;
    }
    if !is_pumsa_array(&klp.words[idx as usize..], b"*IRI")
        || !wcscmp_c(&klp.words[idx as usize], "juc9")
    {
        return false;
    }
    if klp.words[(idx + 1) as usize].b_str_type != 2
        || !wcscmp_c(&klp.words[(idx + 2) as usize], "-")
    {
        return false;
    }
    let Some(day) = is_day_str(&klp.words[(idx + 3) as usize]) else {
        return false;
    };
    let Some((year, month, day2)) = is_juche_date(&klp.words[(idx + 1) as usize]) else {
        return false;
    };
    if day2 >= day || !is_link_day(year + 0x777, month, day) {
        return false;
    }
    *pn_index += 1;
    remove_anal(klp, *pn_index as usize, 3);
    insert_digit_pron(klp, pn_index, &digits::digit_to_china_pron_value(year));
    insert_anal(klp, *pn_index as usize, "nyeN", b'6');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, " ", b'k');
    *pn_index += 1;
    insert_digit_pron(
        klp,
        pn_index,
        &digits::digit_to_china_pron_value_special(month),
    );
    insert_anal(klp, *pn_index as usize, "weL", b'6');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, " ", b'k');
    *pn_index += 1;
    insert_digit_pron(klp, pn_index, &digits::digit_to_china_pron_value(day2));
    insert_anal(klp, *pn_index as usize, "iL", b'6');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, "bute", b'W');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, " ", b'k');
    *pn_index += 1;
    insert_digit_pron(klp, pn_index, &digits::digit_to_china_pron_value(day));
    insert_anal(klp, *pn_index as usize, "iL", b'6');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, "qaji", b'W');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, " ", b'k');
    true
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_picture(klp: &mut KlpState, d: &KmaDicts, pn_index: &mut isize) -> bool {
    let idx = *pn_index;
    if (idx + 4) as usize >= klp.words.len() {
        return false;
    }
    if !is_pumsa_array(&klp.words[idx as usize..], b"*kIRI") {
        return false;
    }
    let Some(d1) = is_digit_value(&klp.words[(idx + 2) as usize]) else {
        return false;
    };
    let Some(d2) = is_digit_value(&klp.words[(idx + 4) as usize]) else {
        return false;
    };
    if !wcscmp_c(&klp.words[(idx + 3) as usize], "-")
        || !is_pre_type_code(d, &klp.words[idx as usize], 1)
    {
        return false;
    }
    *pn_index += 2;
    remove_anal(klp, *pn_index as usize, 3);
    insert_digit_pron(klp, pn_index, &digits::digit_to_china_pron_value(d1));
    insert_anal(klp, *pn_index as usize, "9", b'W');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, " ", b'k');
    *pn_index += 1;
    insert_digit_pron(klp, pn_index, &digits::digit_to_china_pron_value(d2));
    *pn_index = pn_index.saturating_sub(1);
    true
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_date(klp: &mut KlpState, d: &KmaDicts, pn_index: &mut isize, ch_cmp: u8) -> bool {
    let idx = *pn_index;
    if (idx + 4) as usize >= klp.words.len() {
        return false;
    }
    if !is_pumsa_array(&klp.words[idx as usize..], b"IRIRI") {
        return false;
    }
    if !is_symbol_str(&klp.words[(idx + 1) as usize], ch_cmp)
        || !is_symbol_str(&klp.words[(idx + 3) as usize], ch_cmp)
    {
        return false;
    }
    let Some(year) = is_year_str(&klp.words[idx as usize]) else {
        return false;
    };
    let Some(month) = is_month_str(&klp.words[(idx + 2) as usize]) else {
        return false;
    };
    let Some(day) = is_day_str(&klp.words[(idx + 4) as usize]) else {
        return false;
    };
    if !is_link_day(year, month, day) {
        return false;
    }
    let _ = d;
    remove_anal(klp, idx as usize, 5);
    insert_digit_pron(klp, pn_index, &digits::digit_to_china_pron_value(year));
    insert_anal(klp, *pn_index as usize, "nyeN", b'6');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, " ", b'k');
    *pn_index += 1;
    insert_digit_pron(
        klp,
        pn_index,
        &digits::digit_to_china_pron_value_special(month),
    );
    insert_anal(klp, *pn_index as usize, "weL", b'6');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, " ", b'k');
    *pn_index += 1;
    insert_digit_pron(klp, pn_index, &digits::digit_to_china_pron_value(day));
    insert_anal(klp, *pn_index as usize, "iL", b'6');
    true
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_dot_date(klp: &mut KlpState, d: &KmaDicts, pn_index: &mut isize) -> bool {
    let idx = *pn_index;
    if !is_pumsa_array(&klp.words[idx as usize..], b"I") {
        return false;
    }
    let Some((year, month, day)) = is_year_date(&klp.words[idx as usize]) else {
        return false;
    };
    remove_anal(klp, idx as usize, 1);
    insert_digit_pron(klp, pn_index, &digits::digit_to_china_pron_value(year));
    insert_anal(klp, *pn_index as usize, "nyeN", b'6');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, " ", b'k');
    *pn_index += 1;
    insert_digit_pron(
        klp,
        pn_index,
        &digits::digit_to_china_pron_value_special(month),
    );
    insert_anal(klp, *pn_index as usize, "weL", b'6');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, " ", b'k');
    *pn_index += 1;
    insert_digit_pron(klp, pn_index, &digits::digit_to_china_pron_value(day));
    insert_anal(klp, *pn_index as usize, "iL", b'6');
    *pn_index += 1;
    if (*pn_index as usize) + 1 < klp.words.len() {
        let m2d2 = is_month_date(&klp.words[(*pn_index + 1) as usize]);
        let is_range = wcscmp_c(&klp.words[*pn_index as usize], "~")
            || wcscmp_c(&klp.words[*pn_index as usize], "-");
        if let (Some((m2, d2)), true) = (m2d2, is_range) {
            remove_anal(klp, *pn_index as usize, 2);
            insert_anal(klp, *pn_index as usize, "bute", b'W');
            *pn_index += 1;
            insert_anal(klp, *pn_index as usize, " ", b'k');
            *pn_index += 1;
            insert_digit_pron(
                klp,
                pn_index,
                &digits::digit_to_china_pron_value_special(m2),
            );
            insert_anal(klp, *pn_index as usize, "weL", b'6');
            *pn_index += 1;
            insert_anal(klp, *pn_index as usize, " ", b'k');
            *pn_index += 1;
            insert_digit_pron(klp, pn_index, &digits::digit_to_china_pron_value(d2));
            insert_anal(klp, *pn_index as usize, "iL", b'6');
        }
    }
    let _ = d;
    true
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_time(klp: &mut KlpState, d: &KmaDicts, pn_index: &mut isize, ch_cmp: u8) -> bool {
    let idx = *pn_index;
    if (idx + 4) as usize >= klp.words.len() {
        return false;
    }
    if !is_pumsa_array(&klp.words[idx as usize..], b"IRIRI") {
        return false;
    }
    if !is_symbol_str(&klp.words[(idx + 1) as usize], ch_cmp)
        || !is_symbol_str(&klp.words[(idx + 3) as usize], ch_cmp)
    {
        return false;
    }
    let Some(hour) = is_hour_str(&klp.words[idx as usize]) else {
        return false;
    };
    let Some(minute) = is_minute_sec_str(&klp.words[(idx + 2) as usize]) else {
        return false;
    };
    let Some(sec) = is_minute_sec_str(&klp.words[(idx + 4) as usize]) else {
        return false;
    };
    let _ = d;
    remove_anal(klp, idx as usize, 5);
    insert_digit_pron(klp, pn_index, &digits::digit_to_korean_pron_value(hour));
    insert_anal(klp, *pn_index as usize, "si", b'6');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, " ", b'k');
    *pn_index += 1;
    insert_digit_pron(klp, pn_index, &digits::digit_to_china_pron_value(minute));
    insert_anal(klp, *pn_index as usize, "buN", b'6');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, " ", b'k');
    *pn_index += 1;
    insert_digit_pron(klp, pn_index, &digits::digit_to_china_pron_value(sec));
    insert_anal(klp, *pn_index as usize, "co", b'6');
    true
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_month_link(klp: &mut KlpState, d: &KmaDicts, pn_index: &mut isize) -> bool {
    let idx = *pn_index;
    if (idx + 2) as usize >= klp.words.len() {
        return false;
    }
    if !is_pumsa_array(&klp.words[idx as usize..], b"IRI")
        || !wcscmp_c(&klp.words[(idx + 1) as usize], "-")
    {
        return false;
    }
    let Some((y1, m1)) = is_ym_date(&klp.words[idx as usize]) else {
        return false;
    };
    let Some((y2, m2)) = is_ym_date(&klp.words[(idx + 2) as usize]) else {
        return false;
    };
    let _ = d;
    remove_anal(klp, idx as usize, 3);
    insert_digit_pron(klp, pn_index, &digits::digit_to_china_pron_value(y1));
    insert_anal(klp, *pn_index as usize, "nyeN", b'6');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, " ", b'k');
    *pn_index += 1;
    insert_digit_pron(
        klp,
        pn_index,
        &digits::digit_to_china_pron_value_special(m1),
    );
    insert_anal(klp, *pn_index as usize, "weL", b'6');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, ",", b'M');
    *pn_index += 1;
    insert_digit_pron(klp, pn_index, &digits::digit_to_china_pron_value(y2));
    insert_anal(klp, *pn_index as usize, "nyeN", b'6');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, " ", b'k');
    *pn_index += 1;
    insert_digit_pron(
        klp,
        pn_index,
        &digits::digit_to_china_pron_value_special(m2),
    );
    insert_anal(klp, *pn_index as usize, "weL", b'6');
    insert_anal(klp, *pn_index as usize, "co", b'6');
    true
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_tele_number(klp: &mut KlpState, d: &KmaDicts, pn_index: &mut isize) -> bool {
    let idx = *pn_index;
    if (idx + 2) as usize >= klp.words.len() {
        return false;
    }
    if !is_pumsa_array(&klp.words[idx as usize..], b"IRI")
        || !wcscmp_c(&klp.words[(idx + 1) as usize], "-")
    {
        return false;
    }
    if is_length_str(&klp.words[idx as usize], 3).is_none()
        || is_length_str(&klp.words[(idx + 2) as usize], 4).is_none()
    {
        return false;
    }
    let d1 = klp.words[idx as usize].source.clone();
    let d2 = klp.words[(idx + 2) as usize].source.clone();
    let _ = d;
    remove_anal(klp, idx as usize, 3);
    insert_digit_pron(klp, pn_index, &digits::digit_to_one_pron_value_large(&d1));
    insert_anal(klp, *pn_index as usize, "9", b'W');
    *pn_index += 1;
    insert_anal(klp, *pn_index as usize, ",", b'M');
    *pn_index += 1;
    insert_digit_pron(klp, pn_index, &digits::digit_to_one_pron_value_large(&d2));
    *pn_index = pn_index.saturating_sub(1);
    true
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_power_split(klp: &mut KlpState, d: &KmaDicts, pn_index: &mut isize) -> bool {
    let idx = *pn_index;
    if (idx + 2) as usize >= klp.words.len() {
        return false;
    }
    let w_info = klp.words[idx as usize].clone();
    if is_pumsa_array(&klp.words[idx as usize..], b"Ik*") && is_sub_split(&w_info) {
        remove_anal(klp, idx as usize, 1);
        let n = w_info.morphs.len();
        let mut i = 0usize;
        while i < n {
            let v = digits::wtoi(&morph_pyogi_u16(&w_info.morphs[i]));
            insert_digit_pron(klp, pn_index, &digits::digit_to_china_pron_value(v));
            insert_anal(klp, *pn_index as usize, "9", b'W');
            *pn_index += 1;
            insert_anal(klp, *pn_index as usize, " ", b'k');
            *pn_index += 1;
            i += 2;
        }
        *pn_index = pn_index.saturating_sub(1);
        return true;
    }
    if w_info.morphs.first().map(|m| m.ch_tag) != Some(b'I') || w_info.b_str_type != 2 {
        return false;
    }
    if w_info.morphs.len() < 4 {
        return false;
    }
    let _ = d;
    remove_anal(klp, idx as usize, 1);
    let n = w_info.morphs.len();
    let mut i = 0usize;
    while i < n {
        insert_digit_pron(
            klp,
            pn_index,
            &digits::digit_to_china_pron_value_large(&morph_pyogi_u16(&w_info.morphs[i])),
        );
        if n - 1 != i {
            insert_anal(klp, *pn_index as usize, "zeM", b'0');
            *pn_index += 1;
            insert_anal(klp, *pn_index as usize, " ", b'k');
            *pn_index += 1;
        }
        i += 2;
    }
    *pn_index = pn_index.saturating_sub(1);
    true
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_operator(klp: &mut KlpState, d: &KmaDicts, n_index: usize) -> bool {
    if n_index + 2 < klp.words.len() && is_pumsa_array(&klp.words[n_index..], b"IRI") {
        let op = &klp.words[n_index + 1];
        if op.source.len() == 1 {
            let w_code = op.source[0];
            if n_index + 3 < klp.words.len() {
                let c = w_code as u8;
                if c == b'-' {
                    if klp.words[n_index + 3].morphs.first().map(|m| m.ch_tag) == Some(b'6') {
                        remove_anal(klp, n_index + 1, 1);
                        return true;
                    }
                } else if c == b'/' && klp.words[n_index + 3].b_str_type != 0 {
                    klp.words.swap(n_index, n_index + 2);
                    remove_anal(klp, n_index + 1, 1);
                    insert_anal(klp, n_index + 1, "buN9", b'W');
                    return true;
                }
            }
            if let Some(pch) = get_symbol_type_code(d, w_code, 2) {
                remove_anal(klp, n_index + 1, 1);
                insert_anal(klp, n_index + 1, &String::from_utf8_lossy(&pch), b'0');
                return true;
            }
        }
    }
    if n_index + 1 < klp.words.len()
        && is_pumsa_array(&klp.words[n_index..], b"RI")
        && wcscmp_c(&klp.words[n_index], "-")
    {
        remove_anal(klp, n_index, 1);
        insert_anal(klp, n_index, "minus_", b'0');
        return false;
    }
    false
}

fn process_mail_address(klp: &mut KlpState, d: &KmaDicts, n_index: usize) -> bool {
    if n_index + 2 >= klp.words.len() {
        return false;
    }
    if !is_pumsa_array(&klp.words[n_index..], b"JRR") {
        return false;
    }
    if !wcscmp_c(&klp.words[n_index + 1], ":") || !wcscmp_c(&klp.words[n_index + 2], "//") {
        return false;
    }
    if !is_pre_type_code(d, &klp.words[n_index], 6) {
        return false;
    }
    remove_anal(klp, n_index, 3);
    let mut i = n_index;
    while i < klp.words.len() {
        if klp.words[i].morphs.first().map(|m| m.ch_tag) != Some(b'J') {
            if klp.words[i].source.len() == 1 {
                let pch = get_symbol_type_code(d, klp.words[i].source[0], 3);
                remove_anal(klp, i, 1);
                if let Some(pc) = pch {
                    insert_anal(klp, i, &String::from_utf8_lossy(&pc), b'0');
                } else {
                    i = i.saturating_sub(1);
                }
            } else {
                break;
            }
        }
        i += 1;
    }
    true
}

fn process_unit_symbol(d: &KmaDicts, klp: &mut KlpState, n_index: usize) -> bool {
    let w = &klp.words[n_index];
    let mut pch = if w.source.len() == 1 {
        get_symbol_type_code(d, w.source[0], 1)
    } else {
        None
    };
    if pch.is_none() {
        pch = get_str_type_code(d, &w.source, 1);
    }
    let Some(pc) = pch else {
        return false;
    };
    remove_anal(klp, n_index, 1);
    insert_anal(klp, n_index, &String::from_utf8_lossy(&pc), b'6');
    true
}

fn process_unit(klp: &mut KlpState, d: &KmaDicts, n_index: usize) -> bool {
    if n_index + 1 < klp.words.len() {
        if is_pumsa_array(&klp.words[n_index..], b"I6") && process_unit_symbol(d, klp, n_index + 1)
        {
            return true;
        }
        if is_pumsa_array(&klp.words[n_index..], b"IH6")
            && klp.words[n_index + 1].morphs.len() == 1
            && process_unit_symbol(d, klp, n_index + 2)
        {
            return true;
        }
    }
    let w0 = &klp.words[n_index];
    if w0.morphs.first().map(|m| m.ch_tag) == Some(b'6')
        && w0.source.len() == 1
        && (w0.source[0] >> 8) != 0
        && let Some(pch) = get_symbol_type_code(d, w0.source[0], 1)
    {
        remove_anal(klp, n_index, 1);
        insert_anal(klp, n_index, &String::from_utf8_lossy(&pch), b'6');
        return true;
    }
    false
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_symbol(klp: &mut KlpState, d: &KmaDicts, pn_index: &mut isize) -> bool {
    let idx = *pn_index;
    let w0 = klp.words[idx as usize].clone();
    let tag = w0.morphs.first().map_or(0, |m| m.ch_tag);
    if tag.wrapping_add(0xae) >= 2 {
        return false;
    }
    let s_var3 = w0.source.len();
    remove_anal(klp, idx as usize, 1);
    if is_remove_symbol(&w0.source) {
        insert_anal(klp, idx as usize, " ", b'k');
        return true;
    }
    let mut i = 0usize;
    while i < s_var3 {
        let c = w0.source[i];
        if let Some(pc) = get_symbol_type_code(d, c, 4) {
            insert_anal(klp, idx as usize, &String::from_utf8_lossy(&pc), b'0');
            *pn_index += 1;
            i += 1;
            continue;
        }
        if let Some(pc) = get_symbol_type_code(d, c, 6) {
            insert_anal(klp, idx as usize, "do*g_lami", b'0');
            *pn_index += 1;
            insert_anal(klp, *pn_index as usize, " ", b'k');
            *pn_index += 1;
            insert_anal(klp, *pn_index as usize, &String::from_utf8_lossy(&pc), b'0');
            *pn_index += 1;
            insert_anal(klp, *pn_index as usize, ",", b'M');
            *pn_index += 1;
            i += 1;
            continue;
        }
        let t5 = get_symbol_type_code(d, c, 5);
        let t2 = if t5.is_none() {
            get_symbol_type_code(d, c, 2)
        } else {
            None
        };
        if let Some(pc) = t5.or(t2) {
            insert_anal(klp, idx as usize, &String::from_utf8_lossy(&pc), b'0');
            *pn_index += 1;
            insert_anal(klp, *pn_index as usize, " ", b'k');
            *pn_index += 1;
            i += 1;
            continue;
        }
        i += 1;
    }
    *pn_index = pn_index.saturating_sub(1);
    true
}

#[allow(unused_assignments)]
#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_words_month(klp: &mut KlpState, d: &KmaDicts, pn_index: &mut isize) -> bool {
    let idx = *pn_index;
    if (idx + 1) as usize >= klp.words.len() || !is_pumsa_array(&klp.words[idx as usize..], b"I*") {
        return false;
    }
    let Some((month, mut day)) = is_month_date(&klp.words[idx as usize]) else {
        return false;
    };
    let mut sw: Vec<u16> = klp.words[idx as usize].source.clone();
    sw.extend_from_slice(&word_pyogi_u16(&klp.words[(idx + 1) as usize]));
    if !is_pre_type_code_str(d, &sw, 2) {
        return false;
    }
    remove_anal(klp, idx as usize, 1);
    let f_mode = if month < 10 || day != 12 {
        day == 0x14 || day == 10 || day == 0x1e
    } else {
        day = 0xc;
        true
    };
    insert_digit_pron(klp, pn_index, &digits::decimal_read(month, day, f_mode));
    true
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_order_digital(klp: &mut KlpState, d: &KmaDicts, pn_index: &mut isize) -> bool {
    let idx = *pn_index;
    if (idx + 1) as usize >= klp.words.len() || !is_pumsa_array(&klp.words[idx as usize..], b"I*") {
        return false;
    }
    let Some(n) = is_digit_str(&klp.words[idx as usize]) else {
        return false;
    };
    if !is_pre_type_code(d, &klp.words[(idx + 1) as usize], 4) {
        return false;
    }
    remove_anal(klp, idx as usize, 1);
    insert_digit_pron(klp, pn_index, &digits::digit_to_one_pron_value(n));
    true
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_16_digit(klp: &mut KlpState, d: &KmaDicts, pn_index: &mut isize) -> bool {
    let idx = *pn_index;
    if (idx + 1) as usize >= klp.words.len() {
        return false;
    }
    if !is_pumsa_array(&klp.words[idx as usize..], b"IJ")
        || !wcscmp_c(&klp.words[idx as usize], "0")
    {
        return false;
    }
    let wcs: Vec<u16> = klp.words[(idx + 1) as usize].source.clone();
    if wcs.first() != Some(&u16::from(b'X')) && wcs.first() != Some(&u16::from(b'x')) {
        return false;
    }
    let mut sw: Vec<u16> = if wcs.len() < 2 {
        Vec::new()
    } else {
        wcs[1..].to_vec()
    };
    if !is_16_only_digit(&sw) {
        return false;
    }
    let mut n = idx + 2;
    while (n as usize) < klp.words.len() {
        if sw.len() == 4 || !is_16_digit(d, &klp.words[n as usize]) {
            break;
        }
        sw.extend_from_slice(&klp.words[n as usize].source);
        n += 1;
    }
    if sw.len() != 4 {
        return false;
    }
    remove_anal(klp, idx as usize, (n - idx) as usize);
    insert_anal(klp, idx as usize, "go*9Gs_", b'0');
    *pn_index += 1;
    insert_digit_pron(klp, pn_index, &digits::digit_to_16_pron(&sw));
    *pn_index = pn_index.saturating_sub(1);
    true
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn process_digital(klp: &mut KlpState, d: &KmaDicts, pn_index: &mut isize) -> bool {
    let idx = *pn_index;
    if !is_pumsa_array(&klp.words[idx as usize..], b"I") {
        return false;
    }
    let w_info = klp.words[idx as usize].clone();
    let digit = is_digit_str(&w_info);
    let Some(n) = digit else {
        let m0 = morph_pyogi_u16(&w_info.morphs[0]);
        let m2 = if w_info.morphs.len() > 2 {
            morph_pyogi_u16(&w_info.morphs[2])
        } else {
            Vec::new()
        };
        remove_anal(klp, idx as usize, 1);
        let pc = digits::decimal_read_large(&m0, &m2);
        insert_digit_pron(klp, pn_index, &pc);
        *pn_index = pn_index.saturating_sub(1);
        return true;
    };
    let mut i2 = idx;
    if (idx as usize) + 1 < klp.words.len() && n < 100 {
        if is_pre_type_code(d, &klp.words[(idx + 1) as usize], 3) {
            if idx > 0 && is_pre_type_code(d, &klp.words[(idx - 1) as usize], 5) {
                remove_anal(klp, idx as usize, 1);
                let pc = digits::digit_to_china_pron_value(n);
                insert_digit_pron(klp, pn_index, &pc);
                return true;
            }
            i2 = idx;
            remove_anal(klp, i2 as usize, 1);
            let pc = digits::digit_to_korean_pron_value(n);
            insert_digit_pron(klp, pn_index, &pc);
            return true;
        }
        i2 = idx;
    }
    remove_anal(klp, i2 as usize, 1);
    let pc = digits::digit_to_china_pron_value_large(&w_info.source);
    insert_digit_pron(klp, pn_index, &pc);
    *pn_index = pn_index.saturating_sub(1);
    true
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
pub(crate) fn main_pre_process(klp: &mut KlpState, d: &KmaDicts) {
    if klp.words.is_empty() {
        return;
    }
    process_del_space_symbol(klp);
    process_digit_merge(klp);
    process_digit_seperate(klp);
    process_del_sp_symbol(klp);
    process_ma_repeat(klp);
    let mut idx = 0isize;
    while (idx as usize) < klp.words.len() {
        if !process_juche_date_link(klp, d, &mut idx)
            && !process_picture(klp, d, &mut idx)
            && !process_date(klp, d, &mut idx, b'/')
            && !process_date(klp, d, &mut idx, b'/')
            && !process_dot_date(klp, d, &mut idx)
            && !process_time(klp, d, &mut idx, b':')
            && !process_month_link(klp, d, &mut idx)
            && !process_tele_number(klp, d, &mut idx)
            && !process_power_split(klp, d, &mut idx)
        {
            process_operator(klp, d, idx as usize);
            process_mail_address(klp, d, idx as usize);
            process_unit(klp, d, idx as usize);
            if !process_symbol(klp, d, &mut idx)
                && !process_words_month(klp, d, &mut idx)
                && !process_order_digital(klp, d, &mut idx)
                && !process_16_digit(klp, d, &mut idx)
                && !process_digital(klp, d, &mut idx)
                && !process_engilish(klp, d, &mut idx)
            {}
        }
        idx += 1;
        if idx as usize >= klp.words.len() {
            break;
        }
    }
    process_del_sp_symbol(klp);
    process_modify_ma_result_to(klp);
    process_pyogi_to_wansong(klp);
    process_last_cut(klp);
    process_last_merge(klp);
    set_trailing_symbol_cvc(klp);
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
fn set_trailing_symbol_cvc(klp: &mut KlpState) {
    for w in &mut klp.words {
        let Some(&last_ch) = w.source.last() else {
            continue;
        };
        if !tables::is_sen_symbol(last_ch) {
            continue;
        }
        let Some(last_m) = w.morphs.last_mut() else {
            continue;
        };
        if !matches!(last_m.ch_tag, b'L' | b'M') || !last_m.cvc.is_empty() {
            continue;
        }
        last_m.cvc = vec![last_ch as u8];
        if last_ch != 0x2c {
            w.morphs.push(MaMorph {
                ch_tag: b'L',
                pyogi: Vec::new(),
                cvc: Vec::new(),
                prob: 0.0,
                b_merged: false,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ma::{MaMorph, MaWord, klp_state_with_words};

    fn w(source: &str, tag: u8, b_str_type: u8) -> MaWord {
        MaWord {
            source: source.encode_utf16().collect(),
            morphs: vec![MaMorph {
                ch_tag: tag,
                pyogi: source.as_bytes().to_vec(),
                cvc: Vec::new(),
                prob: 0.0,
                b_merged: false,
            }],
            b_str_type,
            ireguler: Vec::new(),
            b_sentence_end: false,
        }
    }

    fn srcs(k: &KlpState) -> Vec<String> {
        k.words
            .iter()
            .map(|x| String::from_utf16_lossy(&x.source))
            .collect()
    }

    fn btypes(k: &KlpState) -> Vec<u8> {
        k.words.iter().map(|x| x.b_str_type).collect()
    }

    #[test]
    fn digit_merge_3_words() {
        let mut k =
            klp_state_with_words(vec![w("010", b'I', 0), w("-", b'L', 0), w("1234", b'I', 0)]);
        process_digit_merge(&mut k);
        assert_eq!(srcs(&k), ["010-1234"]);
        assert_eq!(btypes(&k), [2]);
    }

    #[test]
    fn digit_merge_exactly_three() {
        let mut k =
            klp_state_with_words(vec![w("95", b'I', 0), w("-", b'L', 0), w("1234", b'I', 0)]);
        process_digit_merge(&mut k);
        assert_eq!(srcs(&k), ["95-1234"]);
        assert_eq!(btypes(&k), [2]);
    }

    #[test]
    fn digit_merge_over_99_chars() {
        let long = "1".repeat(99);
        let mut k = klp_state_with_words(vec![w("0", b'I', 0), w("-", b'L', 0), w(&long, b'I', 0)]);
        process_digit_merge(&mut k);
        assert_eq!(k.words.len(), 3);
    }

    #[test]
    fn digit_merge_immi_no_merge() {
        let mut k =
            klp_state_with_words(vec![w("123", b'I', 0), w(",", b'M', 0), w("456", b'I', 0)]);
        process_digit_merge(&mut k);
        assert_eq!(k.words.len(), 3);
    }

    #[test]
    fn digit_seperate_123456789() {
        let mut k = klp_state_with_words(vec![
            w("123", b'I', 0),
            w(",", b'M', 0),
            w("456", b'I', 0),
            w(",", b'M', 0),
            w("789", b'I', 0),
        ]);
        process_digit_seperate(&mut k);
        assert_eq!(srcs(&k), ["123456789"]);
        assert_eq!(btypes(&k), [0]);
        assert_eq!(k.words[0].morphs[0].pyogi, b"123456789");
    }

    #[test]
    fn digit_seperate_1_234() {
        let mut k = klp_state_with_words(vec![w("1", b'I', 0), w(",", b'M', 0), w("234", b'I', 0)]);
        process_digit_seperate(&mut k);
        assert_eq!(srcs(&k), ["1234"]);
    }

    #[test]
    fn digit_seperate_merged_pair_continues() {
        let mut k = klp_state_with_words(vec![
            w("1", b'I', 0),
            w(",", b'M', 0),
            w("1-2", b'I', 2),
            w(",", b'M', 0),
            w("345", b'I', 0),
        ]);
        process_digit_seperate(&mut k);
        assert_eq!(srcs(&k), ["11-2345"]);
    }

    #[test]
    fn digit_seperate_no_merge_advances() {
        let mut k = klp_state_with_words(vec![
            w("123", b'I', 0),
            w(",", b'M', 0),
            w("456", b'I', 0),
            w(",", b'M', 0),
            w(",", b'M', 0),
        ]);
        process_digit_seperate(&mut k);
        assert_eq!(srcs(&k), ["123", ",", "456", ",", ","]);
    }

    #[test]
    fn digit_seperate_skip_long() {
        let mut k = klp_state_with_words(vec![
            w("1", b'I', 0),
            w(",", b'M', 0),
            w("234-5678", b'I', 2),
        ]);
        process_digit_seperate(&mut k);
        assert_eq!(k.words.len(), 3);
    }

    #[test]
    fn del_sp_kk_then_advance() {
        let mut k = klp_state_with_words(vec![w("a", b'k', 0), w("b", b'k', 0), w("c", b'M', 0)]);
        process_del_sp_symbol(&mut k);
        assert_eq!(k.words.len(), 2);
        assert_eq!(first_tags(&k), [b'k', b'M']);
    }

    #[test]
    fn del_sp_km() {
        let mut k = klp_state_with_words(vec![w("a", b'k', 0), w("b", b'M', 0), w("c", b'x', 0)]);
        process_del_sp_symbol(&mut k);
        assert_eq!(first_tags(&k), [b'M', b'x']);
    }

    #[test]
    fn del_sp_mk() {
        let mut k = klp_state_with_words(vec![w("a", b'M', 0), w("b", b'k', 0), w("c", b'M', 0)]);
        process_del_sp_symbol(&mut k);
        assert_eq!(first_tags(&k), [b'M', b'M']);
    }

    #[test]
    fn del_sp_km_two() {
        let mut k = klp_state_with_words(vec![w("a", b'k', 0), w("b", b'M', 0)]);
        process_del_sp_symbol(&mut k);
        assert_eq!(first_tags(&k), [b'M']);
    }

    fn first_tags(k: &KlpState) -> Vec<u8> {
        k.words.iter().map(|x| x.morphs[0].ch_tag).collect()
    }

    #[test]
    fn remove_symbol_constants() {
        assert!(is_remove_symbol(&"**".encode_utf16().collect::<Vec<_>>()));
        assert!(is_remove_symbol(&"//x".encode_utf16().collect::<Vec<_>>()));
        assert!(is_remove_symbol(&"/*".encode_utf16().collect::<Vec<_>>()));
        assert!(is_remove_symbol(&"*/".encode_utf16().collect::<Vec<_>>()));
        assert!(is_remove_symbol(&"-".encode_utf16().collect::<Vec<_>>()));
        assert!(is_remove_symbol(&":".encode_utf16().collect::<Vec<_>>()));
        assert!(!is_remove_symbol(&"-x".encode_utf16().collect::<Vec<_>>()));
        assert!(!is_remove_symbol(&"--".encode_utf16().collect::<Vec<_>>()));
        assert!(!is_remove_symbol(&"abc".encode_utf16().collect::<Vec<_>>()));
        assert!(!is_remove_symbol(&"*".encode_utf16().collect::<Vec<_>>()));
    }

    #[test]
    fn last_cut_long_word_morph_split() {
        let source: String = "가".repeat(21);
        let pyogi: String = "ga".repeat(21);
        let mut k = klp_state_with_words(vec![MaWord {
            source: source.encode_utf16().collect(),
            morphs: vec![MaMorph {
                ch_tag: b'0',
                pyogi: pyogi.as_bytes().to_vec(),
                cvc: Vec::new(),
                prob: 0.0,
                b_merged: false,
            }],
            b_str_type: 1,
            ireguler: Vec::new(),
            b_sentence_end: false,
        }]);
        process_last_cut(&mut k);
        assert_eq!(k.words.len(), 2);
        assert_eq!(
            String::from_utf16_lossy(&k.words[0].source),
            "가".repeat(20)
        );
        assert_eq!(k.words[0].morphs.len(), 1);
        assert_eq!(
            String::from_utf8_lossy(&k.words[0].morphs[0].pyogi),
            "ga".repeat(20)
        );
        assert_eq!(k.words[0].b_str_type, 1);
        assert_eq!(String::from_utf16_lossy(&k.words[1].source), "가");
        assert_eq!(k.words[1].morphs.len(), 1);
        assert_eq!(k.words[1].morphs[0].pyogi, b"ga");
        assert_eq!(k.words[1].morphs[0].ch_tag, b'0');
        assert_eq!(k.words[1].b_str_type, 3);
        assert!(k.words[1].ireguler.is_empty(), "bIrrMorphCount = 0");
    }

    #[test]
    fn last_cut_morph_count_11() {
        let morphs: Vec<MaMorph> = (0..11)
            .map(|_| MaMorph {
                ch_tag: b'0',
                pyogi: b"ga".to_vec(),
                cvc: Vec::new(),
                prob: 0.0,
                b_merged: false,
            })
            .collect();
        let mut k = klp_state_with_words(vec![MaWord {
            source: "가".repeat(11).encode_utf16().collect(),
            morphs,
            b_str_type: 1,
            ireguler: Vec::new(),
            b_sentence_end: false,
        }]);
        process_last_cut(&mut k);
        assert_eq!(k.words.len(), 2);
        assert_eq!(
            String::from_utf16_lossy(&k.words[0].source),
            "가".repeat(10)
        );
        assert_eq!(k.words[0].morphs.len(), 10);
        assert_eq!(String::from_utf16_lossy(&k.words[1].source), "가");
        assert_eq!(k.words[1].morphs.len(), 1);
        assert_eq!(k.words[1].morphs[0].pyogi, b"ga");
        assert_eq!(k.words[1].b_str_type, 3);
    }

    #[test]
    fn last_cut_recursive_split() {
        let source: String = "가".repeat(42);
        let pyogi: String = "ga".repeat(42);
        let mut k = klp_state_with_words(vec![MaWord {
            source: source.encode_utf16().collect(),
            morphs: vec![MaMorph {
                ch_tag: b'0',
                pyogi: pyogi.as_bytes().to_vec(),
                cvc: Vec::new(),
                prob: 0.0,
                b_merged: false,
            }],
            b_str_type: 1,
            ireguler: Vec::new(),
            b_sentence_end: false,
        }]);
        process_last_cut(&mut k);
        let srcs: Vec<String> = k
            .words
            .iter()
            .map(|w| String::from_utf16_lossy(&w.source))
            .collect();
        assert_eq!(srcs, ["가".repeat(20), "가".repeat(20), "가".repeat(2)]);
    }

    #[test]
    fn last_cut_short_word_untouched() {
        let source: String = "가".repeat(19);
        let pyogi: String = "ga".repeat(19);
        let mut k = klp_state_with_words(vec![MaWord {
            source: source.encode_utf16().collect(),
            morphs: vec![MaMorph {
                ch_tag: b'0',
                pyogi: pyogi.as_bytes().to_vec(),
                cvc: Vec::new(),
                prob: 0.0,
                b_merged: false,
            }],
            b_str_type: 1,
            ireguler: Vec::new(),
            b_sentence_end: false,
        }]);
        process_last_cut(&mut k);
        assert_eq!(k.words.len(), 1);
    }

    #[test]
    fn word_cvc_table_positions() {
        let t = set_word_cvc_table(&"안녕".encode_utf16().collect::<Vec<u16>>());
        assert_eq!(t, vec![0, 6, 12, 18, 30, 36, 0, 0]);
    }
}
