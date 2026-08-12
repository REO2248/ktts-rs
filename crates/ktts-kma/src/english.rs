use crate::dict::EngDicts;
use crate::eng_tables::{
    CSWTCH_55, CSWTCH_58, CSWTCH_61, CSWTCH_64, GSCH_ASCII, GSCH_CARDINALS, GSCH_ENGI_KOR_TBL,
    GSCH_ORD_TWENTIES, GSCH_ORDINALS, GSCH_TWENTIES, RULES,
};

const SEP: u8 = b' ';

#[inline]
const fn is_alpha(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

#[inline]
const fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

#[inline]
const fn is_upper(c: u8) -> bool {
    c.is_ascii_uppercase()
}

#[inline]
const fn is_space(c: u8) -> bool {
    c == b'\t' || c == b' ' || c == b'\n' || c == b'\r'
}

#[inline]
const fn make_upper(c: u8) -> u8 {
    c.to_ascii_uppercase()
}

#[inline]
const fn is_vowel(c: u8) -> bool {
    matches!(c, b'E' | b'A' | b'O' | b'I' | b'U')
}

#[inline]
const fn is_consonant(c: u8) -> bool {
    is_upper(c) && !is_vowel(c)
}

#[inline]
const fn is_voiced_consonant(c: u8) -> bool {
    matches!(
        c,
        b'D' | b'B' | b'V' | b'G' | b'J' | b'L' | b'M' | b'N' | b'R' | b'W' | b'Z'
    )
}

#[derive(Default)]
struct Pyo {
    buf: Vec<u8>,
}

impl Pyo {
    fn push_str(&mut self, s: &[u8]) {
        self.buf.extend_from_slice(s);
    }
    fn push(&mut self, c: u8) {
        self.buf.push(c);
    }
    fn clear(&mut self) {
        self.buf.clear();
    }
}

fn say_ascii(pyo: &mut Pyo, ch: u8) {
    pyo.push_str(GSCH_ASCII[(ch & 0x7f) as usize].as_bytes());
    pyo.push(SEP);
}

fn spell_word(pyo: &mut Pyo, word: &[u8]) {
    let mut i = 1usize;
    while i + 1 < word.len() && word[i + 1] != 0 {
        say_ascii(pyo, word[i]);
        i += 1;
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn leftmatch(pat: &str, word: &[u8], mut ctx: i64) -> bool {
    let p = pat.as_bytes();
    let n = p.len();
    if n == 0 {
        return true;
    }
    let mut i = 0usize;
    loop {
        if ctx < 0 {
            return false;
        }
        let pc = p[n - 1 - i];
        if is_alpha(pc) || pc == b'\'' || pc == b' ' {
            if word[ctx as usize] != pc {
                return false;
            }
            ctx -= 1;
        } else if pc == b'.' {
            if !is_voiced_consonant(word[ctx as usize]) {
                return false;
            }
            ctx -= 1;
        } else if pc == b'+' {
            let c = word[ctx as usize];
            if c != b'I' && c != b'E' && c != b'Y' {
                return false;
            }
            ctx -= 1;
        } else if pc == b'#' {
            if !is_vowel(word[ctx as usize]) {
                return false;
            }
            loop {
                ctx -= 1;
                if ctx < 0 || !is_vowel(word[ctx as usize]) {
                    break;
                }
            }
        } else if pc == b':' {
            while ctx >= 0 && is_consonant(word[ctx as usize]) {
                ctx -= 1;
            }
        } else if pc == b'^' {
            if ctx < 0 || !is_consonant(word[ctx as usize]) {
                return false;
            }
            ctx -= 1;
        } else {
            return false;
        }
        i += 1;
        if i == n {
            return true;
        }
    }
}

fn rightmatch(pat: &str, word: &[u8], mut ctx: usize) -> bool {
    let p = pat.as_bytes();
    let n = p.len();
    let mut pi = 0usize;
    loop {
        if pi == n {
            return true;
        }
        let pc = p[pi];
        if is_alpha(pc) || pc == b'\'' || pc == b' ' {
            if ctx >= word.len() || word[ctx] != pc {
                return false;
            }
            ctx += 1;
        } else if pc == b'#' {
            if ctx >= word.len() || !is_vowel(word[ctx]) {
                return false;
            }
            loop {
                ctx += 1;
                if ctx >= word.len() || !is_vowel(word[ctx]) {
                    break;
                }
            }
        } else if pc == b'%' {
            if ctx < word.len() && word[ctx] == b'E' {
                let c2 = word.get(ctx + 1).copied().unwrap_or(0);
                if c2 == b'L' {
                    if word.get(ctx + 2).copied().unwrap_or(0) == b'Y' {
                        ctx += 3;
                    } else {
                        ctx += 1;
                    }
                } else if c2 == b'R' || c2 == b'S' || c2 == b'D' {
                    ctx += 2;
                } else {
                    ctx += 1;
                }
            } else if ctx + 2 < word.len()
                && word[ctx] == b'I'
                && word[ctx + 1] == b'N'
                && word[ctx + 2] == b'G'
            {
                ctx += 3;
            } else {
                return false;
            }
        } else if pc == b'+' {
            if ctx >= word.len() {
                return false;
            }
            let c = word[ctx];
            if c != b'I' && c != b'E' && c != b'Y' {
                return false;
            }
            ctx += 1;
        } else if pc == b'.' {
            if ctx >= word.len() || !is_voiced_consonant(word[ctx]) {
                return false;
            }
            ctx += 1;
        } else if pc == b':' {
            while ctx < word.len() && is_consonant(word[ctx]) {
                ctx += 1;
            }
        } else if pc == b'^' {
            if ctx >= word.len() || !is_consonant(word[ctx]) {
                return false;
            }
            ctx += 1;
        } else {
            return false;
        }
        pi += 1;
    }
}

#[expect(
    clippy::cast_possible_wrap,
    reason = "C port: index/math casts with wrap semantics"
)]
fn find_rule(
    pyo: &mut Pyo,
    word: &[u8],
    nindex: usize,
    rules: &[crate::eng_tables::EngRule],
) -> usize {
    for rule in rules {
        let m = rule.mat.as_bytes();
        if m.is_empty() {
            if leftmatch(rule.left, word, nindex as i64 - 1) && rightmatch(rule.right, word, nindex)
            {
                if rule.out.is_empty() {
                    return nindex;
                }
                pyo.push_str(rule.out.as_bytes());
                pyo.push(SEP);
                return nindex;
            }
            continue;
        }
        if nindex >= word.len() || word[nindex] != m[0] {
            continue;
        }
        let mut k = 1usize;
        while k < m.len() {
            if nindex + k >= word.len() || word[nindex + k] != m[k] {
                break;
            }
            k += 1;
        }
        if k != m.len() {
            continue;
        }
        let end = nindex + m.len();
        if leftmatch(rule.left, word, nindex as i64 - 1) && rightmatch(rule.right, word, end) {
            if rule.out.is_empty() {
                return end;
            }
            pyo.push_str(rule.out.as_bytes());
            pyo.push(SEP);
            return end;
        }
    }
    nindex + 1
}

fn xlate_word(pyo: &mut Pyo, word: &[u8]) {
    let mut nindex = 1usize;
    let mut c = word.get(nindex).copied().unwrap_or(0);
    while c != 0 {
        let idx = if is_upper(c) {
            (c - b'A' + 1) as usize
        } else {
            0
        };
        nindex = find_rule(pyo, word, nindex, RULES[idx]);
        c = word.get(nindex).copied().unwrap_or(0);
    }
}

fn say_cardinal(pyo: &mut Pyo, mut n: u32) {
    if n >= 1_000_000_000 {
        say_cardinal(pyo, n / 1_000_000_000);
        pyo.push_str(b"bIHlIYAXn ");
        n %= 1_000_000_000;
        if n == 0 {
            return;
        }
        if n < 100 {
            pyo.push_str(b"AEnd ");
        }
    }
    if n >= 1_000_000 {
        say_cardinal(pyo, n / 1_000_000);
        pyo.push_str(b"mIHlIYAXn ");
        n %= 1_000_000;
        if n == 0 {
            return;
        }
        if n < 100 {
            pyo.push_str(b"AEnd ");
        }
    }
    if (1000..=1099).contains(&n) || n >= 2000 {
        say_cardinal(pyo, n / 1000);
        pyo.push_str(b"THAWzAEnd ");
        n %= 1000;
        if n == 0 {
            return;
        }
        if n < 100 {
            pyo.push_str(b"AEnd ");
        }
    }
    if n >= 100 {
        pyo.push_str(GSCH_CARDINALS[(n / 100) as usize].as_bytes());
        pyo.push_str(b"hAHndrEHd ");
        n %= 100;
        if n == 0 {
            return;
        }
    }
    if n > 19 {
        pyo.push_str(GSCH_TWENTIES[((n - 20) / 10) as usize].as_bytes());
        n %= 10;
        if n == 0 {
            return;
        }
    }
    pyo.push_str(GSCH_CARDINALS[n as usize].as_bytes());
}

fn say_ordinal(pyo: &mut Pyo, mut n: u32) {
    if n >= 1_000_000_000 {
        say_cardinal(pyo, n / 1_000_000_000);
        n %= 1_000_000_000;
        if n == 0 {
            pyo.push_str(b"bIHlIYAXnTH ");
            return;
        }
        pyo.push_str(b"bIHlIYAXn ");
        if n < 100 {
            pyo.push_str(b"AEnd ");
        }
    }
    if n >= 1_000_000 {
        say_cardinal(pyo, n / 1_000_000);
        n %= 1_000_000;
        if n == 0 {
            pyo.push_str(b"mIHlIYAXnTH ");
            return;
        }
        pyo.push_str(b"mIHlIYAXn ");
        if n < 100 {
            pyo.push_str(b"AEnd ");
        }
    }
    if (1000..=1099).contains(&n) || n >= 2000 {
        say_cardinal(pyo, n / 1000);
        n %= 1000;
        if n == 0 {
            pyo.push_str(b"THAWzAEndTH ");
            return;
        }
        pyo.push_str(b"THAWzAEnd ");
        if n < 100 {
            pyo.push_str(b"AEnd ");
        }
    }
    if n >= 100 {
        pyo.push_str(GSCH_CARDINALS[(n / 100) as usize].as_bytes());
        n %= 100;
        if n == 0 {
            pyo.push_str(b"hAHndrEHdTH ");
            return;
        }
        pyo.push_str(b"hAHndrEHd ");
    }
    if n > 19 {
        let tens = ((n - 20) / 10) as usize;
        n %= 10;
        if n == 0 {
            pyo.push_str(GSCH_ORD_TWENTIES[tens].as_bytes());
            return;
        }
        pyo.push_str(GSCH_TWENTIES[tens].as_bytes());
    }
    pyo.push_str(GSCH_ORDINALS[n as usize].as_bytes());
}

#[allow(unused_assignments)]
fn english_number(pyo: &mut Pyo, buf: &[u8], pn0: usize) -> usize {
    let mut pn = pn0;
    let first = buf[pn];
    pn += 1;
    let mut value: u32 = u32::from(first - b'0');
    let mut ch = buf[pn];
    let mut last_digit: i32 = i32::from(first - b'0');
    let mut prev_digit_char = first;
    if ch != b'\'' {
        loop {
            last_digit = i32::from(ch);
            if !is_digit(ch) {
                last_digit = i32::from(prev_digit_char - b'0');
                break;
            }
            value = value * 10 + u32::from(ch - b'0');
            pn += 1;
            prev_digit_char = ch;
            ch = buf[pn];
            if ch == b'\'' {
                last_digit = i32::from(ch) - 0x30;
                break;
            }
        }
        if ch == b'\'' {
            last_digit = i32::from(ch) - 0x30;
        }
    }
    let mut ordinal = false;
    match last_digit {
        0 | 2 | 4 | 5 | 6 | 7 | 8 | 9 => {
            if make_upper(ch) == b'N' {
                ordinal = true;
            }
        }
        1 => {
            if make_upper(ch) == b'S' && make_upper(buf[pn + 1]) == b'T' {
                ordinal = true;
            }
        }
        3 if make_upper(ch) == b'R' => {
            ordinal = true;
        }
        _ => {}
    }
    if ordinal {
        let d_ok = last_digit == 1 || make_upper(buf[pn + 1]) == b'D';
        if d_ok {
            let c2 = buf[pn + 2];
            if !is_alpha(c2) && !is_digit(c2) {
                say_ordinal(pyo, value);
                pn += 1;
                return pn;
            }
        }
    }
    say_cardinal(pyo, value);
    if ch == b'.' && is_digit(buf[pn + 1]) {
        pyo.push_str(b"pOYnt ");
        pn += 1;
        ch = buf[pn];
        while ch != b'\'' {
            if !is_digit(ch) {
                break;
            }
            say_ascii(pyo, ch);
            pn += 1;
            ch = buf[pn];
        }
    }
    if is_alpha(ch) {
        while is_alpha(ch) {
            say_ascii(pyo, ch);
            pn += 1;
            ch = buf[pn];
        }
    }
    pn.wrapping_sub(1)
}

fn english_dollars(pyo: &mut Pyo, buf: &[u8], pn0: &mut usize) {
    let mut pn = *pn0;
    let mut n: u32 = 0;
    let old = pn;
    pn += 1;
    let mut c = buf[old + 1];
    if c == b'\'' {
        n = 0;
    } else {
        loop {
            if !is_digit(c) {
                say_cardinal(pyo, n);
                if c == b'.' && is_digit(buf[pn + 1]) {
                    pn += 1;
                    c = buf[pn];
                    let d2 = is_digit(buf[pn + 1]);
                    let d3 = is_digit(buf[pn + 2]);
                    if !d2 || d3 {
                        pyo.push_str(b"pOYnt ");
                        while c != b'\'' && is_digit(c) {
                            say_ascii(pyo, c);
                            pn += 1;
                            c = buf[pn];
                        }
                        pyo.push_str(b"dAAlAArz ");
                        pn = pn.wrapping_sub(1);
                        *pn0 = pn;
                        return;
                    }
                    if n == 1 {
                        pyo.push_str(b"dAAlER ");
                    } else {
                        pyo.push_str(b"dAAlAArz ");
                    }
                    if c == b'0' && buf[pn + 1] == b'0' {
                        pn += 1;
                        *pn0 = pn;
                        return;
                    }
                    pyo.push_str(b"AAnd ");
                    let cents = u32::from(c) * 10 + u32::from(buf[pn + 1]) - 0x210;
                    say_cardinal(pyo, cents);
                    if cents == 1 {
                        pyo.push_str(b"sEHnt ");
                    } else {
                        pyo.push_str(b"sEHnts ");
                    }
                    pn += 1;
                    *pn0 = pn;
                    return;
                }
                if n == 1 {
                    pyo.push_str(b"dAAlER ");
                } else {
                    pyo.push_str(b"dAAlAArz ");
                }
                pn = pn.wrapping_sub(1);
                *pn0 = pn;
                return;
            }
            if c == b',' {
                pn = pn.wrapping_sub(1);
                *pn0 = pn;
                return;
            }
            n = n * 10 + u32::from(c - b'0');
            pn += 1;
            c = buf[pn];
            if c == b'\'' {
                break;
            }
        }
    }
    say_cardinal(pyo, n);
    if n == 1 {
        pyo.push_str(b"dAAlER ");
    } else {
        pyo.push_str(b"dAAlAArz ");
    }
    pn = pn.wrapping_sub(1);
    *pn0 = pn;
}

fn english_special(pyo: &mut Pyo, c: u8) {
    if c == b'\n' {
        pyo.clear();
        pyo.push(b' ');
        return;
    }
    if is_space(c) {
        return;
    }
    say_ascii(pyo, c);
}

fn abbrev(pyo: &mut Pyo, word: &[u8], pn: &mut usize) {
    let eq = |s: &[u8]| word.len() >= s.len() && &word[..s.len()] == s;
    if eq(b" DR ") {
        xlate_word(pyo, b" DOCTOR ");
        *pn += 1;
    } else if eq(b" MR ") {
        xlate_word(pyo, b" MISTER ");
        *pn += 1;
    } else if eq(b" MRS ") {
        xlate_word(pyo, b" MISSUS ");
        *pn += 1;
    } else if eq(b" PHD ") {
        spell_word(pyo, b" PHD ");
        *pn += 1;
    } else {
        xlate_word(pyo, word);
    }
}

fn english_letter(pyo: &mut Pyo, buf: &[u8], pn0: usize) -> usize {
    let mut pn = pn0;
    let mut sch = [0u8; 104];
    sch[0] = b' ';
    let mut i_var9: usize = 2;
    let c0 = buf[pn];
    pn += 1;
    sch[1] = make_upper(c0);
    let mut ch_byte = buf[pn];
    let mut i_var3: usize = 3;
    let mut term = b'\'';
    let mut done = false;
    if ch_byte != b'\'' {
        while !done {
            loop {
                if !is_alpha(ch_byte) {
                    i_var3 = i_var9 + 1;
                    term = ch_byte;
                    done = true;
                    break;
                }
                sch[i_var9] = make_upper(ch_byte);
                i_var3 = pn;
                let i_var10 = i_var9 + 1;
                pn += 1;
                ch_byte = buf[pn];
                if i_var10 < 99 {
                    i_var9 = i_var10;
                    break;
                }
                sch[i_var9 + 1] = b' ';
                sch[i_var9 + 2] = 0;
                xlate_word(pyo, &sch);
                i_var9 = 1;
                if ch_byte == b'\'' {
                    i_var3 = 2;
                    term = b'\'';
                    done = true;
                    break;
                }
            }
            if done {
                break;
            }
            if ch_byte == b'\'' {
                i_var3 = i_var9 + 1;
                term = b'\'';
                done = true;
            }
        }
    }
    sch[i_var9] = b' ';
    if i_var3 < sch.len() {
        sch[i_var3] = 0;
    }
    let slen = i_var3.min(sch.len());
    if is_digit(term) {
        spell_word(pyo, &sch);
        return pn.wrapping_sub(1);
    }
    if slen == 3 {
        eprintln!(
            "eng_letter slen3: sch[1]={} term={}",
            sch[1] as char, term as char
        );
        say_ascii(pyo, sch[1]);
        if term == b'-' && is_alpha(buf[pn + 1]) {
            pn += 1;
        }
        return pn.wrapping_sub(1);
    }
    if term == b'.' {
        let mut p = pn;
        abbrev(pyo, &sch, &mut p);
        return p.wrapping_sub(1);
    }
    xlate_word(pyo, &sch);
    if term == b'-' && is_alpha(buf[pn + 1]) {
        pn += 1;
    }
    pn.wrapping_sub(1)
}

#[expect(
    clippy::branches_sharing_code,
    reason = "C port: shared code in branches kept as-is (extraction would change control flow)"
)]
fn phoneme_to_pyogi(pyo: &[u8]) -> String {
    let mut buf: Vec<u8> = Vec::with_capacity(pyo.len() + 2);
    buf.push(b' ');
    buf.extend_from_slice(pyo);
    let mut out = Vec::with_capacity(buf.len());
    let mut i = 0usize;
    let len = buf.len();
    while i < len {
        let c = buf[i];
        let set: &[u8] = match c {
            b' ' => {
                i += 1;
                continue;
            }
            b'A' => b"AEHOWXY",
            b'C' | b'D' | b'H' | b'S' | b'T' | b'W' | b'Z' => b"H",
            b'E' => b"HRY",
            b'I' => b"HY",
            b'N' => b"G",
            b'O' => b"WY",
            b'U' => b"WH",
            _ => {
                out.push(c);
                out.push(b' ');
                i += 1;
                continue;
            }
        };
        let next = buf.get(i + 1).copied().unwrap_or(0);
        if set.contains(&next) {
            out.push(c);
            out.push(next);
            out.push(b' ');
            i += 2;
        } else {
            i += 2;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[must_use]
pub fn english_prosess(word: &[u8]) -> String {
    let mut buf: Vec<u8> = word.to_vec();
    buf.push(b' ');
    buf.push(0);
    let mut pyo = Pyo::default();
    let n = buf.len() - 1;
    let mut i = 0usize;
    while i < n {
        let c = buf[i];
        if is_digit(c) {
            i = english_number(&mut pyo, &buf, i);
        } else if is_alpha(c) {
            i = english_letter(&mut pyo, &buf, i);
        } else if c == b'$' && is_digit(buf[i + 1]) {
            english_dollars(&mut pyo, &buf, &mut i);
        } else {
            english_special(&mut pyo, c);
        }
        i += 1;
    }
    phoneme_to_pyogi(&pyo.buf)
}

fn init_alpha(ctx: &EngDicts, phonemes: &str) -> Vec<i16> {
    let items: Vec<&[u8]> = phonemes
        .split(' ')
        .filter(|t| !t.is_empty())
        .map(str::as_bytes)
        .collect();
    let mut alpha: Vec<i16> = Vec::new();
    let mut i = 0usize;
    while i < items.len() {
        let c = match ctx.engsym_code(items[i]) {
            Some(c) => c,
            None => {
                if let Some((_, v)) = crate::eng_tables::ENG_SYM_TBL
                    .iter()
                    .find(|(k, _)| k.as_bytes() == items[i])
                {
                    *v
                } else {
                    i += 1;
                    continue;
                }
            }
        };
        let b = i16::from(c);
        let n = alpha.len();
        let merged = match b {
            0x1d => {
                if n > 0 && alpha[n - 1] == 0x1f {
                    alpha[n - 1] = 0x2a;
                    true
                } else {
                    false
                }
            }
            0x13 | 0x26 if n > 0 && alpha[n - 1] == 9 => {
                alpha[n - 1] = if b == 0x26 { 0x2b } else { 0x13 };
                true
            }
            _ => false,
        };
        if merged {
            continue;
        }
        match b {
            0x25 => alpha.push(0x25),
            9 => alpha.push(9),
            100 => {
                alpha.push(0x11);
                alpha.push(0xc);
            }
            101 => {
                alpha.push(0xb);
                alpha.push(0xc);
            }
            102 => {
                alpha.push(0x21);
                alpha.push(0xc);
            }
            _ => alpha.push(b),
        }
        i += 1;
    }
    alpha
}

#[inline]
#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn get_alpha_type(code: i16) -> i32 {
    if (1..=0x2b).contains(&code) {
        CSWTCH_55[(code - 1) as usize]
    } else {
        -1
    }
}

#[inline]
const fn is_str_end(count: usize, end: usize) -> bool {
    end - 1 == count
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn is_more_alpha(alpha: &[i16], count: usize, end: usize) -> bool {
    if end - 1 == count {
        return false;
    }
    let next = alpha[count + 1];
    if (7..=0x2b).contains(&next) {
        CSWTCH_61[(next - 7) as usize] != 0
    } else {
        false
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn is_jong_code(alpha: &[i16], count: usize) -> bool {
    if count == 0 {
        return false;
    }
    let prev = alpha[count - 1];
    if (2..=0x28).contains(&prev) {
        CSWTCH_58[(prev - 2) as usize] != 0
    } else {
        false
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn is_checksw_alpha_end(alpha: &[i16], count: usize, end: usize) -> bool {
    if end - 1 == count {
        return false;
    }
    let next = alpha[count + 1];
    if (7..=0x27).contains(&next) {
        CSWTCH_64[(next - 7) as usize] != 0
    } else {
        false
    }
}

#[derive(Clone)]
struct CvcState {
    cho: [u8; 42],
    jung: [u8; 42],
    jong: [u8; 42],
    index: usize,
    count: usize,
}

impl CvcState {
    const fn new() -> Self {
        Self {
            cho: [13; 42],
            jung: [2; 42],
            jong: [1; 42],
            index: 0,
            count: 0,
        }
    }
}

fn voiceless_consonant(st: &mut CvcState, alpha: &[i16], end: usize) {
    let s = alpha[st.count];
    let _i5: usize;
    if is_str_end(st.count, end) {
        if !is_jong_code(alpha, st.count) {
            match s {
                0x1b => st.cho[st.index] = 0x13,
                0x1f => st.cho[st.index] = 0x12,
                0x14 => st.cho[st.index] = 0x11,
                _ => {}
            }
            st.count += 1;
            st.jung[st.index] = 0x1b;
            st.index += 1;
            return;
        }
        match s {
            0x1b => st.jong[st.index - 1] = 0x13,
            0x1f => st.jong[st.index - 1] = 0x15,
            0x14 => {}
            _ => st.jong[st.index - 1] = 2,
        }
        st.count += 1;
        return;
    }
    if !is_more_alpha(alpha, st.count, end) {
        match s {
            0x1b => {
                st.count += 1;
                st.cho[st.index] = 0x13;
                return;
            }
            0x1f => {
                st.count += 1;
                st.cho[st.index] = 0x12;
                return;
            }
            0x14 => {
                st.cho[st.index] = 0x11;
            }
            _ => {}
        }
        st.count += 1;
        return;
    }
    let mut i5 = st.count + 1;
    if is_jong_code(alpha, st.count) {
        let s2 = alpha[st.count + 1];
        if s2 == 0x1c || s2 == 0x15 || s2 == 0x16 || s2 == 0x17 {
            i5 = st.count + 1;
        } else {
            match s {
                0x1b => {
                    st.count = i5;
                    st.jong[st.index - 1] = 0x1c;
                    return;
                }
                0x1f => {
                    st.count = i5;
                    st.jong[st.index - 1] = 0x1b;
                    return;
                }
                0x14 => {
                    st.count = i5;
                    st.jong[st.index - 1] = 0x1a;
                    return;
                }
                _ => {}
            }
        }
    }
    match s {
        0x1b => st.cho[st.index] = 0x13,
        0x1f => st.cho[st.index] = 0x12,
        0x14 => {
            st.cho[st.index] = 0x11;
            if alpha[i5] == 0x29 || alpha[i5] == 0x24 {
                st.count = i5;
                return;
            }
        }
        _ => {}
    }
    st.count = i5;
    st.jung[st.index] = 0x1b;
    st.index += 1;
}

#[expect(
    clippy::branches_sharing_code,
    reason = "C port: shared code in branches kept as-is (extraction would change control flow)"
)]
fn set_plosive(st: &mut CvcState, alpha: &[i16], end: usize) {
    let s = alpha[st.count];
    match s {
        7 => st.cho[st.index] = 9,
        9 => st.cho[st.index] = 5,
        0xf => st.cho[st.index] = 2,
        _ => {}
    }
    if is_str_end(st.count, end) {
        st.jung[st.index] = 0x1b;
        st.index += 1;
        st.count += 1;
    } else {
        if is_more_alpha(alpha, st.count, end) {
            st.jung[st.index] = 0x1b;
            st.index += 1;
        }
        st.count += 1;
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
#[expect(
    clippy::branches_sharing_code,
    reason = "C port: shared code in branches kept as-is (extraction would change control flow)"
)]
fn set_fricative(st: &mut CvcState, alpha: &[i16], end: usize) {
    let s = alpha[st.count];
    if s > 0x27 || (s as u16).wrapping_sub(10) > 0x1d {
        return;
    }
    let bit = 1u32 << (s - 10);
    if bit & 0x1248_0011 != 0 {
        match s {
            0x1d | 0x20 => st.cho[st.index] = 0xb,
            0x26 => st.cho[st.index] = 0xe,
            0xe => st.cho[st.index] = 0x13,
            0x23 => st.cho[st.index] = 9,
            10 => st.cho[st.index] = 5,
            _ => {}
        }
        if is_str_end(st.count, end) || is_more_alpha(alpha, st.count, end) {
            st.jung[st.index] = 0x1b;
            st.index += 1;
            st.count += 1;
        } else {
            st.count += 1;
        }
        return;
    }
    if bit & 0x2000_0000 != 0 {
        let i3 = st.index;
        st.cho[i3] = 0xe;
        if is_str_end(st.count, i3 * 6) {
            st.jung[st.index] = 0x17;
            st.index += 1;
        }
        st.count += 1;
        return;
    }
    if bit & 0x10_0000 != 0 {
        if is_str_end(st.count, end) {
            st.cho[st.index] = 0xb;
            st.jung[st.index] = 0x17;
            st.index += 1;
            st.count += 1;
            return;
        }
        if is_more_alpha(alpha, st.count, end) {
            st.cho[st.index] = 0xb;
            if alpha[st.count + 1] != 0x17 {
                st.jung[st.index] = 0x1a;
                st.index += 1;
                st.count += 1;
                return;
            }
            st.jung[st.index] = 0xb;
            st.index += 1;
            st.count += 1;
            return;
        }
        let s2 = alpha[st.count + 1];
        let i3 = st.index;
        st.cho[i3] = 0xb;
        match s2 {
            6 => {
                st.jung[i3] = 5;
                st.jung[i3 + 1] = 0x1d;
                st.index = i3 + 2;
                st.count += 2;
            }
            5 => {
                st.jung[i3] = 5;
                st.jung[i3 + 1] = 0x14;
                st.index = i3 + 2;
                st.count += 2;
            }
            0xd => {
                st.jung[i3] = 0xc;
                st.jung[i3 + 1] = 0x1d;
                st.index = i3 + 2;
                st.count += 2;
            }
            0x1a => {
                st.jung[i3] = 0x13;
                st.jung[i3 + 1] = 0x1d;
                st.index = i3 + 2;
                st.count += 2;
            }
            0x19 => {
                st.jung[i3] = 0x13;
                st.index = i3 + 1;
                st.count += 2;
            }
            1 => {
                st.jung[i3] = 5;
                st.index = i3 + 1;
                st.count += 2;
            }
            2 => {
                st.jung[i3] = 6;
                st.index = i3 + 1;
                st.count += 2;
            }
            0xc | 3 | 0x28 => {
                st.jung[i3] = 0xb;
                st.index = i3 + 1;
                st.count += 2;
            }
            4 => {}
            0xb => {
                st.jung[i3] = 0xc;
                st.index = i3 + 1;
                st.count += 2;
            }
            0x11 | 0x12 => {
                st.jung[i3] = 0x17;
                st.index = i3 + 1;
                st.count += 2;
            }
            0x21 | 0x22 => {
                st.jung[i3] = 0x1a;
                st.index = i3 + 1;
                st.count += 2;
            }
            _ => {
                st.count += 1;
            }
        }
        return;
    }
    st.count += 1;
}

fn set_affricate(st: &mut CvcState, alpha: &[i16], end: usize) {
    let s = alpha[st.count];
    if s == 0x2a {
        st.cho[st.index] = 0x10;
    } else if s == 0x2b {
        st.cho[st.index] = 0xe;
    }
    if is_str_end(st.count, end) || is_more_alpha(alpha, st.count, end) {
        st.jung[st.index] = 0x1b;
        st.index += 1;
    }
    st.count += 1;
}

fn set_cleft_voice(st: &mut CvcState, alpha: &[i16], end: usize) {
    let s = alpha[st.count];
    if s == 8 {
        st.cho[st.index] = 0x10;
    } else if s == 0x13 {
        st.cho[st.index] = 0xe;
    }
    if is_str_end(st.count, end) || is_more_alpha(alpha, st.count, end) {
        st.jung[st.index] = 0x1d;
        st.index += 1;
    }
    st.count += 1;
}

#[expect(
    clippy::branches_sharing_code,
    reason = "C port: shared code in branches kept as-is (extraction would change control flow)"
)]
fn set_nasal_voice(st: &mut CvcState, alpha: &[i16], end: usize) {
    let mut i4 = st.count;
    if i4 == 0 && st.index == 0 {
        if is_more_alpha(alpha, 0, end) {
            match alpha[0] {
                0x16 => st.cho[st.index] = 8,
                0x17 => st.cho[st.index] = 4,
                _ => {}
            }
            st.jung[st.index] = 0x1b;
            st.index += 1;
            st.count += 1;
            i4 = st.count;
        } else {
            i4 = st.count;
        }
    }
    if !is_str_end(i4, end) && !is_more_alpha(alpha, st.count, end) {
        match alpha[st.count] {
            0x16 => {
                st.cho[st.index] = 8;
                st.count += 1;
                return;
            }
            0x17 => {
                st.cho[st.index] = 4;
                st.count += 1;
                return;
            }
            _ => {}
        }
    } else {
        match alpha[st.count] {
            0x16 => {
                if st.index >= 1 {
                    st.jong[st.index - 1] = 0x11;
                }
                st.count += 1;
                return;
            }
            0x17 => {
                if st.index >= 1 {
                    st.jong[st.index - 1] = 5;
                }
                st.count += 1;
                return;
            }
            _ => {}
        }
    }
    if alpha[st.count] == 0x18 && st.index >= 1 {
        st.jong[st.index - 1] = 0x17;
    }
    st.count += 1;
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn set_echo_voice(st: &mut CvcState, alpha: &[i16], end: usize) {
    let mut i4 = st.count;
    if alpha[i4] == 0x1c {
        i4 += 1;
        st.cho[st.index] = 7;
        st.count = i4;
        return;
    }
    let head = i4 == 0 && st.index == 0;
    if !head || !is_more_alpha(alpha, 0, end) {
        if is_str_end(i4, end) {
            st.count += 1;
            if st.index > 0 {
                st.jong[st.index - 1] = 9;
            }
            return;
        }
        if !is_more_alpha(alpha, st.count, end) {
            let c = st.count;
            if c > 0 && (alpha[c - 1] as u16).wrapping_sub(0x16) > 2 {
                let i1 = st.index;
                st.count = c + 1;
                if i1 > 0 {
                    st.jong[i1 - 1] = 9;
                }
                st.cho[i1] = 7;
                return;
            }
            st.count += 1;
            st.cho[st.index] = 7;
            return;
        }
        if st.index > 0 {
            st.jong[st.index - 1] = 9;
        }
        i4 = st.count + 1;
        if (alpha[i4] as u16).wrapping_sub(0x16) > 2 {
            st.count = i4;
            return;
        }
        if is_checksw_alpha_end(alpha, i4, end) {
            st.count += 1;
            return;
        }
    }
    let i1 = st.index;
    st.cho[i1] = 7;
    st.jung[i1] = 0x1b;
    st.index = i1 + 1;
    st.count += 1;
}

fn set_monophthong(st: &mut CvcState, alpha: &[i16]) {
    let s = alpha[st.count];
    let jung: u8 = match s {
        1 => 0x3,
        2 => 0x4,
        3 | 12 | 40 => 0x7,
        4 => 0xd,
        11 => 0xa,
        17 | 18 => 0x1d,
        33 | 34 => 0x14,
        _ => {
            st.count += 1;
            return;
        }
    };
    st.jung[st.index] = jung;
    st.index += 1;
    st.count += 1;
}

fn set_diphthong(st: &mut CvcState, alpha: &[i16]) {
    let s = alpha[st.count];
    let i2 = st.index;
    match s {
        5 => {
            st.jung[i2] = 3;
            st.jung[i2 + 1] = 0x14;
            st.index = i2 + 2;
        }
        6 | 7 | 0x1b => {
            st.jung[i2] = 3;
            st.jung[i2 + 1] = 0x1d;
            st.index = i2 + 2;
        }
        0xd => {
            st.jung[i2] = 10;
            st.jung[i2 + 1] = 0x1d;
            st.index = i2 + 2;
        }
        0x19 => {
            st.jung[i2] = 0xd;
            st.index = i2 + 1;
        }
        0x1a => {
            st.jung[i2] = 0xd;
            st.jung[i2 + 1] = 0x1d;
            st.index = i2 + 2;
        }
        _ => {
            st.count += 1;
            return;
        }
    }
    st.count += 1;
}

fn set_castle_voice(st: &mut CvcState, alpha: &[i16], end: usize) {
    st.cho[st.index] = 0x14;
    if is_more_alpha(alpha, st.count, end) {
        st.jung[st.index] = 0x1b;
        st.index += 1;
    }
    st.count += 1;
}

#[allow(unused_assignments)]
fn set_nosal_comb(st: &mut CvcState, alpha: &[i16], end: usize) {
    let n_comp = st.count;
    let mut i2 = n_comp + 1;
    let mut wf_set = *alpha.get(i2).unwrap_or(&0);
    if wf_set == 0x25 {
        st.count = i2;
        wf_set = *alpha.get(n_comp + 2).unwrap_or(&0);
        i2 = st.count;
    }
    if is_more_alpha(alpha, n_comp, end) {
        st.jung[st.index] = 0x15;
        st.index += 1;
        st.count += 1;
        return;
    }
    if is_str_end(st.count, end) {
        st.jung[st.index] = 0x14;
        st.index += 1;
        st.count += 1;
        return;
    }
    let i3 = st.index;
    match wf_set {
        6 => {
            st.jung[i3] = 0xe;
            st.jung[i3 + 1] = 0x1d;
            st.index = i3 + 2;
            st.count += 2;
        }
        5 => {
            st.jung[i3] = 0xe;
            st.jung[i3 + 1] = 0x14;
            st.index = i3 + 2;
            st.count += 2;
        }
        0xd => {
            st.jung[i3] = 0x16;
            st.jung[i3 + 1] = 0x1d;
            st.index = i3 + 2;
            st.count += 2;
        }
        0x1a => {
            st.jung[i3] = 0x15;
            st.jung[i3 + 1] = 0x1d;
            st.index = i3 + 2;
            st.count += 2;
        }
        1 => {
            st.jung[i3] = 0xe;
            st.index = i3 + 1;
            st.count += 2;
        }
        2 => {
            st.jung[i3] = 0xf;
            st.index = i3 + 1;
            st.count += 2;
        }
        0x19 | 0xc | 3 | 4 | 0x28 => {
            st.jung[i3] = 0x15;
            st.index = i3 + 1;
            st.count += 2;
        }
        0xb => {
            st.jung[i3] = 0x16;
            st.index = i3 + 1;
            st.count += 2;
        }
        0x11 | 0x12 => {
            st.jung[i3] = 0x17;
            st.index = i3 + 1;
            st.count += 2;
        }
        0x21 | 0x27 => {
            st.jung[i3] = 0x14;
            st.index = i3 + 1;
            st.count += 2;
        }
        _ => {
            st.index += 1;
            st.count += 1;
        }
    }
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
fn set_high_voice(st: &mut CvcState, alpha: &[i16], end: usize) {
    if !is_more_alpha(alpha, st.count, end) && !is_str_end(st.count, end) {
        let local_30 = st.count;
        let prev = if local_30 >= 1 {
            alpha[local_30 - 1]
        } else {
            0
        };
        let next1 = alpha[local_30 + 1];
        let cond_a = local_30 < 1
            || ((prev != 0x15 && prev != 9 && prev != 0x1c && prev != 0x17)
                && (next1 != 3 && next1 != 0xc && next1 != 0x28));
        if cond_a {
            let local_30 = local_30 + 1;
            let s = alpha[local_30];
            let i4 = st.index;
            match s {
                6 => {
                    st.jung[i4] = 5;
                    st.jung[i4 + 1] = 0x1d;
                    st.index = i4 + 2;
                    st.count += 2;
                }
                5 => {
                    st.jung[i4] = 0x14;
                    st.index = i4 + 2;
                    st.count += 2;
                }
                0xd => {
                    st.jung[i4] = 6;
                    st.jung[i4 + 1] = 0x1d;
                    st.index = i4 + 2;
                    st.count += 2;
                }
                0x1a => {
                    st.jung[i4] = 0x13;
                    st.jung[i4 + 1] = 0x1d;
                    st.index = i4 + 2;
                    st.count += 2;
                }
                0x19 => {
                    st.jung[i4] = 0x13;
                    st.jung[i4 + 1] = 0x14;
                    st.index = i4 + 2;
                    st.count += 2;
                }
                1 => {
                    st.jung[i4] = 5;
                    st.index = i4 + 1;
                    st.count += 2;
                }
                2 => {
                    st.jung[i4] = 6;
                    st.index = i4 + 1;
                    st.count += 2;
                }
                0xc | 3 | 0x28 => {
                    st.jung[i4] = 0xb;
                    st.index = i4 + 1;
                    st.count += 2;
                }
                4 => {
                    st.jung[i4] = 0x13;
                    st.index = i4 + 1;
                    st.count += 2;
                }
                0xb => {
                    st.jung[i4] = 0xc;
                    st.index = i4 + 1;
                    let s2 = alpha[st.count + 3];
                    st.count += 2;
                    if (s2 as u16).wrapping_sub(0x11) < 2 {
                        st.jung[i4] = 0x1d;
                    } else if (s2 as u16).wrapping_sub(0x21) <= 1 {
                        st.jung[i4] = 0x1a;
                    } else {
                        st.count += 2;
                        return;
                    }
                    st.index = i4 + 1;
                    st.count += 2;
                    return;
                }
                0x11 | 0x12 => {
                    st.jung[i4] = 0x1d;
                    st.index = i4 + 1;
                    st.count += 2;
                }
                0x21 | 0x22 => {
                    st.jung[i4] = 0x1a;
                    st.index = i4 + 1;
                    st.count += 2;
                }
                _ => {
                    st.count += 2;
                    return;
                }
            }
            return;
        }
        st.jung[st.index] = 0x1d;
        st.jung[st.index + 1] = 7;
        st.count = local_30 + 2;
        st.index += 2;
        return;
    }
    st.jung[st.index] = 0x1d;
    st.index += 1;
    st.count += 1;
}

fn make_eng_johab_code(alpha_in: &[i16]) -> Option<Vec<u8>> {
    let mut alpha = alpha_in.to_vec();
    let mut mi = 1usize;
    while mi < alpha.len() {
        let b = alpha[mi];
        if b == 0x1d && alpha[mi - 1] == 0x1f {
            alpha[mi - 1] = 0x2a;
        } else if (b == 0x13 || b == 0x26) && alpha[mi - 1] == 9 {
            alpha[mi - 1] = if b == 0x26 { 0x2b } else { 0x13 };
        }
        mi += 1;
    }
    let end = alpha.len();
    if end == 0 {
        return None;
    }
    let mut st = CvcState::new();
    while st.count < end {
        if st.index > 0x28 {
            return None;
        }
        match get_alpha_type(alpha[st.count]) {
            1 => voiceless_consonant(&mut st, &alpha, end),
            2 => set_plosive(&mut st, &alpha, end),
            3 => set_fricative(&mut st, &alpha, end),
            4 => set_affricate(&mut st, &alpha, end),
            5 => set_cleft_voice(&mut st, &alpha, end),
            6 => set_nasal_voice(&mut st, &alpha, end),
            7 => set_echo_voice(&mut st, &alpha, end),
            8 => set_monophthong(&mut st, &alpha),
            9 => set_diphthong(&mut st, &alpha),
            10 => set_castle_voice(&mut st, &alpha, end),
            11 => set_nosal_comb(&mut st, &alpha, end),
            12 => set_high_voice(&mut st, &alpha, end),
            _ => {
                st.count += 1;
            }
        }
    }
    let mut out = Vec::with_capacity(st.index * 3);
    for i in 0..st.index {
        out.push(st.cho[i]);
        out.push(st.jung[i]);
        out.push(st.jong[i]);
    }
    Some(out)
}

#[must_use]
pub fn change_pro_to_johab(ctx: &EngDicts, word: &[u8]) -> Option<Vec<u8>> {
    let phonemes = ctx.english_lookup(word).map_or_else(
        || {
            let ph = english_prosess(word);
            ph.to_ascii_lowercase()
        },
        |p| String::from_utf8_lossy(&p).into_owned(),
    );
    let alpha = init_alpha(ctx, &phonemes);
    make_eng_johab_code(&alpha)
}

fn is_all_large_letter(word: &[u8]) -> bool {
    !word.is_empty()
        && word
            .iter()
            .all(|&c| !c.is_ascii_alphabetic() || c.is_ascii_uppercase())
}

#[must_use]
pub fn english_word_to_pyogi(ctx: &EngDicts, word: &[u8]) -> Option<String> {
    if word.is_empty() {
        return None;
    }
    let lower: Vec<u8> = word.iter().map(|&c| c.to_ascii_lowercase()).collect();
    if let Some(r) = ctx.unienglishpron_lookup(&lower) {
        return Some(r);
    }
    if !is_all_large_letter(word)
        && lower.len() < 0x15
        && let Some(cvc) = change_pro_to_johab(ctx, &lower)
    {
        return Some(String::from_utf8_lossy(&crate::code::conv_cvc_to_pyogi(&cvc)).into_owned());
    }
    let mut s = String::new();
    for &c in &lower {
        if c.is_ascii_lowercase() {
            s.push_str(GSCH_ENGI_KOR_TBL[(c - b'a') as usize]);
            s.push(' ');
        }
    }
    if s.is_empty() { None } else { Some(s) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dbg_prosess() {
        for w in [
            "A.B", "dr.", "phd.", "mr.", "a-b", "hello", "KCC", "AZXQ", "x", "xl",
        ] {
            println!("{w:8} -> [{}]", english_prosess(w.as_bytes()));
        }
    }

    #[test]
    fn prosess_rules_known_words() {
        let ctx = EngDicts::empty();
        let cases = [
            ("are", "a"),
            ("card", "kad_"),
            ("wifi", "waipi"),
            ("xyz", "k_sij_"),
            ("azxq", "8j_k_s_k_"),
        ];
        for (word, expect) in cases {
            let got = english_word_to_pyogi(&ctx, word.as_bytes());
            assert_eq!(got.as_deref(), Some(expect), "{word}");
        }
    }

    #[test]
    fn onechar_spelling_path() {
        let ctx = EngDicts::empty();
        assert_eq!(
            english_word_to_pyogi(&ctx, b"KCC").as_deref(),
            Some("k9i vi vi ")
        );
        assert_eq!(
            english_word_to_pyogi(&ctx, b"AZXQ").as_deref(),
            Some("9i j9t_ 9Gs_ kyu ")
        );
        let long = "abcdefghijklmnopqrstuvwxyz";
        let got = english_word_to_pyogi(&ctx, long.as_bytes()).unwrap();
        assert!(got.starts_with("9i "), "{got}");
    }

    #[test]
    fn phoneme_merge() {
        assert_eq!(phoneme_to_pyogi(b"AA r "), "AA r ");
        assert_eq!(phoneme_to_pyogi(b"AB "), "");
        assert_eq!(phoneme_to_pyogi(b"NG "), "NG ");
        assert_eq!(phoneme_to_pyogi(b"CA "), "");
        assert_eq!(phoneme_to_pyogi(b"ZH "), "ZH ");
        assert_eq!(phoneme_to_pyogi(b"UE "), "");
    }

    #[test]
    fn pattern_matching() {
        assert!(rightmatch("%", b"ING", 0));
        assert!(rightmatch("%", b"E", 0));
        assert!(rightmatch("%", b"ELYZ", 0));
        assert!(!rightmatch("%", b"IZZ", 0));
        assert!(rightmatch("%L", b"ELZ", 0));
        assert!(!rightmatch("%R", b"ERZ", 0));
        assert!(!rightmatch("%I", b"ING", 0));
        assert!(rightmatch("#", b"AA", 0));
        assert!(!rightmatch("#", b"BA", 0));
        assert!(rightmatch("^", b"B", 0));
        assert!(!rightmatch("^", b"A", 0));
        assert!(rightmatch("+", b"I", 0));
        assert!(!rightmatch("+", b"X", 0));
        assert!(rightmatch(".", b"B", 0));
        assert!(!rightmatch(".", b"A", 0));
        assert!(rightmatch(":", b"CCC", 0));
        assert!(rightmatch(":", b"A", 0));
        assert!(!rightmatch("^%", b"FILY", 0));
        assert!(rightmatch("^%", b"FING", 0));
        assert!(leftmatch("^", b"B", 0));
        assert!(!leftmatch("^", b"A", 0));
        assert!(leftmatch(".", b"B", 0));
        assert!(!leftmatch(".", b"A", 0));
        assert!(leftmatch("+", b"I", 0));
        assert!(leftmatch(":", b"ZZ", 1));
        assert!(leftmatch(":", b"A", 0));
    }

    #[test]
    fn xlate_basic() {
        let mut pyo = Pyo::default();
        xlate_word(&mut pyo, b" ARE ");
        assert_eq!(pyo.buf, b"AAr   ");
        let mut pyo = Pyo::default();
        xlate_word(&mut pyo, b" WIFI ");
        assert_eq!(pyo.buf, b"w AY f IH   ");
        let mut pyo = Pyo::default();
        xlate_word(&mut pyo, b" HELLO ");
        assert_eq!(pyo.buf, b"h EH l OW   ");
        let mut pyo = Pyo::default();
        xlate_word(&mut pyo, b" CARD ");
        assert_eq!(pyo.buf, b"k AAr d   ");
    }

    #[test]
    fn cardinal_ordinal() {
        let mut pyo = Pyo::default();
        say_cardinal(&mut pyo, 12345);
        assert_eq!(pyo.buf, b"twEHlv THAWzAEnd THrIY hAHndrEHd fAOrtIY fAYv ");
        let mut pyo = Pyo::default();
        say_cardinal(&mut pyo, 1_000_000);
        assert_eq!(pyo.buf, b"wAHn mIHlIYAXn ");
        let mut pyo = Pyo::default();
        say_ordinal(&mut pyo, 21);
        assert_eq!(pyo.buf, b"twEHntIY fERst ");
        let mut pyo = Pyo::default();
        say_ordinal(&mut pyo, 1000);
        assert_eq!(pyo.buf, b"wAHn THAWzAEndTH ");
    }

    #[test]
    fn english_prosess_oracle() {
        let cases = [
            ("hello", "h EH l OW "),
            ("computer", "k AA m p y UW t ER "),
            ("wifi", "w AY f IH "),
            ("card", "k AA r d "),
            ("are", "AA r "),
            ("azxq", "AE z k s k "),
            ("don", "d AH n "),
            ("t", "t IY "),
            ("s", "EH z "),
            ("xl", "k s l "),
            ("xyz", "k s IH z "),
            ("dr.", "d AA k t ER "),
            ("mr.", "m IH s t ER "),
            ("mrs.", "m IH s AH s "),
            ("phd.", "p IY EY t CH d IY "),
            ("1st", "f ER s t "),
            ("21st", "t w EH n t IY f ER s t "),
            ("123", "w AH n h AH n d r EH d t w EH n t IY TH r IY "),
            ("2.5", "t UW p OY n t f AY v "),
            ("a-b", "EY b IY "),
            ("A.B", "EY p IY r IY AA d b IY "),
            ("don't", "d AH n k w OW t t IY "),
        ];
        for (input, expect) in cases {
            let got = english_prosess(input.as_bytes());
            assert_eq!(&got, expect, "{input}");
        }
    }

    #[test]
    fn dollars_oracle() {
        let cases = [
            ("$5", "f AY v d AA l AA r z "),
            ("$12", "t w EH l v d AA l AA r z "),
            ("$1.00", "w AH n d AA l ER "),
            (
                "$0.99",
                "z IH r OW d AA l AA r z AA n d n AY n t IY n AY n s EH n t s ",
            ),
            ("$100", "w AH n h AH n d r EH d d AA l AA r z "),
            (
                "$12.50",
                "t w EH l v d AA l AA r z AA n d f IH f t IY s EH n t s ",
            ),
            ("$1.5", "w AH n p OY n t f AY v d AA l AA r z "),
            (
                "$1.500",
                "w AH n p OY n t f AY v z IH r OW z IH r OW d AA l AA r z ",
            ),
        ];
        for (input, expect) in cases {
            let got = english_prosess(input.as_bytes());
            assert_eq!(&got, expect, "{input}");
        }
    }

    #[test]
    fn word_final_k_silent() {
        let ctx = EngDicts::empty();
        assert_eq!(
            english_word_to_pyogi(&ctx, b"quick").as_deref(),
            Some("kwu")
        );
        assert_eq!(english_word_to_pyogi(&ctx, b"book").as_deref(), Some("bu"));
        assert_eq!(
            english_word_to_pyogi(&ctx, b"bake").as_deref(),
            Some("b9ik_")
        );
    }

    #[test]
    fn spellword_oracle() {
        let mut pyo = Pyo::default();
        spell_word(&mut pyo, b" KCC ");
        assert_eq!(pyo.buf, b"kEY  sIY  sIY  ");
    }
}
