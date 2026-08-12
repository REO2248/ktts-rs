use crate::tables::{
    CHO_NO, CHOSONG, JAMO_CVC, JONG_NO, JUNG_NO, SSCH_CHO, SSCH_JONG, SSCH_JUNG, UNI_JONG_ID,
    UNI_JUNG_ID, cvc_type_from_pyogi,
};

#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
pub fn conv_uni_wan_to_cvc(pw: &[u16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(pw.len() * 3);
    for &w in pw {
        let t = i32::from(w) - 0xac00;
        out.push((t / 0x24c + 2) as u8);
        out.push(UNI_JUNG_ID[((t % 0x24c) / 0x1c) as usize]);
        out.push(UNI_JONG_ID[(t % 0x1c) as usize]);
    }
    out
}

#[must_use]
#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
pub fn conv_uni_code_to_cvc(w: u16) -> [u8; 3] {
    if w == 0 {
        return [0, 0, 0];
    }
    let t = i32::from(w) - 0xac00;
    let r = t % 0x24c;
    [
        CHO_NO[(t / 0x24c + 2) as usize],
        JUNG_NO[UNI_JUNG_ID[(r / 0x1c) as usize] as usize],
        JONG_NO[UNI_JONG_ID[(r % 0x1c) as usize] as usize],
    ]
}

#[must_use]
pub fn conv_cvc_to_pyogi(cvc: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 2 < cvc.len() {
        let c = cvc[i];
        if CHO_ONE(c) != 0 {
            out.push(CHO_ONE(c));
        }
        let mut j = JUNG_ONE(cvc[i + 1]);
        if j != 0 {
            out.push(j);
            j = JUNG_TWO(cvc[i + 1]);
            if j != 0 {
                out.push(j);
            }
        }
        let mut k = JONG_ONE(cvc[i + 2]);
        if k != 0 {
            out.push(k);
            k = JONG_TWO(cvc[i + 2]);
            if k != 0 {
                out.push(k);
            }
        }
        i += 3;
    }
    out
}

#[allow(non_snake_case)]
fn CHO_ONE(c: u8) -> u8 {
    CHOSONG.get(c as usize).copied().unwrap_or(0)
}
#[allow(non_snake_case)]
fn JUNG_ONE(c: u8) -> u8 {
    JUNG_ONE_T.get(c as usize).copied().unwrap_or(0)
}
#[allow(non_snake_case)]
fn JUNG_TWO(c: u8) -> u8 {
    JUNG_TWO_T.get(c as usize).copied().unwrap_or(0)
}
#[allow(non_snake_case)]
fn JONG_ONE(c: u8) -> u8 {
    JONG_ONE_T.get(c as usize).copied().unwrap_or(0)
}
#[allow(non_snake_case)]
fn JONG_TWO(c: u8) -> u8 {
    JONG_TWO_T.get(c as usize).copied().unwrap_or(0)
}
const JUNG_ONE_T: [u8; 32] = crate::tables::JUNG_ONE;
const JUNG_TWO_T: [u8; 32] = crate::tables::JUNG_TWO;
const JONG_ONE_T: [u8; 32] = crate::tables::JONG_ONE;
const JONG_TWO_T: [u8; 32] = crate::tables::JONG_TWO;

#[must_use]
pub fn conv_uni_wan_to_pyogi(pw: &[u16]) -> Vec<u8> {
    conv_cvc_to_pyogi(&conv_uni_wan_to_cvc(pw))
}

#[expect(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn search_str_from_str_list(list: &[&[u8]], n: usize, s: &[u8]) -> Option<usize> {
    let mut i = n as isize - 1;
    while i >= 0 {
        let e = list[i as usize];
        if !e.is_empty() && s.starts_with(e) {
            return Some(i as usize);
        }
        i -= 1;
    }
    None
}

fn get_cvc_from_pyogi_id(ids: &[u8], types: &[u8], n: usize) -> [u8; 3] {
    let mut cvc = [0u8; 3];
    if n == 2 {
        if types[0] == 0 {
            if types[1] == 2 {
                cvc[0] = ids[0];
                cvc[1] = ids[1];
                cvc[2] = 1;
                return cvc;
            }
        } else if types[0] == 2 && types[1] == 3 {
            cvc[0] = 13;
            cvc[1] = ids[0];
            cvc[2] = ids[1];
            return cvc;
        }
    } else if n == 3 {
        cvc[0] = ids[0];
        cvc[1] = ids[1];
        cvc[2] = ids[2];
        return cvc;
    } else if n == 1 {
        match types[0] {
            0 => {
                cvc[0] = ids[0];
                cvc[1] = 1;
                cvc[2] = 1;
            }
            2 => {
                cvc[0] = 13;
                cvc[1] = ids[0];
                cvc[2] = 1;
            }
            3 => {
                cvc[0] = 1;
                cvc[1] = 1;
                cvc[2] = ids[0];
            }
            _ => return [0, 0, 0],
        }
        return cvc;
    }
    [0, 0, 0]
}

#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
pub fn conv_pyogi_to_cvc(pyogi: &[u8]) -> Vec<u8> {
    if pyogi.is_empty() {
        return Vec::new();
    }
    let n = pyogi.len();
    let mut ptype = vec![0u8; n];
    for i in 0..n {
        ptype[i] = cvc_type_from_pyogi(pyogi[i]);
    }
    let mut ids = vec![0u8; n];
    let mut n_id = 0usize;
    let mut i = 0usize;
    'outer: while i < n {
        while i < n && ptype[i] != 0 {
            if (ptype[i] - 1) < 2 {
                if let Some(idx) = search_str_from_str_list(&SSCH_JUNG, 0x20, &pyogi[i..]) {
                    ptype[n_id] = 2;
                    ids[n_id] = idx as u8;
                    n_id += 1;
                    i += if SSCH_JUNG[idx].len() == 2 { 2 } else { 1 };
                } else {
                    break 'outer;
                }
            } else if ptype[i] == 3 {
                if let Some(idx) = search_str_from_str_list(&SSCH_JONG, 0x20, &pyogi[i..]) {
                    ptype[n_id] = 3;
                    ids[n_id] = idx as u8;
                    n_id += 1;
                    i += if SSCH_JONG[idx].len() == 2 { 2 } else { 1 };
                } else {
                    break 'outer;
                }
            } else {
                i += 1;
            }
        }
        if i >= n {
            break;
        }
        if let Some(idx) = search_str_from_str_list(&SSCH_CHO, 0x20, &pyogi[i..]) {
            i += 1;
            ptype[n_id] = 0;
            ids[n_id] = idx as u8;
            n_id += 1;
        } else {
            break 'outer;
        }
    }
    let mut out = Vec::new();
    if n_id != 0 {
        let mut j = 0usize;
        let mut i = 1usize;
        loop {
            let emit = if i < n_id {
                let prev_type = ptype[i - 1];
                let cur_type = ptype[i];
                !((prev_type == 0 && cur_type == 2) || (prev_type == 2 && cur_type == 3))
            } else {
                true
            };
            if emit {
                let cvc = get_cvc_from_pyogi_id(&ids[j..], &ptype[j..], i - j);
                out.extend_from_slice(&cvc);
                j = i;
            }
            if i >= n_id {
                break;
            }
            i += 1;
        }
    }
    out
}

#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
pub fn conv_cvc_to_uni_wan(cvc: &[u8]) -> Vec<u16> {
    let mut out = Vec::new();
    let n = cvc.len() / 3;
    for i in 0..n {
        let c = &cvc[i * 3..i * 3 + 3];
        let jamo = if c[0] == 1 {
            c[1] == 1 || c[2] == 1
        } else {
            c[1] == 1 && c[2] == 1
        };
        if jamo {
            let mut found = false;
            let mut k = 0usize;
            for (k2, jc) in JAMO_CVC.iter().enumerate() {
                if jc.0 == c[0] && jc.1 == c[1] && jc.2 == c[2] {
                    found = true;
                    k = k2;
                    break;
                }
            }
            if found {
                out.push(0x3131 + k as u16);
            } else {
                out.push(0);
            }
        } else {
            let w = (i32::from(CHO_NO[c[0] as usize]) - 1) * 0x24c
                + (i32::from(JUNG_NO[c[1] as usize]) - 1) * 0x1c
                + i32::from(JONG_NO[c[2] as usize])
                - 0x5400;
            out.push(w as u16);
        }
    }
    out
}

#[must_use]
pub fn conv_pyogi_to_uni_wan(pyogi: &[u8]) -> Vec<u16> {
    let cvc = conv_pyogi_to_cvc(pyogi);
    conv_cvc_to_uni_wan(&cvc)
}

#[must_use]
pub fn get_kchar_count(s: &[u8]) -> usize {
    let mut n = 0;
    let mut i = 0;
    while i < s.len() {
        let c = s[i];
        if b"aeoui_89".contains(&c) {
            n += 1;
            i += 1;
        } else if b"yw".contains(&c) {
            n += 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annyeonghaseyo_romanization() {
        let w = [0xc548u16, 0xb155, 0xd558, 0xc138, 0xc694];
        let pyogi = conv_uni_wan_to_pyogi(&w);
        assert_eq!(pyogi, b"aNnye*has9yo");
        let cvc = conv_uni_wan_to_cvc(&w);
        assert_eq!(
            cvc,
            vec![13, 3, 5, 4, 11, 23, 20, 3, 1, 11, 10, 1, 13, 19, 1]
        );
        assert_eq!(conv_cvc_to_uni_wan(&cvc), w);
        assert_eq!(conv_pyogi_to_cvc(&pyogi), cvc);
        assert_eq!(conv_pyogi_to_cvc(b"aNnye*"), vec![13, 3, 5, 4, 11, 23]);
        assert_eq!(conv_pyogi_to_cvc(b"ha"), vec![20, 3, 1]);
        assert_eq!(conv_pyogi_to_cvc(b"s9yo"), vec![11, 10, 1, 13, 19, 1]);
    }

    #[test]
    fn kchar_count() {
        assert_eq!(get_kchar_count(b"aNnye*"), 2);
        assert_eq!(get_kchar_count(b"has9yo"), 3);
        assert_eq!(get_kchar_count(b"eyo"), 2);
    }
}
