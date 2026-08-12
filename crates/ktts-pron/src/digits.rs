use crate::dicts::PronContext;

const DIGIT_KOR: [&str; 10] = [
    "yoN", "iL", "i", "saM", "sa", "o", "yuG", "ciL", "paL", "gu",
];

#[must_use]
pub fn digits_telephone(digits: &[u8]) -> String {
    let mut s = String::new();
    for &d in digits {
        if d.is_ascii_digit() {
            s.push_str(DIGIT_KOR[(d - b'0') as usize]);
        }
    }
    s
}

const UNIT_SMALL: [&str; 4] = ["", "siB", "b8G", "ceN"];
const UNIT_LARGE: [&str; 5] = ["", "maN", "eG", "jo", "gyeong"];

#[must_use]
pub fn digits_cardinal(digits: &[u8]) -> String {
    let d: Vec<u8> = digits.iter().skip_while(|&&c| c == b'0').copied().collect();
    if d.is_empty() {
        return "yoN".to_string();
    }
    let n = d.len();
    let mut out = String::new();
    let first_group_len = ((n - 1) % 4) + 1;
    let mut pos = 0usize;
    let mut group = first_group_len;
    let mut group_idx = (n - 1) / 4;
    while pos < n {
        let mut wrote = false;
        for i in 0..group {
            let v = (d[pos + i] - b'0') as usize;
            if v == 0 {
                continue;
            }
            if v > 1 || (v == 1 && group_idx == 0 && group - 1 - i == 0) {
                out.push_str(DIGIT_KOR[v]);
            }
            out.push_str(UNIT_SMALL[group - 1 - i]);
            wrote = true;
        }
        if wrote && group_idx > 0 {
            out.push_str(UNIT_LARGE[group_idx]);
        }
        pos += group;
        group = 4;
        group_idx = group_idx.saturating_sub(1);
    }
    out
}

fn u16s_to_le_bytes(v: &[u16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 2);
    for &c in v {
        out.extend_from_slice(&c.to_le_bytes());
    }
    out
}

#[must_use]
/// Applies the user dictionary to a text.
///
/// # Panics
///
/// Panics if the dictionary data is inconsistent.
pub fn apply_user_dic(ctx: &PronContext, text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while !rest.is_empty() {
        let chars: Vec<u16> = rest.encode_utf16().collect();
        let bytes = u16s_to_le_bytes(&chars);
        if let Some((key_len_bytes, val)) = ctx.user_lookup(&bytes) {
            out.push_str(&val);
            let consumed = key_len_bytes / 2;
            let take: usize = chars[..consumed]
                .iter()
                .map(|&c| match c {
                    0..=0x7F => 1,
                    0x80..=0x7FF => 2,
                    _ => 3,
                })
                .sum();
            rest = &rest[take..];
        } else {
            let c = rest
                .chars()
                .next()
                .expect("loop body only runs while rest is non-empty");
            out.push(c);
            rest = &rest[c.len_utf8()..];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cardinal() {
        assert_eq!(digits_cardinal(b"12"), "siBi");
        assert_eq!(digits_cardinal(b"1984"), "ceNgub8GpaLsiBsa");
        assert_eq!(digits_cardinal(b"0"), "yoN");
        assert_eq!(digits_cardinal(b"100"), "b8G");
        assert_eq!(digits_cardinal(b"10000"), "maN");
        assert_eq!(digits_cardinal(b"120"), "b8GisiB");
    }

    #[test]
    fn telephone() {
        assert_eq!(digits_telephone(b"1984"), "iLgu paLsa".replace(' ', ""));
        assert_eq!(
            digits_telephone(b"1984"),
            "iLgu paLsa"
                .chars()
                .filter(|c| *c != ' ')
                .collect::<String>()
        );
    }
}
