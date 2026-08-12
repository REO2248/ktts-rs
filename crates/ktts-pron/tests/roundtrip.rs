#![expect(
    unused_crate_dependencies,
    reason = "deps used by other targets of this package"
)]
use ktts_pron::dicts::PronContext;
use ktts_pron::kma_types::{Morph, WordAnal};

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
fn load_all_real_dicts() {
    let ctx = load_ctx();
    assert_eq!(ctx.pronrule.as_ref().unwrap().count, 43);
    assert_eq!(ctx.strpron.as_ref().unwrap().num_records(), 121);
    assert_eq!(ctx.prepron.as_ref().unwrap().num_records(), 383);
    assert_eq!(ctx.unipron.as_ref().unwrap().num_records(), 558);
    assert_eq!(ctx.morphmodify.as_ref().unwrap().num_records(), 90);
    assert_eq!(ctx.eng.unienglishpron.as_ref().unwrap().num_records(), 1061);
    assert_eq!(ctx.eng.engsym.as_ref().unwrap().num_records(), 48);
    assert_eq!(ctx.user.as_ref().unwrap().num_records(), 21);
    assert_eq!(ctx.hanja.as_ref().unwrap().table.len(), 65536);
    let eng = ctx.eng.english.as_ref().expect("englishpyogi not loaded");
    assert_eq!(eng.table().len(), 105_160);
    assert_eq!(eng.hash_dic().len(), 16_101);
    assert_eq!(eng.hash_bin().len(), 16_127);
}

#[test]
fn engsym_full_roundtrip() {
    let ctx = load_ctx();
    let d = ctx.eng.engsym.as_ref().unwrap();
    for i in 0..d.num_records() {
        let sym = d.key_string(i).expect("engsym key");
        let code = d.code(i).expect("engsym code");
        let got = ctx
            .engsym_code(sym.as_bytes())
            .unwrap_or_else(|| panic!("engsym lookup failed for {sym}"));
        assert_eq!(got, code, "engsym {sym}");
    }
    assert_eq!(ctx.engsym_code(b"h"), Some(16));
    assert_eq!(ctx.engsym_code(b"hh"), Some(16));
    assert_eq!(ctx.engsym_code(b"jh"), Some(19));
    assert_eq!(ctx.engsym_code(b"zh"), Some(39));
    assert_eq!(ctx.engsym_code(b"ia"), Some(100));
    assert_eq!(ctx.engsym_code(b"sil"), Some(103));
}

#[test]
fn unienglishpron_full_roundtrip() {
    let ctx = load_ctx();
    let d = ctx.eng.unienglishpron.as_ref().unwrap();
    for i in 0..d.num_records() {
        let key = d.key_string(i).expect("key");
        let val = d.value_string(i).expect("value");
        let got = ctx
            .unienglishpron_lookup(key.as_bytes())
            .unwrap_or_else(|| panic!("unienglishpron lookup failed for {key}"));
        assert_eq!(got, val, "unienglishpron {key}");
    }
    assert_eq!(ctx.unienglishpron_lookup(b"zero").as_deref(), Some("jilou"));
    assert_eq!(ctx.unienglishpron_lookup(b"womb").as_deref(), Some("uM"));
    assert_eq!(
        ctx.unienglishpron_lookup(b"abide").as_deref(),
        Some("ebaid_")
    );
}

#[test]
fn englishpyogi_full_roundtrip() {
    let ctx = load_ctx();
    let set = ctx.eng.english.as_ref().unwrap();
    let mut checked = 0usize;
    for b in &set.hash_dic().blocks {
        for (i, w) in b.words.iter().enumerate() {
            let expected = set
                .table()
                .pron(set.table().index_of(b.pron_offsets[i]).unwrap())
                .unwrap();
            let got = ctx.english_lookup(w).expect("englishpyogi lookup failed");
            assert_eq!(
                &got[..],
                expected,
                "englishpyogi {}",
                String::from_utf8_lossy(w)
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 105_160);
}

#[test]
fn strpron_full_roundtrip() {
    let ctx = load_ctx();
    let d = ctx.strpron.as_ref().unwrap();
    for i in 0..d.num_records() {
        let key = d.key_bytes(i).expect("key").to_vec();
        let val = d.value_string(i).expect("value");
        let key_disp: String = key
            .chunks_exact(2)
            .map(|c| char::from_u32(u32::from(u16::from_le_bytes([c[0], c[1]]))).unwrap_or('?'))
            .collect();
        let got = ctx
            .strpron_lookup(&key)
            .unwrap_or_else(|| panic!("strpron lookup failed for {key_disp}"));
        assert_eq!(got, val, "strpron {key_disp}");
    }
    assert_eq!(
        ctx.strpron_lookup(&u16le("1e/s")).as_deref(),
        Some("com8jeNjaboLt_")
    );
    assert_eq!(
        ctx.strpron_lookup(&u16le("1m3")).as_deref(),
        Some("liBba*m9te")
    );
    assert_eq!(ctx.strpron_lookup(&u16le("1hz")).as_deref(), Some("h9l_z_"));
    assert_eq!(
        ctx.strpron_lookup(&u16le("1MHz")).as_deref(),
        Some("m9gah9l_z_")
    );
    assert_eq!(
        ctx.strpron_lookup(&u16le("1GHz")).as_deref(),
        Some("gigah9l_z_")
    );
}

#[test]
fn unipron_full_roundtrip() {
    let ctx = load_ctx();
    let d = ctx.unipron.as_ref().unwrap();
    for i in 0..d.num_records() {
        let key = d.key_bytes(i).expect("key").to_vec();
        let val = d.value_string(i).expect("value");
        let got = ctx
            .unipron_lookup(&key)
            .unwrap_or_else(|| panic!("unipron lookup failed for {key:?}"));
        assert_eq!(got, val, "unipron {key:?}");
    }
    assert_eq!(ctx.unipron_lookup(&u16le("2+")).as_deref(), Some("dehagi"));
    assert_eq!(ctx.unipron_lookup(&u16le("2-")).as_deref(), Some("deLgi"));
    assert_eq!(ctx.unipron_lookup(&u16le("2/")).as_deref(), Some("nanugi"));
    assert_eq!(ctx.unipron_lookup(&u16le("3.")).as_deref(), Some("zeM"));
}

#[test]
fn user_dict_pool_scan() {
    let ctx = load_ctx();
    let entries = ctx.user_entries().expect("user entries");
    assert_eq!(entries.len(), 21);
    assert_eq!(
        entries[0],
        ("D.P.R.K".to_string(), "조선민주주의인민공화국".to_string())
    );
}

#[test]
fn prepron_morphmodify_roundtrip() {
    let ctx = load_ctx();
    let d = ctx.prepron.as_ref().unwrap();
    for i in 0..d.num_records() {
        let key = d.key_bytes(i).expect("key").to_vec();
        let code = d.code(i).expect("code");
        let got = ctx
            .prepron_code(&key)
            .unwrap_or_else(|| panic!("prepron {key:?}"));
        assert_eq!(got, code);
    }
    let d = ctx.morphmodify.as_ref().unwrap();
    for i in 0..d.num_records() {
        let key = d.key_bytes(i).expect("key").to_vec();
        let code = d.code(i).expect("code");
        let got = ctx
            .morphmodify_code(&key)
            .unwrap_or_else(|| panic!("morphmodify {key:?}"));
        assert_eq!(got, code);
    }
}

#[test]
fn hanja_samples() {
    let ctx = load_ctx();
    let h = ctx.hanja.as_ref().unwrap();
    assert_eq!(h.get(0x4E00), 0xC77C);
    assert_eq!(h.get(0x4E8C), 0xC774);
    assert_eq!(h.get(0x4E09), 0xC0BC);
    assert_eq!(h.get(0x56D7), 0xAD6D);
    assert_eq!(h.get(0xAC00), 0xAC00);
    assert_eq!(h.non_identity_cjk_count(), 20_902);
}

#[test]
fn pronounce_smoke() {
    let ctx = load_pron();
    let words = vec![
        WordAnal {
            morphs: vec![Morph {
                cvc: ktts_pron::kma_code::conv_pyogi_to_cvc(b"baNgaB"),
                pos: [b'0', 0],
                prob: 0.0,
                surface_len: 1,
            }],
            w_byte_num: 6,
            word_cvc: ktts_pron::kma_code::conv_pyogi_to_cvc(b"baNgaB"),
            source: vec![],
            b_word_sen: false,
        },
        WordAnal {
            morphs: vec![Morph {
                cvc: ktts_pron::kma_code::conv_pyogi_to_cvc(b"s9bnida"),
                pos: [b'g', 0],
                prob: 0.0,
                surface_len: 1,
            }],
            w_byte_num: 6,
            word_cvc: ktts_pron::kma_code::conv_pyogi_to_cvc(b"s9bnida"),
            source: vec![],
            b_word_sen: false,
        },
    ];
    let out = ktts_pron::pronounce(&ctx, &words).expect("pronounce failed");
    assert!(!out.syllables.is_empty());
    let mut flat = Vec::new();
    for s in &out.syllables {
        flat.extend_from_slice(&s.cvc);
    }
    assert_eq!(out.phoneme_codes, flat);
    assert!(out.syllables.iter().all(|s| s.word_idx < 2));
    let first = out.syllables.first().unwrap();
    assert!(first.is_word_start);
}

fn load_pron() -> PronContext {
    ktts_pron::load_pron_dicts(&klang_dic()).expect("load_pron_dicts failed")
}

fn u16le(s: &str) -> Vec<u8> {
    u16s(&s.encode_utf16().collect::<Vec<u16>>())
}

fn u16s(v: &[u16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 2);
    for &c in v {
        out.extend_from_slice(&c.to_le_bytes());
    }
    out
}
