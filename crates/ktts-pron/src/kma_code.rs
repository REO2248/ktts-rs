pub const CHO_PYOGI: [&str; 21] = [
    "", "", "g", "q", "n", "d", "f", "l", "m", "b", "r", "s", "v", "", "j", "z", "c", "k", "t",
    "p", "h",
];

pub const JUNG_PYOGI: [&str; 30] = [
    "", "", "", "a", "8", "ya", "y8", "e", "", "", "9", "ye", "y9", "o", "wa", "w8", "", "", "wi",
    "yo", "u", "we", "w9", "wu", "", "", "yu", "_", "yi", "i",
];

pub const JONG_PYOGI: [&str; 30] = [
    "", "", "G", "Q", "GS", "N", "NJ", "NH", "D", "L", "LG", "LM", "LB", "LS", "LT", "LP", "LH",
    "M", "", "B", "BS", "S", "V", "*", "J", "C", "K", "T", "P", "H",
];

#[must_use]
pub fn cvc_type_from_pyogi(c: u8) -> u8 {
    if b"ghqndlmbrsvfjzcktp".contains(&c) {
        0
    } else if b"wy".contains(&c) {
        1
    } else if b"aeoui_89".contains(&c) {
        2
    } else if b"NDLMBSJG*CPHQVTK".contains(&c) {
        3
    } else {
        0xFF
    }
}

#[must_use]
pub const fn cho_code(pyogi: u8) -> u8 {
    match pyogi {
        b'g' => 2,
        b'q' => 3,
        b'n' => 4,
        b'd' => 5,
        b'f' => 6,
        b'l' => 7,
        b'm' => 8,
        b'b' => 9,
        b'r' => 10,
        b's' => 11,
        b'v' => 12,
        b'j' => 14,
        b'z' => 15,
        b'c' => 16,
        b'k' => 17,
        b't' => 18,
        b'p' => 19,
        b'h' => 20,
        _ => 0,
    }
}

#[must_use]
pub const fn jung_code(a: u8, b: Option<u8>) -> u8 {
    match (a, b) {
        (b'a', None) => 3,
        (b'8', None) => 4,
        (b'e', None) => 7,
        (b'9', None) => 10,
        (b'o', None) => 13,
        (b'u', None) => 20,
        (b'_', None) => 27,
        (b'i', None) => 29,
        (b'y', Some(b'a')) => 5,
        (b'y', Some(b'8')) => 6,
        (b'y', Some(b'e')) => 11,
        (b'y', Some(b'9')) => 12,
        (b'y', Some(b'o')) => 19,
        (b'y', Some(b'u')) => 26,
        (b'w', Some(b'a')) => 14,
        (b'w', Some(b'8')) => 15,
        (b'w', Some(b'e')) => 21,
        (b'w', Some(b'9')) => 22,
        (b'w', Some(b'u')) => 23,
        (b'w', Some(b'i')) => 18,
        _ => 0,
    }
}

#[must_use]
pub const fn jong_code(a: u8, b: Option<u8>) -> u8 {
    match (a, b) {
        (b'G', None) => 2,
        (b'Q', None) => 3,
        (b'N', None) => 5,
        (b'D', None) => 8,
        (b'L', None) => 9,
        (b'M', None) => 17,
        (b'B', None) => 19,
        (b'S', None) => 21,
        (b'V', None) => 22,
        (b'*', None) => 23,
        (b'J', None) => 24,
        (b'C', None) => 25,
        (b'K', None) => 26,
        (b'T', None) => 27,
        (b'P', None) => 28,
        (b'H', None) => 29,
        (b'G', Some(b'S')) => 4,
        (b'N', Some(b'J')) => 6,
        (b'N', Some(b'H')) => 7,
        (b'L', Some(b'G')) => 10,
        (b'L', Some(b'M')) => 11,
        (b'L', Some(b'B')) => 12,
        (b'L', Some(b'S')) => 13,
        (b'L', Some(b'T')) => 14,
        (b'L', Some(b'P')) => 15,
        (b'L', Some(b'H')) => 16,
        (b'B', Some(b'S')) => 20,
        _ => 0,
    }
}

pub const CHO_NO: [u8; 32] = [
    0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0,
];

pub const JUNG_NO: [u8; 32] = [
    0, 0, 0, 1, 2, 3, 4, 5, 0, 0, 6, 7, 8, 9, 10, 11, 0, 0, 12, 13, 14, 15, 16, 17, 0, 0, 18, 19,
    20, 21, 0, 0,
];

pub const JONG_NO: [u8; 32] = [
    0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 0, 17, 18, 19, 20, 21, 22, 23, 24,
    25, 26, 27, 0, 0,
];

pub const SUN_JUNG_ID: [u8; 21] = [
    3, 4, 5, 6, 7, 10, 11, 12, 13, 14, 15, 18, 19, 20, 21, 22, 23, 26, 27, 28, 29,
];

pub const SUN_JONG_ID: [u8; 28] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20, 21, 22, 23, 24, 25, 26, 27,
    28, 29,
];

#[must_use]
pub fn is_uni_korean_jamo(w: u16) -> bool {
    (0x3131..=0x3163).contains(&w)
}

#[must_use]
pub fn is_uni_wansong(w: u16) -> bool {
    (0xAC00..=0xD7A3).contains(&w)
}

#[must_use]
#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
pub fn conv_uni_wan_to_cvc(w: u16) -> [u8; 3] {
    let t = u32::from(w.wrapping_sub(0xAC00));
    let cho = (t / 0x24C) as u8 + 2;
    let jung = SUN_JUNG_ID[((t % 0x24C) / 0x1C) as usize];
    let jong = SUN_JONG_ID[(t % 0x1C) as usize];
    [cho, jung, jong]
}

#[must_use]
#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
pub fn conv_cvc_to_uni_wan(cvc: &[u8]) -> u16 {
    let cho = cvc[0] as usize;
    let jung = cvc[1] as usize;
    let jong = cvc[2] as usize;
    if cho < 32 && jung < 32 && jong < 32 {
        let w = (u16::from(CHO_NO[cho]).wrapping_sub(1))
            .wrapping_mul(0x24C)
            .wrapping_add((u16::from(JUNG_NO[jung]).wrapping_sub(1)).wrapping_mul(0x1C))
            .wrapping_add(u16::from(JONG_NO[jong]))
            .wrapping_sub(0x5400);
        if cho == 1 || jung == 1 {
            for (k, &j) in GSW_KS_JOHAB_TBL.iter().enumerate() {
                let c = ((j >> 10) & 0x1F) as u8;
                let u = ((j >> 5) & 0x1F) as u8;
                let o = (j & 0x1F) as u8;
                if c == cvc[0] && u == cvc[1] && o == cvc[2] {
                    return 0x3131 + k as u16;
                }
            }
        }
        w
    } else {
        0
    }
}

#[must_use]
pub fn conv_uni_wan_to_cvc_special_pre(chars: &[u16]) -> (Vec<u8>, Vec<u8>) {
    let mut cvc = Vec::new();
    let mut ty = Vec::new();
    for &w in chars {
        if is_uni_korean_jamo(w) {
            let j = GSW_KS_JOHAB_TBL[(w - 0x3131) as usize];
            cvc.push(((j >> 10) & 0x1F) as u8);
            cvc.push(((j >> 5) & 0x1F) as u8);
            cvc.push((j & 0x1F) as u8);
            ty.extend_from_slice(b"HHH");
        } else if is_uni_wansong(w) {
            let [a, b, c] = conv_uni_wan_to_cvc(w);
            cvc.extend_from_slice(&[a, b, c]);
            ty.extend_from_slice(b"HHH");
        } else {
            cvc.push((w & 0xFF) as u8);
            ty.push(b'X');
        }
    }
    (cvc, ty)
}

#[must_use]
pub fn conv_cvc_to_uni_wan_special(cvc: &[u8], ty: &[u8]) -> Vec<u16> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < cvc.len() {
        let t = ty.get(i).copied().unwrap_or(b'X');
        if (t == b'H' || t == b'U') && i + 2 < cvc.len() {
            out.push(conv_cvc_to_uni_wan(&cvc[i..i + 3]));
            i += 3;
            continue;
        }
        out.push(u16::from(cvc[i]));
        i += 1;
    }
    out
}

#[must_use]
#[expect(
    clippy::many_single_char_names,
    reason = "C port: original single-letter variable names"
)]
pub fn conv_pyogi_to_cvc(pyogi: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    let n = pyogi.len();
    while i < n {
        let c = pyogi[i];
        if cvc_type_from_pyogi(c) == 0xFF {
            i += 1;
            continue;
        }
        let cho = if cvc_type_from_pyogi(c) == 0 {
            let code = cho_code(c);
            i += 1;
            code
        } else {
            13
        };
        let j = if i < n && cvc_type_from_pyogi(pyogi[i]) == 1 {
            let g = pyogi[i];
            if i + 1 < n && cvc_type_from_pyogi(pyogi[i + 1]) == 2 {
                let code = jung_code(g, Some(pyogi[i + 1]));
                i += 2;
                code
            } else {
                i += 1;
                0
            }
        } else if i < n && cvc_type_from_pyogi(pyogi[i]) == 2 {
            let code = jung_code(pyogi[i], None);
            i += 1;
            code
        } else {
            0
        };
        let mut jong = 1;
        if i < n && cvc_type_from_pyogi(pyogi[i]) == 3 {
            let a = pyogi[i];
            if i + 1 < n && cvc_type_from_pyogi(pyogi[i + 1]) == 3 {
                let code = jong_code(a, Some(pyogi[i + 1]));
                if code != 0 {
                    jong = code;
                    i += 2;
                } else {
                    jong = jong_code(a, None);
                    i += 1;
                }
            } else {
                jong = jong_code(a, None);
                i += 1;
            }
        }
        out.extend_from_slice(&[cho, j, jong]);
    }
    out
}

#[must_use]
#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
pub fn conv_cvc_to_pyogi(cvc: &[u8]) -> String {
    let mut s = String::new();
    let mut i = 0;
    while i + 2 < cvc.len() {
        let (cho, jung, jong) = (cvc[i], cvc[i + 1], cvc[i + 2]);
        if cho > 1 && (cho as usize) < CHO_PYOGI.len() {
            s.push_str(CHO_PYOGI[cho as usize]);
        }
        if (jung as usize) < JUNG_PYOGI.len() {
            s.push_str(JUNG_PYOGI[jung as usize]);
        }
        if jong > 1 && (jong as usize) < JONG_PYOGI.len() {
            s.push_str(JONG_PYOGI[jong as usize]);
        }
        i += 3;
    }
    s
}

pub use crate::tables::GSW_KS_JOHAB_TBL;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pyogi_roundtrip_samples() {
        for (pyogi, hangul) in [
            ("baNjeL", "반절"),
            ("d_LmeGd_LmeG", "들먹들먹"),
            ("useNju", "우선주"),
            ("g8yodo", "개요도"),
            ("daLnai", "달나이"),
            ("jagisu", "자기수"),
            ("s8*bye*siN", "생병신"),
            ("daMvoGdaMvoG", "담쏙담쏙"),
            ("solig8", "소리개"),
            ("myeNjuL", "면줄"),
            ("mojaG", "모작"),
            ("d8naC", "대낯"),
            ("gwuiLjeM", "귀일점"),
            ("laseNsa*", "라선상"),
            ("j9ga*so", "제강소"),
            ("hye*gwa*c9", "형광체"),
            ("jupa", "주파"),
            ("sil8gi", "시래기"),
        ] {
            let cvc = conv_pyogi_to_cvc(pyogi.as_bytes());
            assert_eq!(cvc.len() % 3, 0, "{pyogi}");
            let back = conv_cvc_to_pyogi(&cvc);
            assert_eq!(back, pyogi, "{pyogi}");
            let chars: Vec<u16> = hangul_to_u16(hangul);
            let (cvc2, ty) = conv_uni_wan_to_cvc_special_pre(&chars);
            assert_eq!(cvc2, cvc, "{pyogi}");
            assert!(ty.iter().all(|&t| t == b'H'));
            let wansong = conv_cvc_to_uni_wan_special(&cvc, &ty);
            assert_eq!(wansong, chars, "{pyogi}");
        }
    }

    fn hangul_to_u16(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn conv_pyogi_to_cvc_skips_unknown_chars() {
        let cvc = conv_pyogi_to_cvc(b"a\x01b");
        assert_eq!(cvc, [13, 3, 1, 9, 0, 1]);
        let cvc2 = conv_pyogi_to_cvc(&[0x01, 0x02, 0x03]);
        assert!(cvc2.is_empty());
    }

    #[test]
    fn conv_uni_wan_known_values() {
        assert_eq!(conv_uni_wan_to_cvc(0xAC00), [2, 3, 1]);
        assert_eq!(conv_uni_wan_to_cvc(0xBC18), [9, 3, 5]);
        assert_eq!(conv_uni_wan_to_cvc(0xC808), [14, 7, 9]);
        assert_eq!(conv_cvc_to_uni_wan(&[9, 3, 5]), 0xBC18);
        assert_eq!(conv_cvc_to_uni_wan(&[14, 7, 9]), 0xC808);
        assert_eq!(conv_cvc_to_uni_wan(&[2, 3, 1]), 0xAC00);
    }

    #[test]
    fn jamo_branch() {
        let j = GSW_KS_JOHAB_TBL[0];
        assert_eq!((j >> 10) & 0x1F, 1);
        assert_eq!((j >> 5) & 0x1F, 1);
        assert_eq!(j & 0x1F, 2);
        let (cvc, ty) = conv_uni_wan_to_cvc_special_pre(&[0x3131]);
        assert_eq!(cvc, [1, 1, 2]);
        assert_eq!(&ty, b"HHH");
        let (cvc, _) = conv_uni_wan_to_cvc_special_pre(&[0x3137]);
        assert_eq!(cvc, [1, 1, 8]);
    }
}
