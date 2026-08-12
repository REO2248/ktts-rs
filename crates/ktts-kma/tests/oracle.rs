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
fn oracle_annyeonghaseyo() {
    let c = ctx();
    let words = analyze(&c, "안녕하세요").expect("analyze");
    assert_eq!(words.len(), 1);
    let w = &words[0];
    assert_eq!(w.morphs.len(), 3);
    assert_eq!(w.morphs[0].cvc, vec![13, 3, 5, 4, 11, 23]);
    assert_eq!(w.morphs[1].cvc, vec![20, 3, 1]);
    assert_eq!(w.morphs[2].cvc, vec![11, 10, 1, 13, 19, 1]);
    assert_eq!(w.morphs[0].pos[0], b'2');
    assert_eq!(w.morphs[1].pos[0], b'D');
    assert_eq!(w.morphs[2].pos[0], b'^');
    assert!(
        (w.morphs[0].prob - (-16.943_1)).abs() < 1e-3,
        "morph[0].prob={}",
        w.morphs[0].prob
    );
    assert!(
        (w.morphs[1].prob - (-18.231_7)).abs() < 1e-3,
        "morph[1].prob={}",
        w.morphs[1].prob
    );
    assert!(
        (w.morphs[2].prob - (-28.914_5)).abs() < 1e-3,
        "morph[2].prob={}",
        w.morphs[2].prob
    );
    assert_eq!(w.w_byte_num, 5);
    assert!(!w.b_word_sen);
}

#[test]
fn oracle_one_word() {
    let c = ctx();
    let words = analyze(&c, "하늘").expect("analyze");
    assert_eq!(words.len(), 1);
    let w = &words[0];
    assert_eq!(w.morphs.len(), 1);
    assert_eq!(w.morphs[0].pos[0], b'0');
    assert!(
        (w.morphs[0].prob - (-8.058_986)).abs() < 1e-3,
        "{}",
        w.morphs[0].prob
    );
    assert_eq!(w.morphs[0].cvc, vec![20, 3, 1, 4, 27, 9]);
}

#[test]
fn oracle_gamsa() {
    let c = ctx();
    let words = analyze(&c, "감사").expect("analyze");
    assert!(!words.is_empty());
    let w = &words[0];
    assert_eq!(w.morphs.len(), 1);
    assert_eq!(w.morphs[0].pos[0], b'0');
    assert!(
        (w.morphs[0].prob - (-13.682_782)).abs() < 1e-3,
        "{}",
        w.morphs[0].prob
    );
}

#[test]
fn oracle_digits() {
    let c = ctx();
    let words = analyze(&c, "012안").expect("analyze");
    assert!(!words.is_empty());
    let w = words.last().unwrap();
    assert_eq!(w.morphs.last().unwrap().cvc, vec![13, 3, 5]);
}

#[test]
fn sentence_symbol() {
    let c = ctx();
    let words = analyze(&c, "안녕하세요.").expect("analyze");
    assert_eq!(words.len(), 1);
    assert!(words[0].b_word_sen);
    assert_eq!(words[0].w_byte_num, 5);
    assert_eq!(words[0].morphs.len(), 3);
    let words = analyze(&c, "안녕하세요,").expect("analyze");
    assert_eq!(words.len(), 1);
    assert!(words[0].b_word_sen);
    assert_eq!(words[0].morphs.len(), 4);
}
