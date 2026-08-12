use crate::dict::KmaDicts;
use crate::tables;

#[derive(Debug, Clone, Copy)]
pub struct CharInfo {
    pub f_char_type: u8,
    pub w_char: u16,
}

#[derive(Debug, Clone)]
pub struct StrInfo {
    pub f_char_type: u8,
    pub pw_str: Vec<u16>,
    pub n_str_len: usize,
}

pub fn get_uni_char_attribute(pw_code: &mut u16, d: Option<&KmaDicts>) -> u8 {
    let w = *pw_code;
    if w.wrapping_sub(0x30) < 10 {
        return 0x02;
    }
    if w.wrapping_sub(0x61) < 0x1a || w.wrapping_sub(0x41) < 0x1a {
        return 0x03;
    }
    if w == 9 || w == 0x20 {
        return 0x05;
    }
    if w == 0x28 || w == 0x3c || w == 0x5b || w == 0x7b {
        return 0x06;
    }
    if w == 0x29 || w == 0x3e || w == 0x5d || w == 0x7d {
        return 0x07;
    }
    if w == 0x22 || w == 0x27 {
        return 0x0b;
    }
    if w == 0x21 || w == 0x2e || w == 0x3f {
        return 0x08;
    }
    if w == 0x2c {
        return 0x09;
    }
    if w == 0x2d
        || w == 0x2b
        || w == 0x2f
        || w == 0x2a
        || w == 0x3d
        || w == 0x26
        || w == 0x25
        || w == 0x5e
        || w == 0x7c
    {
        return 0x0d;
    }
    if w == 10 || w == 0x0d {
        return 0x04;
    }
    if tables::is_uni_korean_jamo(w) {
        return 0x01;
    }
    if tables::is_uni_korean_code(w) {
        return 0x00;
    }
    if let Some(d) = d
        && d.unipron.contains_key(&vec![0x31, w])
    {
        return 0x0c;
    }
    if let Some(hanja) = d.map(|d| &d.hanja) {
        let v = hanja.get(w);
        if v != w && v != 0 {
            *pw_code = v;
            return 0x00;
        }
    }
    let mut w2 = w;
    let r = match w {
        0x300d | 0x226b | 0x300b | 0xf10a => 0x07,
        0x3008 => {
            w2 = 0x3c;
            0x06
        }
        0x201d => {
            w2 = 0x22;
            0x07
        }
        0x201c => {
            w2 = 0x22;
            0x06
        }
        0x3009 => {
            w2 = 0x3e;
            0x07
        }
        0x226a | 0x300a | 0x300c | 0x300e | 0x300f | 0x3010 | 0x3011 | 0xf109 => 0x06,
        0x3014 => {
            w2 = 0x5b;
            0x06
        }
        0x3015 => {
            w2 = 0x5d;
            0x07
        }
        0xf107 => {
            w2 = 0x2c;
            0x06
        }
        0xf108 => {
            w2 = 0x27;
            0x07
        }
        _ => 0x0a,
    };
    *pw_code = w2;
    r
}

#[must_use]
pub fn divide_str_to_char(pch: &[u16], d: Option<&KmaDicts>) -> Vec<CharInfo> {
    let mut out = Vec::with_capacity(pch.len());
    for &w in pch {
        let mut wc = w;
        let t = get_uni_char_attribute(&mut wc, d);
        out.push(CharInfo {
            f_char_type: t,
            w_char: wc,
        });
    }
    out
}

#[must_use]
#[expect(
    clippy::many_single_char_names,
    reason = "C port: original single-letter variable names"
)]
pub fn merge_char_to_sub_str(chars: &[CharInfo]) -> Vec<StrInfo> {
    let mut out: Vec<StrInfo> = Vec::new();
    let n = chars.len();
    let mut i = 0usize;
    while i < n {
        let t = chars[i].f_char_type;
        let size;
        if t == 0x02 || t == 0x00 || t == 0x03 || t == 0x05 || t == 0x0d {
            let mut j = i + 1;
            if j < n && t == chars[j].f_char_type {
                let mut k = 2;
                while k < 0x14 && j + 1 < n && t == chars[j + 1].f_char_type {
                    j += 1;
                    k += 1;
                }
                size = k;
            } else {
                size = 1;
            }
        } else {
            size = 1;
        }
        let mut s = Vec::with_capacity(size);
        for c in &chars[i..i + size] {
            s.push(c.w_char);
        }
        let n_str_len = s.len();
        out.push(StrInfo {
            f_char_type: t,
            pw_str: s,
            n_str_len,
        });
        i += size;
    }
    out
}

#[expect(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
pub fn unit_str_merge_process(infos: &mut Vec<StrInfo>, d: Option<&KmaDicts>) {
    let Some(d) = d else { return };
    let mut n = 0usize;
    while n < infos.len() {
        let mut sw_temp: Vec<u16> = vec![0x31];
        let mut n_end: isize = -1;
        let mut sw_copy: Vec<u16> = Vec::new();
        let mut i = n;
        loop {
            let s_len = infos[i].pw_str.len();
            if sw_temp.len() + s_len > 6 {
                break;
            }
            sw_temp.extend_from_slice(&infos[i].pw_str);
            if is_str_type_code(d, &sw_temp) {
                sw_copy = sw_temp[1..].to_vec();
                n_end = i as isize;
            }
            if infos.len() <= i + 1 {
                break;
            }
            i += 1;
        }
        if n_end == -1 {
            n += 1;
            continue;
        }
        if n_end as usize == n {
            infos[n].f_char_type = 0x0c;
        } else {
            let n_end_u = n_end as usize;
            let mut new_infos: Vec<StrInfo> = Vec::with_capacity(infos.len() - (n_end_u - n));
            new_infos.extend_from_slice(&infos[..n]);
            new_infos.push(StrInfo {
                f_char_type: 0x0c,
                pw_str: sw_copy.clone(),
                n_str_len: sw_copy.len(),
            });
            new_infos.extend_from_slice(&infos[n_end_u + 1..]);
            *infos = new_infos;
        }
        n += 1;
    }
}

#[must_use]
pub fn is_str_type_code(d: &KmaDicts, w: &[u16]) -> bool {
    d.strpron.contains_key(w)
}

#[must_use]
pub fn get_char_type_str(pch: &[u16], b_unit: bool, d: Option<&KmaDicts>) -> Vec<StrInfo> {
    let chars = divide_str_to_char(pch, d);
    let mut infos = merge_char_to_sub_str(&chars);
    if b_unit {
        unit_str_merge_process(&mut infos, d);
    }
    infos
}

#[must_use]
pub const fn set_tag_from_attribute(att: u8) -> u8 {
    match att {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_types() {
        let mut w = 0xac00u16;
        assert_eq!(get_uni_char_attribute(&mut w, None), 0);
        let mut w = u16::from(b'1');
        assert_eq!(get_uni_char_attribute(&mut w, None), 2);
        let mut w = u16::from(b'A');
        assert_eq!(get_uni_char_attribute(&mut w, None), 3);
        let mut w = u16::from(b' ');
        assert_eq!(get_uni_char_attribute(&mut w, None), 5);
        let mut w = u16::from(b'.');
        assert_eq!(get_uni_char_attribute(&mut w, None), 8);
        let mut w = u16::from(b',');
        assert_eq!(get_uni_char_attribute(&mut w, None), 9);
    }

    #[test]
    fn merge_korean() {
        let s: Vec<u16> = "안녕하세요".encode_utf16().collect();
        let infos = get_char_type_str(&s, true, None);
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].f_char_type, 0);
        assert_eq!(infos[0].pw_str, s);
    }

    #[test]
    fn tags() {
        assert_eq!(set_tag_from_attribute(0x01), b'S');
        assert_eq!(set_tag_from_attribute(0x02), b'I');
        assert_eq!(set_tag_from_attribute(0x03), b'J');
        assert_eq!(set_tag_from_attribute(0x05), b'k');
        assert_eq!(set_tag_from_attribute(0x06), b'N');
        assert_eq!(set_tag_from_attribute(0x08), b'L');
        assert_eq!(set_tag_from_attribute(0x0c), b'6');
        assert_eq!(set_tag_from_attribute(0x0d), b'R');
        assert_eq!(set_tag_from_attribute(0x04), b'R');
    }
}
