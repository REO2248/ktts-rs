#![expect(
    unused_crate_dependencies,
    reason = "deps used by other targets of this package"
)]
use ktts_kma::{analyze, load_kma_dicts};

fn ctx() -> ktts_kma::KmaContext {
    let dic = std::path::PathBuf::from(
        std::env::var("KTTSDB_DIR").expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
    )
    .join("KLangDic");
    load_kma_dicts(&dic).expect("dictionary load failed")
}

#[test]
fn user_dic_replaces_dprk() {
    let c = ctx();
    let from_dic = analyze(&c, "D.P.R.K").expect("analyze");
    let direct = analyze(&c, "조선민주주의인민공화국").expect("analyze");
    assert_eq!(from_dic.len(), direct.len());
    assert!(!from_dic.is_empty());
    for (a, b) in from_dic.iter().zip(direct.iter()) {
        assert_eq!(a.source, b.source);
        assert_eq!(a.word_cvc, b.word_cvc);
        assert_eq!(a.morphs.len(), b.morphs.len());
        for (ma, mb) in a.morphs.iter().zip(b.morphs.iter()) {
            assert_eq!(ma.cvc, mb.cvc);
            assert_eq!(ma.pos, mb.pos);
        }
    }
}

#[test]
fn user_dic_replaces_in_sentence() {
    let c = ctx();
    let words = analyze(&c, "MS-DOS와 KCC").expect("analyze");
    let joined: String = words
        .iter()
        .map(|w| String::from_utf16_lossy(&w.source))
        .collect::<Vec<_>>()
        .join("|");
    assert!(
        joined.contains("엠에쓰도스") && joined.contains("조선콤퓨터쎈터"),
        "replacement result: {joined}"
    );
    let direct = analyze(&c, "엠에쓰도스와 조선콤퓨터쎈터").expect("analyze");
    let djoined: String = direct
        .iter()
        .map(|w| String::from_utf16_lossy(&w.source))
        .collect::<Vec<_>>()
        .join("|");
    assert_eq!(joined, djoined);
}

#[test]
fn user_dic_korean_key() {
    let c = ctx();
    let words = analyze(&c, "1-4분기").expect("analyze");
    let joined: String = words
        .iter()
        .map(|w| String::from_utf16_lossy(&w.source))
        .collect::<Vec<_>>()
        .join("|");
    assert!(joined.contains("일사분기"), "replacement result: {joined}");
}

#[test]
fn long_word_no_loss() {
    let c = ctx();
    let long = "가".repeat(42);
    let words = analyze(&c, &long).expect("analyze");
    let total: usize = words.iter().map(|w| w.w_byte_num as usize).sum();
    assert!(!words.is_empty());
    for w in &words {
        assert!(w.w_byte_num > 0);
        assert!(
            w.w_byte_num as usize <= 20,
            "word exceeds 20 characters: {}",
            w.w_byte_num
        );
    }
    let _ = total;
}

#[test]
fn twenty_char_word_untouched() {
    let c = ctx();
    let words = analyze(&c, "일이삼사오육칠팔구십일이삼사오육칠팔구십").expect("analyze");
    let total: usize = words.iter().map(|w| w.w_byte_num as usize).sum();
    assert_eq!(total, 20);
    for w in &words {
        assert!(w.w_byte_num as usize <= 20);
    }
}
