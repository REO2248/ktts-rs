#![expect(
    unused_crate_dependencies,
    reason = "deps used by other targets of this package"
)]
use ktts_pron::dicts::PronContext;
use ktts_pron::kma_code;
use ktts_pron::kma_types::{Morph, WordAnal};
use ktts_pron::symbols::*;

fn klang_dic() -> std::path::PathBuf {
    std::path::PathBuf::from(
        std::env::var("KTTSDB_DIR").expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
    )
    .join("KLangDic")
}

fn load_ctx() -> PronContext {
    PronContext::load(&klang_dic()).expect("KLangDic load failed")
}

#[test]
fn symbol_type_code_type1_unit_square_and_currency() {
    let ctx = load_ctx();
    assert_eq!(get_symbol_type_code(&ctx, 0x2103, 1).as_deref(), Some("do"));
    assert_eq!(get_symbol_type_code(&ctx, 0x2109, 1).as_deref(), Some("do"));
    assert_eq!(get_symbol_type_code(&ctx, 0x00B0, 1).as_deref(), Some("do"));
    assert_eq!(
        get_symbol_type_code(&ctx, 0x33CA, 1).as_deref(),
        Some("h9Gtal_")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, 0x33A1, 1).as_deref(),
        Some("pye*ba*m9te")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, 0x339E, 1).as_deref(),
        Some("kilom9te")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, 0x338F, 1).as_deref(),
        Some("kilog_laM")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, 0xFF05, 1).as_deref(),
        Some("p_lo")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, 0xFF04, 1).as_deref(),
        Some("faLla")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, 0x20AC, 1).as_deref(),
        Some("yulo")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, 0x2030, 1).as_deref(),
        Some("pipi9M")
    );
}

#[test]
fn symbol_type_code_type2_operators() {
    let ctx = load_ctx();
    assert_eq!(
        get_symbol_type_code(&ctx, '+' as u16, 2).as_deref(),
        Some("dehagi")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, '-' as u16, 2).as_deref(),
        Some("deLgi")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, '*' as u16, 2).as_deref(),
        Some("goBhagi")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, '/' as u16, 2).as_deref(),
        Some("nanugi")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, '~' as u16, 2).as_deref(),
        Some("n8ji")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, '=' as u16, 2).as_deref(),
        Some("gaTgi")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, ':' as u16, 2).as_deref(),
        Some("d8")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, '&' as u16, 2).as_deref(),
        Some("8Nd_")
    );
}

#[test]
fn symbol_type_code_type3_symbols() {
    let ctx = load_ctx();
    assert_eq!(
        get_symbol_type_code(&ctx, '~' as u16, 3).as_deref(),
        Some("tiLd_")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, '.' as u16, 3).as_deref(),
        Some("zeM")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, '/' as u16, 3).as_deref(),
        Some("s_Llasi")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, '@' as u16, 3).as_deref(),
        Some("9t_")
    );
}

#[test]
fn symbol_type_code_type4_katakana_greek() {
    let ctx = load_ctx();
    assert_eq!(get_symbol_type_code(&ctx, 0x30AB, 4).as_deref(), Some("ga"));
    assert_eq!(get_symbol_type_code(&ctx, 0x30F2, 4).as_deref(), Some("o"));
    assert_eq!(get_symbol_type_code(&ctx, 0x30F3, 4).as_deref(), Some("_*"));
    assert_eq!(get_symbol_type_code(&ctx, 0x3042, 4).as_deref(), Some("a"));
    assert_eq!(
        get_symbol_type_code(&ctx, 0x03B1, 4).as_deref(),
        Some("aLpa")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, 0x03C9, 4).as_deref(),
        Some("om9ga")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, 0x0416, 4).as_deref(),
        Some("jw9")
    );
}

#[test]
fn symbol_type_code_type5_roman_math() {
    let ctx = load_ctx();
    assert_eq!(get_symbol_type_code(&ctx, 0x2160, 5).as_deref(), Some("iL"));
    assert_eq!(get_symbol_type_code(&ctx, 0x2170, 5).as_deref(), Some("iL"));
    assert_eq!(
        get_symbol_type_code(&ctx, 0x2179, 5).as_deref(),
        Some("siB")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, 0x203B, 5).as_deref(),
        Some("caMgobuho")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, 0x221E, 5).as_deref(),
        Some("muhaNd8")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, 0x00D7, 5).as_deref(),
        Some("goBhagi")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, 0x00F7, 5).as_deref(),
        Some("nanugi")
    );
}

#[test]
fn symbol_type_code_type6_circled_numbers() {
    let ctx = load_ctx();
    assert_eq!(get_symbol_type_code(&ctx, 0x2460, 6).as_deref(), Some("iL"));
    assert_eq!(get_symbol_type_code(&ctx, 0x2463, 6).as_deref(), Some("sa"));
    assert_eq!(
        get_symbol_type_code(&ctx, 0x2473, 6).as_deref(),
        Some("isiB")
    );
    assert_eq!(
        get_symbol_type_code(&ctx, 0x3260, 6).as_deref(),
        Some("gi_G")
    );
    assert_eq!(get_symbol_type_code(&ctx, 0x3270, 6).as_deref(), Some("da"));
    assert_eq!(
        get_symbol_type_code(&ctx, 0x24F1, 6).as_deref(),
        Some("isiBi")
    );
}

#[test]
fn symbol_type_code_not_found() {
    let ctx = load_ctx();
    assert_eq!(get_symbol_type_code(&ctx, '#' as u16, 4), None);
    assert_eq!(get_symbol_type_code(&ctx, 'a' as u16, 1), None);
    assert_eq!(get_symbol_type_code(&ctx, 0xAC00, 2), None);
    assert!(!is_symbol_type_code(&ctx, '#' as u16, 2));
    assert!(is_symbol_type_code(&ctx, '+' as u16, 2));
    assert!(is_symbol_type_code(&ctx, 0x2460, 6));
}

#[test]
fn str_type_code_units() {
    let ctx = load_ctx();
    let u = |s: &str| s.encode_utf16().collect::<Vec<u16>>();
    assert_eq!(
        get_str_type_code(&ctx, &u("km"), 1).as_deref(),
        Some("kilom9te")
    );
    assert_eq!(
        get_str_type_code(&ctx, &u("kg"), 1).as_deref(),
        Some("kilog_laM")
    );
    assert_eq!(get_str_type_code(&ctx, &u("%"), 1).as_deref(), Some("p_lo"));
    assert_eq!(
        get_str_type_code(&ctx, &u("$"), 1).as_deref(),
        Some("faLla")
    );
    assert_eq!(
        get_str_type_code(&ctx, &u("m/s"), 1).as_deref(),
        Some("m9tem8co")
    );
    assert_eq!(
        get_str_type_code(&ctx, &u("hz"), 1).as_deref(),
        Some("h9l_z_")
    );
    assert_eq!(get_str_type_code(&ctx, &u("m"), 1).as_deref(), Some("m9te"));
    assert_eq!(get_str_type_code(&ctx, &u("s"), 1).as_deref(), Some("co"));
    assert_eq!(
        get_str_type_code(&ctx, &u("mm2"), 1).as_deref(),
        Some("pye*ba*milim9te")
    );
    assert_eq!(get_str_type_code(&ctx, &u("㎡"), 1), None);
    assert_eq!(get_str_type_code(&ctx, &u("xyz"), 1), None);
    assert!(is_str_type_code(&ctx, &u("km")));
    assert!(!is_str_type_code(&ctx, &u("zzz")));
}

#[test]
fn remove_symbol() {
    assert!(is_remove_symbol(&[u16::from(b'*'), u16::from(b'*')]));
    assert!(is_remove_symbol(&[
        u16::from(b'*'),
        u16::from(b'*'),
        u16::from(b'a'),
        u16::from(b'b')
    ]));
    assert!(is_remove_symbol(&[u16::from(b'/'), u16::from(b'/')]));
    assert!(is_remove_symbol(&[u16::from(b'/'), u16::from(b'*')]));
    assert!(is_remove_symbol(&[u16::from(b'*'), u16::from(b'/')]));
    assert!(is_remove_symbol(&[u16::from(b'-')]));
    assert!(is_remove_symbol(&[u16::from(b':')]));
    assert!(!is_remove_symbol(&[u16::from(b'+')]));
    assert!(!is_remove_symbol(&[u16::from(b'-'), u16::from(b'-')]));
    assert!(!is_remove_symbol(&[u16::from(b'*')]));
    assert!(!is_remove_symbol(&[u16::from(b'/'), u16::from(b'x')]));
    assert!(!is_remove_symbol(&[u16::from(b':'), u16::from(b':')]));
}

#[test]
fn process_symbol_operator() {
    let ctx = load_ctx();
    let items = process_symbol(&ctx, &[u16::from(b'+')]).unwrap();
    assert_eq!(
        items,
        vec![("dehagi".to_string(), b'0'), (" ".to_string(), b'k')]
    );
}

#[test]
fn process_symbol_roman_numeral() {
    let ctx = load_ctx();
    let items = process_symbol(&ctx, &[0x2170]).unwrap();
    assert_eq!(
        items,
        vec![("iL".to_string(), b'0'), (" ".to_string(), b'k')]
    );
}

#[test]
fn process_symbol_circled_number() {
    let ctx = load_ctx();
    let items = process_symbol(&ctx, &[0x2460]).unwrap();
    assert_eq!(
        items,
        vec![
            ("do*g_lami".to_string(), b'0'),
            (" ".to_string(), b'k'),
            ("iL".to_string(), b'0'),
            (",".to_string(), b'M'),
        ]
    );
}

#[test]
fn process_symbol_katakana() {
    let ctx = load_ctx();
    let items = process_symbol(&ctx, &[0x30AB]).unwrap();
    assert_eq!(items, vec![("ga".to_string(), b'0')]);
}

#[test]
fn process_symbol_mixed_types() {
    let ctx = load_ctx();
    let items = process_symbol(&ctx, &[0x203B, u16::from(b'+')]).unwrap();
    assert_eq!(
        items,
        vec![
            ("caMgobuho".to_string(), b'0'),
            (" ".to_string(), b'k'),
            ("dehagi".to_string(), b'0'),
            (" ".to_string(), b'k'),
        ]
    );
    let items = process_symbol(&ctx, &[u16::from(b'+'), 0x2460]).unwrap();
    assert_eq!(
        items,
        vec![
            ("dehagi".to_string(), b'0'),
            (" ".to_string(), b'k'),
            ("do*g_lami".to_string(), b'0'),
            (" ".to_string(), b'k'),
            ("iL".to_string(), b'0'),
            (",".to_string(), b'M'),
        ]
    );
}

#[test]
fn process_symbol_remove_symbol_word() {
    let ctx = load_ctx();
    let items = process_symbol(&ctx, &[u16::from(b'*'), u16::from(b'*')]).unwrap();
    assert_eq!(items, vec![(" ".to_string(), b'k')]);
    let items = process_symbol(&ctx, &[u16::from(b'-')]).unwrap();
    assert_eq!(items, vec![(" ".to_string(), b'k')]);
}

#[test]
fn process_symbol_unknown_dropped() {
    let ctx = load_ctx();
    let items = process_symbol(&ctx, &[u16::from(b'#')]).unwrap();
    assert!(items.is_empty());
    let items = process_symbol(&ctx, &[u16::from(b'#'), u16::from(b'+')]).unwrap();
    assert_eq!(
        items,
        vec![("dehagi".to_string(), b'0'), (" ".to_string(), b'k')]
    );
}

#[test]
fn unit_symbol_single_char() {
    let ctx = load_ctx();
    assert_eq!(
        process_unit_symbol(&ctx, &[0x339E]).as_deref(),
        Some("kilom9te")
    );
    assert_eq!(process_unit_symbol(&ctx, &[0x2103]).as_deref(), Some("do"));
    assert_eq!(
        process_unit_symbol(&ctx, &[0x33CA]).as_deref(),
        Some("h9Gtal_")
    );
}

#[test]
fn unit_symbol_multi_char() {
    let ctx = load_ctx();
    assert_eq!(
        process_unit_symbol(&ctx, &[u16::from(b'k'), u16::from(b'm')]).as_deref(),
        Some("kilom9te")
    );
    assert_eq!(
        process_unit_symbol(&ctx, &[u16::from(b'k'), u16::from(b'g')]).as_deref(),
        Some("kilog_laM")
    );
    assert_eq!(
        process_unit_symbol(&ctx, &[u16::from(b'm'), u16::from(b'/'), u16::from(b's')]).as_deref(),
        Some("m9tem8co")
    );
    assert_eq!(
        process_unit_symbol(&ctx, &[u16::from(b'h'), u16::from(b'z')]).as_deref(),
        Some("h9l_z_")
    );
    assert_eq!(
        process_unit_symbol(&ctx, &[u16::from(b'%')]).as_deref(),
        Some("p_lo")
    );
    assert_eq!(
        process_unit_symbol(&ctx, &[u16::from(b'z'), u16::from(b'z')]),
        None
    );
}

fn unit_word(tag: u8, chars: &[u16], morph_count: usize) -> UnitWord {
    UnitWord {
        tag,
        chars: chars.to_vec(),
        morph_count,
    }
}

#[test]
fn process_unit_i6_pattern() {
    let ctx = load_ctx();
    let words = [
        unit_word(b'I', &[u16::from(b'1'), u16::from(b'0')], 1),
        unit_word(b'6', &[u16::from(b'k'), u16::from(b'm')], 1),
    ];
    assert_eq!(
        process_unit(&ctx, &words, 0),
        Some((1, "kilom9te".to_string()))
    );
    let words = [
        unit_word(b'I', &[u16::from(b'1'), u16::from(b'0')], 1),
        unit_word(b'6', &[u16::from(b'k'), u16::from(b'g')], 1),
    ];
    assert_eq!(
        process_unit(&ctx, &words, 0),
        Some((1, "kilog_laM".to_string()))
    );
    let words = [
        unit_word(b'I', &[u16::from(b'1'), u16::from(b'0')], 1),
        unit_word(b'6', &[u16::from(b'%')], 1),
    ];
    assert_eq!(process_unit(&ctx, &words, 0), Some((1, "p_lo".to_string())));
}

#[test]
fn process_unit_ih6_pattern() {
    let ctx = load_ctx();
    let words = [
        unit_word(b'I', &[u16::from(b'1')], 1),
        unit_word(b'H', &[0xB144], 1),
        unit_word(b'6', &[u16::from(b'k'), u16::from(b'm')], 1),
    ];
    assert_eq!(
        process_unit(&ctx, &words, 0),
        Some((2, "kilom9te".to_string()))
    );
}

#[test]
fn process_unit_standalone_unit_word() {
    let ctx = load_ctx();
    let words = [unit_word(b'6', &[0x339E], 1)];
    assert_eq!(
        process_unit(&ctx, &words, 0),
        Some((0, "kilom9te".to_string()))
    );
    let words = [unit_word(b'6', &[u16::from(b'%')], 1)];
    assert_eq!(process_unit(&ctx, &words, 0), None);
    let words = [unit_word(b'6', &[u16::from(b'k'), u16::from(b'm')], 1)];
    assert_eq!(process_unit(&ctx, &words, 0), None);
}

#[test]
fn process_unit_no_match() {
    let ctx = load_ctx();
    let words = [
        unit_word(b'N', &[0xAC00], 1),
        unit_word(b'6', &[u16::from(b'z'), u16::from(b'z')], 1),
    ];
    assert_eq!(process_unit(&ctx, &words, 0), None);
    let words = [
        unit_word(b'I', &[u16::from(b'1')], 1),
        unit_word(b'6', &[u16::from(b'z'), u16::from(b'z')], 1),
    ];
    assert_eq!(process_unit(&ctx, &words, 0), None);
}

#[test]
fn hanja_conversion() {
    let ctx = load_ctx();
    assert_eq!(is_uni_korea_hanja(&ctx, 0x97D3), Some(0xD55C));
    assert_eq!(is_uni_korea_hanja(&ctx, 0x570B), Some(0xAD6D));
    assert_eq!(is_uni_korea_hanja(&ctx, 0x4E00), Some(0xC77C));
    assert_eq!(is_uni_korea_hanja(&ctx, 0x4E8C), Some(0xC774));
    assert_eq!(is_uni_korea_hanja(&ctx, 0x4E09), Some(0xC0BC));
    assert_eq!(is_uni_korea_hanja(&ctx, 0x4E01), Some(0xC815));
    assert_eq!(is_uni_korea_hanja(&ctx, 0x5927), Some(0xB300));
    assert_eq!(is_uni_korea_hanja(&ctx, u16::from(b'A')), None);
    assert_eq!(is_uni_korea_hanja(&ctx, 0xAC00), None);
}

#[test]
fn hanja_word_pyogi() {
    let ctx = load_ctx();
    let s = symbol_word_pyogi(&ctx, &[0x97D3, 0x570B]);
    assert_eq!(s, "haNguG");
    let s = symbol_word_pyogi(&ctx, &[0x4E00, 0x4E8C, 0x4E09]);
    assert_eq!(s, "iLisaM");
    let s = symbol_word_pyogi(&ctx, &[0x5927, 0x97D3, 0x6C11, 0x570B]);
    assert_eq!(s, "d8haNmiNguG");
}

#[test]
fn symbol_word_pyogi_mixed() {
    let ctx = load_ctx();
    assert_eq!(symbol_word_pyogi(&ctx, &[u16::from(b'+')]), "dehagi ");
    assert_eq!(symbol_word_pyogi(&ctx, &[0x30AB, 0x30BF]), "gada");
    assert_eq!(symbol_word_pyogi(&ctx, &[0x2460]), "do*g_lami iL,");
    assert_eq!(
        symbol_word_pyogi(&ctx, &[0x4E00, u16::from(b'+')]),
        "iLdehagi "
    );
    assert_eq!(symbol_word_pyogi(&ctx, &[u16::from(b'#')]), "#");
}

fn sym_word(b: u8, tag: u8) -> WordAnal {
    WordAnal {
        morphs: vec![Morph {
            cvc: vec![b],
            pos: [tag, 0],
            prob: 0.0,
            surface_len: 1,
        }],
        w_byte_num: 1,
        word_cvc: vec![b],
        source: vec![],
        b_word_sen: false,
    }
}

fn digit_word(s: &str) -> WordAnal {
    let cvc: Vec<u8> = s.bytes().collect();
    WordAnal {
        morphs: vec![Morph {
            cvc: cvc.clone(),
            pos: [b'I', 0],
            prob: 0.0,
            surface_len: 1,
        }],
        w_byte_num: cvc.len(),
        word_cvc: cvc,
        source: vec![],
        b_word_sen: false,
    }
}

fn unit_word_anal(s: &str) -> WordAnal {
    let cvc: Vec<u8> = s.bytes().collect();
    WordAnal {
        morphs: vec![Morph {
            cvc: cvc.clone(),
            pos: [b'6', 0],
            prob: 0.0,
            surface_len: 1,
        }],
        w_byte_num: cvc.len(),
        word_cvc: cvc,
        source: vec![],
        b_word_sen: false,
    }
}

#[test]
fn pronounce_plus_symbol() {
    let ctx = load_ctx();
    let out = ktts_pron::pronounce(&ctx, &[sym_word(b'+', b'S')]).unwrap();
    let pyogi: String = out
        .phoneme_codes
        .chunks(3)
        .map(kma_code::conv_cvc_to_pyogi)
        .collect();
    assert_eq!(pyogi, "dehagi");
    assert_eq!(out.syllables.len(), 3);
}

#[test]
fn pronounce_remove_symbol_word() {
    let ctx = load_ctx();
    let word = WordAnal {
        morphs: vec![Morph {
            cvc: vec![b'*', b'*'],
            pos: [b'R', 0],
            prob: 0.0,
            surface_len: 1,
        }],
        w_byte_num: 2,
        word_cvc: vec![b'*', b'*'],
        source: vec![],
        b_word_sen: false,
    };
    let out = ktts_pron::pronounce(&ctx, &[word]).unwrap();
    assert_eq!(out.syllables.len(), 1,);
    assert_eq!(out.syllables[0].word_idx, 0);
    assert_eq!(out.syllables[0].cvc, vec![b' ']);
}

#[test]
fn pronounce_digit_unit_pair() {
    let ctx = load_ctx();
    let words = vec![digit_word("10"), unit_word_anal("km")];
    let out = ktts_pron::pronounce(&ctx, &words).unwrap();
    let pyogi: String = out
        .phoneme_codes
        .chunks(3)
        .map(kma_code::conv_cvc_to_pyogi)
        .collect();
    assert_eq!(pyogi, "siBkilom9te");
    assert_eq!(out.syllables.len(), 5);
}

#[test]
fn pronounce_digit_percent_pair() {
    let ctx = load_ctx();
    let words = vec![digit_word("10"), unit_word_anal("%")];
    let out = ktts_pron::pronounce(&ctx, &words).unwrap();
    let pyogi: String = out
        .phoneme_codes
        .chunks(3)
        .map(kma_code::conv_cvc_to_pyogi)
        .collect();
    assert_eq!(pyogi, "siBp_lo");
}

#[test]
fn pronounce_asterisk_operator() {
    let ctx = load_ctx();
    let out = ktts_pron::pronounce(&ctx, &[sym_word(b'*', b'R')]).unwrap();
    let pyogi: String = out
        .phoneme_codes
        .chunks(3)
        .map(kma_code::conv_cvc_to_pyogi)
        .collect();
    assert_eq!(pyogi, "gopagi");
}

#[test]
fn set_tag_from_attribute_mapping() {
    assert_eq!(set_tag_from_attribute(0x01), b'S');
    assert_eq!(set_tag_from_attribute(0x02), b'I');
    assert_eq!(set_tag_from_attribute(0x03), b'J');
    assert_eq!(set_tag_from_attribute(0x05), b'k');
    assert_eq!(set_tag_from_attribute(0x06), b'N');
    assert_eq!(set_tag_from_attribute(0x07), b'O');
    assert_eq!(set_tag_from_attribute(0x08), b'L');
    assert_eq!(set_tag_from_attribute(0x09), b'M');
    assert_eq!(set_tag_from_attribute(0x0c), b'6');
    assert_eq!(set_tag_from_attribute(0x0d), b'R');
    assert_eq!(set_tag_from_attribute(0x00), b'R');
}
