pub(crate) const ONE_DIGIT_KOR: [&str; 10] = [
    "go*", "iL", "i", "saM", "sa", "o", "yuG", "ciL", "paL", "gu",
];

const ONE_DIGIT1_KOR: [&str; 10] = ["go*", "iL", "i", "saM", "sa", "o", "yu", "ciL", "paL", "gu"];

const DIGIT_LOW_UNIT: [&str; 4] = ["", "siB", "b8G", "ceN"];

const DIGIT1_LOW_UNIT: [&str; 4] = ["", "si", "b8G", "ceN"];

const DIGIT_HIGH_UNIT: [&str; 8] = ["", "maN", "eG", "jo", "gye*", "h8", "se", "ya*"];

const TEN_DIGIT_KOR: [&str; 10] = [
    "", "yeL", "s_muL", "sel_N", "mah_N", "swuN", "y9suN", "iLh_N", "yed_N", "ah_N",
];

const TEN_DIGIT1_KOR: [&str; 10] = [
    "", "haN", "du", "s9", "n9", "daseS", "yeseS", "iLgoB", "yedeLB", "ahoB",
];

const ENG_KOR: [&str; 26] = [
    "9i", "bi", "vi", "di", "i", "9p_", "jwu", "9cwu", "ai", "j9i", "k9i", "9L", "9M", "9N", "o",
    "pi", "kyu", "al_", "9v_", "ti", "yu", "b_i", "deb_Llyu", "9Gs_", "wai", "j9t_",
];

#[allow(clippy::too_many_arguments)]
fn digit_to_pron(
    pch_digit: &[u8],
    n: usize,
    n_rest_len: usize,
    n_cmp: usize,
    out: &mut String,
    f_flag: bool,
    one_tbl: &[&str; 10],
    low_tbl: &[&str; 4],
) {
    let c = pch_digit[n];
    if c == b'0' {
        if n + 1 < pch_digit.len() && pch_digit[n + 1] == b'0' {
            return;
        }
        if !f_flag {
            return;
        }
        if n_rest_len.trailing_zeros() >= 2 {
            return;
        }
        out.push_str("go* ");
        return;
    }
    let mut u_var3: usize = 0;
    let mut skip_digit = false;
    if n_rest_len == n_cmp || (n_rest_len & 3) != 0 {
        u_var3 = n_rest_len & 3;
        if n_rest_len == 0 {
            u_var3 = 0;
        } else if c == b'1' {
            skip_digit = true;
        }
    }
    if !skip_digit {
        out.push_str(one_tbl[(c - b'0') as usize]);
    }
    if u_var3 == 0 {
        let i_var4 = n_rest_len >> 2;
        out.push_str(DIGIT_HIGH_UNIT[i_var4]);
        if i_var4 != 0 {
            out.push('\t');
        }
    } else {
        out.push_str(low_tbl[n_rest_len & 3]);
        if 1 < (n_rest_len & 3) {
            out.push(' ');
        }
    }
}

pub fn digit_to_china_pron(pch_str: &[u8], out: &mut String) {
    digit_to_china_pron_tbl(pch_str, out, &ONE_DIGIT_KOR, &DIGIT_LOW_UNIT);
}

pub fn digit_to_china_pron_tbl(
    pch_str: &[u8],
    out: &mut String,
    one_tbl: &[&str; 10],
    low_tbl: &[&str; 4],
) {
    out.clear();
    let len = pch_str.len();
    if len == 1 {
        if pch_str[0] == b'0' {
            out.push_str("lye*");
            return;
        }
        digit_to_pron(pch_str, 0, 0, 0, out, true, one_tbl, low_tbl);
    } else {
        let n_rest_len = len.saturating_sub(1);
        let f_flag = len != 4;
        let mut n = 0usize;
        while n < len {
            digit_to_pron(
                pch_str,
                n,
                n_rest_len - n,
                n_rest_len,
                out,
                f_flag,
                one_tbl,
                low_tbl,
            );
            n += 1;
        }
    }
    if out.is_empty() {
        out.push_str("lye*");
    }
}

pub fn digit_to_korean_pron(pch_digit: &[u8], out: &mut String) {
    out.clear();
    let len = pch_digit.len();
    if len == 1 {
        if pch_digit[0] == b'0' {
            out.push_str("lye*");
        } else {
            korean_pron_loop(pch_digit, 0, out);
        }
    } else if len > 0 {
        korean_pron_loop(pch_digit, len - 1, out);
    }
    if out.is_empty() {
        out.push_str("lye*");
    }
}

fn korean_pron_loop(pch_digit: &[u8], local_28: usize, out: &mut String) {
    let len = pch_digit.len();
    let mut n = 0usize;
    let mut n_rest_len = local_28;
    loop {
        if n + 2 < len {
            digit_to_pron(
                pch_digit,
                n,
                n_rest_len,
                local_28,
                out,
                true,
                &ONE_DIGIT_KOR,
                &DIGIT_LOW_UNIT,
            );
        } else if n_rest_len == 1 {
            let c = pch_digit[n];
            if c == b'2' {
                if pch_digit[n + 1] == b'0' {
                    out.push_str("s_mu");
                } else {
                    out.push_str(TEN_DIGIT_KOR[2]);
                }
            } else if c == b'0' && pch_digit[n + 1] != b'0' {
                out.push_str("go* ");
            } else {
                out.push_str(TEN_DIGIT_KOR[(c - b'0') as usize]);
            }
        } else if n_rest_len == 0 {
            out.push_str(TEN_DIGIT1_KOR[(pch_digit[n] - b'0') as usize]);
        }
        if len <= n + 1 {
            break;
        }
        n += 1;
        n_rest_len = n_rest_len.saturating_sub(1);
    }
}

#[must_use]
pub fn digit_to_china_pron_value(n_digit: i32) -> String {
    let s = n_digit.to_string();
    let mut out = String::new();
    digit_to_china_pron(s.as_bytes(), &mut out);
    out
}

#[must_use]
pub fn digit_to_china_pron_value_special(n_digit: i32) -> String {
    let s = n_digit.to_string();
    let mut out = String::new();
    digit_to_china_pron_tbl(s.as_bytes(), &mut out, &ONE_DIGIT1_KOR, &DIGIT1_LOW_UNIT);
    out
}

#[must_use]
pub fn digit_to_korean_pron_value(n_digit: i32) -> String {
    let s = n_digit.to_string();
    let mut out = String::new();
    digit_to_korean_pron(s.as_bytes(), &mut out);
    out
}

#[must_use]
pub fn digit_to_one_pron_value(n_digit: i32) -> String {
    let s = n_digit.to_string();
    let mut out = String::new();
    for c in s.bytes() {
        out.push_str(ONE_DIGIT_KOR[(c - b'0') as usize]);
    }
    out
}

#[must_use]
pub fn digit_zero_remove(pw_str: &[u16]) -> Vec<u16> {
    let mut i = 0usize;
    while i < pw_str.len() && pw_str[i] == u16::from(b'0') {
        i += 1;
    }
    if i == 0 {
        pw_str.to_vec()
    } else {
        pw_str[i..].to_vec()
    }
}

#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
pub fn digit_to_china_pron_value_large(pw_digit: &[u16]) -> String {
    let z = digit_zero_remove(pw_digit);
    let s: Vec<u8> = z.iter().map(|&c| c as u8).collect();
    let mut out = String::new();
    digit_to_china_pron(&s, &mut out);
    out
}

#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
pub fn digit_to_one_pron_value_large(pw_digit: &[u16]) -> String {
    let mut out = String::new();
    for &c in pw_digit {
        if (c as u8).is_ascii_digit() {
            out.push_str(ONE_DIGIT_KOR[(c as u8 - b'0') as usize]);
        }
    }
    out
}

#[must_use]
pub fn decimal_read(n_digit1: i32, n_digit2: i32, f_mode: bool) -> String {
    let mut out = digit_to_china_pron_value(n_digit1);
    if f_mode {
        out.push_str(&digit_to_china_pron_value(n_digit2));
    } else {
        out.push_str(&digit_to_one_pron_value(n_digit2));
    }
    out
}

#[must_use]
pub fn decimal_read_large(pw_digit1: &[u16], pw_digit2: &[u16]) -> String {
    let mut out = digit_to_china_pron_value_large(pw_digit1);
    out.push_str(" zeM ");
    out.push_str(&digit_to_one_pron_value_large(pw_digit2));
    out
}

#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
pub fn digit_to_16_pron(pw_str: &[u16]) -> String {
    let mut out = String::new();
    for &c in pw_str {
        if (c as u8).is_ascii_digit() {
            out.push_str(ONE_DIGIT_KOR[(c as u8 - b'0') as usize]);
        } else if (c as u8).is_ascii_uppercase() {
            out.push_str(ENG_KOR[(c as u8 - b'A') as usize]);
        } else if (c as u8).is_ascii_lowercase() {
            out.push_str(ENG_KOR[(c as u8 - b'a') as usize]);
        }
    }
    out
}

#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
pub fn wtoi(pw: &[u16]) -> i32 {
    let mut v: i64 = 0;
    for &c in pw {
        if (c as u8).is_ascii_digit() {
            v = v * 10 + i64::from(c as u8 - b'0');
            if v > i64::from(i32::MAX) {
                v = i64::from(i32::MAX);
            }
        } else {
            break;
        }
    }
    v as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn china_small() {
        let mut o = String::new();
        digit_to_china_pron(b"95", &mut o);
        assert_eq!(o, "gusiBo");
        let mut o = String::new();
        digit_to_china_pron(b"17", &mut o);
        assert_eq!(o, "siBciL");
        let mut o = String::new();
        digit_to_china_pron(b"1", &mut o);
        assert_eq!(o, "iL");
        let mut o = String::new();
        digit_to_china_pron(b"10", &mut o);
        assert_eq!(o, "siB");
        let mut o = String::new();
        digit_to_china_pron(b"0", &mut o);
        assert_eq!(o, "lye*");
        let mut o = String::new();
        digit_to_china_pron(b"101", &mut o);
        assert_eq!(o, "b8G go* iL");
        let mut o = String::new();
        digit_to_china_pron(b"1005", &mut o);
        assert_eq!(o, "ceN o");
    }

    #[test]
    fn china_large() {
        let mut o = String::new();
        digit_to_china_pron(b"1995", &mut o);
        assert_eq!(o, "ceN gub8G gusiBo");
        let mut o = String::new();
        digit_to_china_pron(b"123456789", &mut o);
        assert_eq!(o, "eG\ticeN saMb8G sasiBomaN\tyuGceN ciLb8G paLsiBgu");
        let mut o = String::new();
        digit_to_china_pron(b"1200", &mut o);
        assert_eq!(o, "ceN ib8G ");
    }

    #[test]
    fn korean_native() {
        let mut o = String::new();
        digit_to_korean_pron(b"3", &mut o);
        assert_eq!(o, "s9");
        let mut o = String::new();
        digit_to_korean_pron(b"12", &mut o);
        assert_eq!(o, "yeLdu");
        let mut o = String::new();
        digit_to_korean_pron(b"20", &mut o);
        assert_eq!(o, "s_mu");
        let mut o = String::new();
        digit_to_korean_pron(b"25", &mut o);
        assert_eq!(o, "s_muLdaseS");
        let mut o = String::new();
        digit_to_korean_pron(b"0", &mut o);
        assert_eq!(o, "lye*");
    }

    #[test]
    fn special_month() {
        let mut o = String::new();
        digit_to_china_pron_tbl(b"10", &mut o, &ONE_DIGIT1_KOR, &DIGIT1_LOW_UNIT);
        assert_eq!(o, "si");
        let mut o = String::new();
        digit_to_china_pron_tbl(b"6", &mut o, &ONE_DIGIT1_KOR, &DIGIT1_LOW_UNIT);
        assert_eq!(o, "yu");
        let mut o = String::new();
        digit_to_china_pron_tbl(b"12", &mut o, &ONE_DIGIT1_KOR, &DIGIT1_LOW_UNIT);
        assert_eq!(o, "sii");
    }

    #[test]
    fn one_by_one() {
        assert_eq!(
            digit_to_one_pron_value_large(&[u16::from(b'0'), u16::from(b'1'), u16::from(b'0')]),
            "go*iLgo*"
        );
        assert_eq!(
            digit_to_one_pron_value_large(&[
                u16::from(b'1'),
                u16::from(b'2'),
                u16::from(b'3'),
                u16::from(b'4')
            ]),
            "iLisaMsa"
        );
        assert_eq!(
            digit_to_china_pron_value_large(&[u16::from(b'0'), u16::from(b'1'), u16::from(b'2')]),
            "siBi"
        );
    }

    #[test]
    fn decimal_and_16() {
        assert_eq!(
            decimal_read_large(&[u16::from(b'9'), u16::from(b'5')], &[u16::from(b'5')]),
            "gusiBo zeM o"
        );
        assert_eq!(
            digit_to_16_pron(&[u16::from(b'1'), u16::from(b'F')]),
            "iL9p_"
        );
        assert_eq!(
            digit_to_16_pron(&[
                u16::from(b'0'),
                u16::from(b'x'),
                u16::from(b'2'),
                u16::from(b'0')
            ]),
            "go*9Gs_igo*"
        );
    }
}
