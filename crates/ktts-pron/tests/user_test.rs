#![expect(
    unused_crate_dependencies,
    reason = "deps used by other targets of this package"
)]
use ktts_pron::dicts::PronContext;

fn klang_dic() -> std::path::PathBuf {
    std::path::PathBuf::from(
        std::env::var("KTTSDB_DIR").expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
    )
    .join("KLangDic")
}

fn load_ctx() -> PronContext {
    PronContext::load(&klang_dic()).expect("KLangDic load failed")
}

fn u16le(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() * 2);
    for c in s.encode_utf16() {
        out.extend_from_slice(&c.to_le_bytes());
    }
    out
}

#[test]
fn user_dict_full_roundtrip() {
    let ctx = load_ctx();
    let entries = ctx.user_entries().expect("user entries");
    assert_eq!(entries.len(), 21);
    let expected: &[(&str, &str)] = &[
        ("D.P.R.K", "조선민주주의인민공화국"),
        ("1-4분기", "일사분기"),
        ("2-4분기", "이사분기"),
        ("3-4분기", "삼사분기"),
        ("4-4분기", "사사분기"),
        ("11메터", "십일메터"),
        ("MS-DOS", "엠에쓰도스"),
        ("ㅌ.ㄷ", "트드"),
        ("B.C.", "기원전"),
        ("FIFA", "국제축구련맹"),
        ("bulb", "분기부"),
        ("A.C.", "기원후"),
        ("i486", "아이사팔륙"),
        ("ISO", "이써"),
        ("C++", "씨쁠라스 쁠라스"),
        ("S/W", "쏘프트웨어"),
        ("례:", "례,"),
        ("KCC", "조선콤퓨터쎈터"),
        ("sp", "에쓰피"),
        ("?-", "이란"),
        ("<5027>", "오공이칠"),
    ];
    for (i, (k, v)) in expected.iter().enumerate() {
        assert_eq!(&entries[i].0, k, "user key {i}");
        assert_eq!(&entries[i].1, v, "user value {i} ({k})");
    }
    assert_eq!(
        ctx.user_lookup(&u16le("D.P.R.K"))
            .map(|(_, v)| v)
            .as_deref(),
        Some("조선민주주의인민공화국")
    );
    assert_eq!(
        ctx.user_lookup(&u16le("FIFA컵")).map(|(_, v)| v).as_deref(),
        Some("국제축구련맹")
    );
    assert_eq!(
        ctx.user_lookup(&u16le("KCC")).map(|(_, v)| v).as_deref(),
        Some("조선콤퓨터쎈터")
    );
    assert_eq!(
        ctx.user_lookup(&u16le("MS-DOS")).map(|(_, v)| v).as_deref(),
        Some("엠에쓰도스")
    );
}
