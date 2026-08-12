pub const CHOSONG: [u8; 32] = *b"\0\0gqndflmbrsv\0jzcktph\0\0\0\0\0\0\0\0\0\0\0";
pub const JUNG_ONE: [u8; 32] = *b"\0\0\0a8yye\0\09yyoww\0\0wyuwww\0\0y_yi\0\0";
pub const JUNG_TWO: [u8; 32] = *b"\0\0\0\0\0a8\0\0\0\0e9\0a8\0\0io\0e9u\0\0u\0i\0\0\0";
pub const JONG_ONE: [u8; 32] = *b"\0\0GQGNNNDLLLLLLLLM\0BBSV*JCKTPH\0\0";
pub const JONG_TWO: [u8; 32] = *b"\0\0\0\0S\0JH\0\0GMBSTPH\0\0\0S\0\0\0\0\0\0\0\0\0\0\0";

pub const CHO_POS_TBL: [i16; 32] = [
    0, 0, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 0, 6, 6, 6, 6, 6, 6, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub const JUNG_POS_TBL: [i16; 32] = [
    0, 0, 0, 6, 6, 12, 12, 6, 0, 0, 6, 12, 12, 6, 12, 12, 0, 0, 12, 12, 6, 12, 12, 12, 0, 0, 12, 6,
    12, 6, 0, 0,
];
pub const JONG_POS_TBL: [i16; 32] = [
    0, 0, 6, 6, 12, 6, 12, 12, 6, 6, 12, 12, 12, 12, 12, 12, 12, 6, 0, 6, 12, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 0, 0,
];

pub const CHO_NO: [u8; 32] = *b"\0\0\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13\0\0\0\0\0\0\0\0\0\0\0";
pub const JUNG_NO: [u8; 32] = *b"\0\0\0\x01\x02\x03\x04\x05\0\0\x06\x07\x08\x09\x0a\x0b\0\0\x0c\x0d\x0e\x0f\x10\x11\0\0\x12\x13\x14\x15\0\0";
pub const JONG_NO: [u8; 32] = *b"\0\0\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\0\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\0\0";

pub const UNI_JUNG_ID: [u8; 21] = [
    3, 4, 5, 6, 7, 10, 11, 12, 13, 14, 15, 18, 19, 20, 21, 22, 23, 26, 27, 28, 29,
];
pub const UNI_JONG_ID: [u8; 28] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20, 21, 22, 23, 24, 25, 26, 27,
    28, 29,
];

pub const KS_JOHAB: [u16; 51] = [
    0x8422, 0x8423, 0x8424, 0x8425, 0x8426, 0x8427, 0x8428, 0x9821, 0x8429, 0x842a, 0x842b, 0x842c,
    0x842d, 0x842e, 0x842f, 0x8430, 0x8431, 0x8433, 0xa821, 0x8434, 0x8435, 0x8436, 0x8437, 0x8438,
    0xbc21, 0x8439, 0x843a, 0x843b, 0x843c, 0x843d, 0x8461, 0x8481, 0x84a1, 0x84c1, 0x84e1, 0x8541,
    0x8561, 0x8581, 0x85a1, 0x85c1, 0x85e1, 0x8641, 0x8661, 0x8681, 0x86a1, 0x86c1, 0x86e1, 0x8741,
    0x8761, 0x8781, 0x87a1,
];

pub const JAMO_CVC: [(u8, u8, u8); 51] = {
    let mut out = [(0u8, 0u8, 0u8); 51];
    let mut i = 0;
    while i < 51 {
        let w = KS_JOHAB[i];
        out[i] = (
            ((w >> 10) & 0x1f) as u8,
            ((w >> 5) & 0x1f) as u8,
            (w & 0x1f) as u8,
        );
        i += 1;
    }
    out
};

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
        4
    }
}

#[must_use]
pub fn is_k_root_pumsa(tag: u8) -> bool {
    (b'0'..=b'K').contains(&tag)
}

#[must_use]
pub fn is_k_char_root_part(tag: u8) -> bool {
    (b'0'..=b'G').contains(&tag) || tag == b'K'
}

#[must_use]
pub fn is_k_cheon_part(tag: u8) -> bool {
    (b'0'..=b'>').contains(&tag) || matches!(tag, b'F' | b'H' | b'I')
}

#[must_use]
pub fn is_to(tag: u8) -> bool {
    (b'T'..=b'h').contains(&tag)
}

#[must_use]
pub fn is_case_to(tag: u8) -> bool {
    (b'T'..=b'[').contains(&tag)
}

#[must_use]
pub const fn is_help_to(tag: u8) -> bool {
    tag == b']'
}

#[must_use]
pub const fn is_plural_to(tag: u8) -> bool {
    tag == b'\\'
}

#[must_use]
pub const fn is_bagum_yi(tag: u8) -> bool {
    tag == b'c'
}

#[must_use]
pub fn is_k_cheon_to(tag: u8) -> bool {
    is_case_to(tag) || is_help_to(tag) || is_plural_to(tag) || is_bagum_yi(tag)
}

#[must_use]
pub fn is_symbol_pumsa(tag: u8) -> bool {
    (b'L'..=b'R').contains(&tag)
}

#[must_use]
pub const fn is_k_symbol_pumsa(tag: u8) -> bool {
    matches!(
        tag,
        b'k' | b'I' | b'J' | b'L' | b'M' | b'N' | b'O' | b'P' | b'Q' | b'R' | b'S'
    )
}

#[must_use]
pub const fn is_k_yongon_to(tag: u8) -> bool {
    matches!(tag, b'^' | b'_' | b'`' | b'a' | b'b' | b'd' | b'h' | b'g')
}

#[must_use]
pub const fn is_k_voice_yong_yon(tag: u8) -> bool {
    matches!(tag, b'B' | b'@' | b'D' | b'C' | b'f')
}

#[must_use]
pub fn is_k_voice_yong_yon_to(tag: u8) -> bool {
    is_k_voice_yong_yon(tag) && is_to(tag)
}

#[must_use]
pub const fn is_k_voice_jarib_cheon(tag: u8) -> bool {
    if (tag.wrapping_sub(b'0') > 3) && tag != b'9' && tag != b'<' && tag != b'F' && tag != b':' {
        return tag == b'H';
    }
    true
}

#[must_use]
pub const fn is_bound_pumsa(tag: u8) -> bool {
    matches!(tag, b'k' | b'L' | b'M')
}

#[must_use]
pub const fn is_sen_symbol(w: u16) -> bool {
    matches!(w, 0x2e | 0x2c | 0x21 | 0x3f)
}

#[must_use]
pub fn is_korean_code(w: u16) -> bool {
    if (w >> 8) != 0 {
        i32::from(u8::from(w.wrapping_add(0x5400) < 0x2bb0)) * 2 - 1 != -1
    } else {
        false
    }
}

#[must_use]
pub const fn is_uni_korean_code(w: u16) -> bool {
    w.wrapping_add(0x5400) < 0x2ba4
}

#[must_use]
pub const fn is_uni_korean_jamo(w: u16) -> bool {
    w.wrapping_add(0xcecf) < 0x33
}

#[must_use]
pub const fn is_korean_code_uni(w: u16) -> bool {
    is_uni_korean_code(w)
}

pub const SSCH_CHO: [&[u8]; 32] = [
    b"", b"", b"g", b"q", b"n", b"d", b"f", b"l", b"m", b"b", b"r", b"s", b"v", b"", b"j", b"z",
    b"c", b"k", b"t", b"p", b"h", b"", b"", b"", b"", b"", b"", b"", b"", b"", b"", b"",
];
pub const SSCH_JUNG: [&[u8]; 32] = [
    b"", b"", b"", b"a", b"8", b"ya", b"y8", b"e", b"", b"", b"9", b"ye", b"y9", b"o", b"wa",
    b"w8", b"", b"", b"wi", b"yo", b"u", b"we", b"w9", b"wu", b"", b"", b"yu", b"_", b"yi", b"i",
    b"", b"",
];
pub const SSCH_JONG: [&[u8]; 32] = [
    b"", b"", b"G", b"Q", b"GS", b"N", b"NJ", b"NH", b"D", b"L", b"LG", b"LM", b"LB", b"LS", b"LT",
    b"LP", b"LH", b"M", b"", b"B", b"BS", b"S", b"V", b"*", b"J", b"C", b"K", b"T", b"P", b"H",
    b"", b"",
];

pub const PROB_ALL_ONE_NODE: f64 = -32.236_191_301_917;
pub const PENALTY_BIGRAM_ZERO: f64 = 36.841_361_487_905;
#[allow(clippy::approx_constant)]
pub const PENALTY_CHAR_ROOT: f64 = 2.302_585_092_994;
pub const PENALTY_CHEON: f64 = 4.605_170_185_988_1;
pub const PENALTY_VOICE_CHEON: f64 = 2.995_732_273_554;

pub const GRAPHEME_OFFSET: [i32; 20] = [
    -1, 0, -1, -1, 0, -2, -2, -1, -2, -2, -4, -1, 0, 0, 0, -1, -1, -1, 0, -1,
];

pub const CHEON_NO_LINKABLE: [&[u8]; 0x2f] = [
    b"su", b"beB", b"de", b"haN", b"c9", b"ma*je*", b"nawu", b"nyeM", b"caM", b"geL", b"s9M",
    b"g9", b"ca", b"d9", b"balaM", b"te", b"teG", b"ti", b"s8", b"juL", b"tu", b"geN", b"li",
    b"giM", b"ge", b"maNci", b"ceG", b"iL", b"fal_M", b"qadaLG", b"ja", b"ba", b"c8", b"jeG",
    b"j__M", b"j9", b"mulyeB", b"beN", b"dwu", b"joGjoG", b"qeS", b"ya*", b"j_G", b"haNpyeN",
    b"to*", b"t9", b"paN",
];
pub const YONGYON_NO_LINKABLE: [&[u8]; 0x38] = [
    b"al8",
    b"aN",
    b"aNpaG",
    b"aP",
    b"baL",
    b"baQ",
    b"bug_N",
    b"byeL",
    b"ci",
    b"co",
    b"d_*",
    b"d_Lsai",
    b"d8",
    b"da*",
    b"da_M",
    b"do*aNviG",
    b"eci",
    b"egaN",
    b"f8muN",
    b"faN",
    b"fawu",
    b"fawunoM",
    b"galya*",
    b"gaN",
    b"gaqai",
    b"gaT_NgeS",
    b"guNd9",
    b"h8*",
    b"ha",
    b"iha",
    b"ihu",
    b"ijeN",
    b"il8",
    b"in8",
    b"isa*",
    b"iwi",
    b"j__M",
    b"jeN",
    b"jiGjeN",
    b"ju*yeB",
    b"juyijeG",
    b"miT",
    b"n8",
    b"n8ji",
    b"q8na",
    b"qili",
    b"s_s_lo",
    b"si",
    b"teG",
    b"to*",
    b"u",
    b"viG",
    b"ye",
    b"yeha",
    b"zaG",
    b"zali",
];

#[must_use]
pub const fn check_chosong(moum: u8) -> i32 {
    if moum != 0 && (moum == 3 || moum == 1 || moum == 13) {
        return 1;
    }
    if moum == 9 { 2 } else { 0 }
}
