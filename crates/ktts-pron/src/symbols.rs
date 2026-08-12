use crate::dicts::PronContext;
use crate::kma_code;
use crate::kma_types::WordAnal;

fn u16_bytes(v: &[u16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 2);
    for &c in v {
        out.extend_from_slice(&c.to_le_bytes());
    }
    out
}

#[must_use]
pub fn get_symbol_type_code(ctx: &PronContext, w_code: u16, w_type: u16) -> Option<String> {
    let key = [u16::from(b'0' + (w_type & 0xff) as u8), w_code];
    ctx.unipron_lookup(&u16_bytes(&key))
}

#[must_use]
pub fn is_symbol_type_code(ctx: &PronContext, w_code: u16, w_type: u16) -> bool {
    get_symbol_type_code(ctx, w_code, w_type).is_some()
}

#[must_use]
pub fn get_str_type_code(ctx: &PronContext, pw_code: &[u16], w_type: u16) -> Option<String> {
    let mut key = Vec::with_capacity(pw_code.len() + 1);
    key.push(u16::from(b'0' + (w_type & 0xff) as u8));
    key.extend_from_slice(pw_code);
    ctx.strpron_lookup(&u16_bytes(&key))
}

#[must_use]
pub fn is_str_type_code(ctx: &PronContext, pw_code: &[u16]) -> bool {
    let mut key = Vec::with_capacity(pw_code.len() + 1);
    key.push(u16::from(b'1'));
    key.extend_from_slice(pw_code);
    ctx.strpron_lookup(&u16_bytes(&key)).is_some()
}

#[must_use]
pub fn is_remove_symbol(pw_str: &[u16]) -> bool {
    for p in ["**", "//", "/*", "*/"] {
        let p: Vec<u16> = p.encode_utf16().collect();
        if pw_str.len() >= p.len() && pw_str[..p.len()] == p[..] {
            return true;
        }
    }
    pw_str == [0x2d] || pw_str == [0x3a]
}

#[must_use]
pub fn symbol_char_items(ctx: &PronContext, c: u16) -> Option<Vec<(String, u8)>> {
    if let Some(p) = get_symbol_type_code(ctx, c, 4) {
        return Some(vec![(p, b'0')]);
    }
    if let Some(p) = get_symbol_type_code(ctx, c, 6) {
        return Some(vec![
            ("do*g_lami".to_string(), b'0'),
            (" ".to_string(), b'k'),
            (p, b'0'),
            (",".to_string(), b'M'),
        ]);
    }
    let t5 = get_symbol_type_code(ctx, c, 5);
    let t2 = if t5.is_none() {
        get_symbol_type_code(ctx, c, 2)
    } else {
        None
    };
    if let Some(p) = t5.or(t2) {
        return Some(vec![(p, b'0'), (" ".to_string(), b'k')]);
    }
    None
}

#[must_use]
pub fn process_symbol(ctx: &PronContext, word_chars: &[u16]) -> Option<Vec<(String, u8)>> {
    if is_remove_symbol(word_chars) {
        return Some(vec![(" ".to_string(), b'k')]);
    }
    let mut items: Vec<(String, u8)> = Vec::new();
    for &c in word_chars {
        if let Some(mut it) = symbol_char_items(ctx, c) {
            items.append(&mut it);
        }
    }
    Some(items)
}

#[must_use]
pub fn process_unit_symbol(ctx: &PronContext, word_chars: &[u16]) -> Option<String> {
    if word_chars.len() == 1
        && let Some(p) = get_symbol_type_code(ctx, word_chars[0], 1)
    {
        return Some(p);
    }
    get_str_type_code(ctx, word_chars, 1)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitWord {
    pub tag: u8,
    pub chars: Vec<u16>,
    pub morph_count: usize,
}

#[must_use]
pub fn unit_word(word: &WordAnal) -> UnitWord {
    UnitWord {
        tag: word.morphs.first().map_or(0, |m| m.pos[0]),
        chars: word_chars(word),
        morph_count: word.morphs.len(),
    }
}

#[must_use]
pub fn word_chars(word: &WordAnal) -> Vec<u16> {
    let mut chars: Vec<u16> = Vec::new();
    for m in &word.morphs {
        if WordAnal::is_symbol_morph(m) {
            chars.extend(m.cvc.iter().map(|&b| u16::from(b)));
            continue;
        }
        let cvc = &m.cvc;
        let mut i = 0;
        while i < cvc.len() {
            if cvc[i] > 29 {
                chars.push(u16::from(cvc[i]));
                i += 1;
            } else if i + 2 < cvc.len() {
                let w = kma_code::conv_cvc_to_uni_wan(&cvc[i..i + 3]);
                if kma_code::is_uni_wansong(w) || kma_code::is_uni_korean_jamo(w) {
                    chars.push(w);
                } else {
                    chars.push(u16::from(cvc[i]));
                }
                i += 3;
            } else {
                chars.push(u16::from(cvc[i]));
                i += 1;
            }
        }
    }
    if chars.is_empty() && !word.morphs.is_empty() {
        let cvc = word.cvc();
        let mut i = 0;
        while i + 2 < cvc.len() {
            chars.push(kma_code::conv_cvc_to_uni_wan(&cvc[i..i + 3]));
            i += 3;
        }
    }
    chars
}

#[must_use]
pub fn is_pumsa_array(words: &[UnitWord], n_index: usize, pch_tag: &[u8]) -> bool {
    if n_index + pch_tag.len() > words.len() {
        return false;
    }
    for (k, &t) in pch_tag.iter().enumerate() {
        let w = &words[n_index + k];
        if t != b'*' && w.tag != t {
            return false;
        }
    }
    true
}

#[must_use]
pub fn process_unit(
    ctx: &PronContext,
    words: &[UnitWord],
    n_index: usize,
) -> Option<(usize, String)> {
    if n_index + 1 < words.len() {
        if is_pumsa_array(words, n_index, b"I6")
            && let Some(p) = process_unit_symbol(ctx, &words[n_index + 1].chars)
        {
            return Some((n_index + 1, p));
        }
        if is_pumsa_array(words, n_index, b"IH6")
            && words[n_index + 1].morph_count == 1
            && let Some(p) = process_unit_symbol(ctx, &words[n_index + 2].chars)
        {
            return Some((n_index + 2, p));
        }
    }
    let w0 = &words[n_index];
    if w0.tag == b'6'
        && w0.chars.len() == 1
        && (w0.chars[0] >> 8) != 0
        && let Some(p) = get_symbol_type_code(ctx, w0.chars[0], 1)
    {
        return Some((n_index, p));
    }
    None
}

#[must_use]
pub fn is_uni_korea_hanja(ctx: &PronContext, cp: u16) -> Option<u16> {
    let h = ctx.hanja_get(cp)?;
    if h != cp && h != 0 { Some(h) } else { None }
}

#[must_use]
pub fn symbol_word_pyogi(ctx: &PronContext, chars: &[u16]) -> String {
    let mut s = String::new();
    for &c in chars {
        if let Some(han) = is_uni_korea_hanja(ctx, c) {
            s.push_str(&kma_code::conv_cvc_to_pyogi(
                &kma_code::conv_uni_wan_to_cvc_special_pre(&[han]).0,
            ));
            continue;
        }
        if let Some(items) = symbol_char_items(ctx, c) {
            for (p, _tag) in items {
                s.push_str(&p);
            }
            continue;
        }
        s.push(char::from_u32(u32::from(c)).unwrap_or(' '));
    }
    s
}

#[must_use]
pub const fn set_tag_from_attribute(b_char_att: u8) -> u8 {
    match b_char_att {
        0x06 => b'N',
        0x07 => b'O',
        0x08 => b'L',
        0x03 => b'J',
        0x09 => b'M',
        0x01 => b'S',
        0x05 => b'k',
        0x02 => b'I',
        0x0c => b'6',
        _ => b'R',
    }
}
