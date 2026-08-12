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

fn raw_words(text: &str) -> Vec<ktts_kma::ma::MaWord> {
    let c = ctx();
    let input: Vec<u16> = text.encode_utf16().collect();
    ktts_kma::ma::klp_proc_all(&c, &input).expect("klp_proc_all crashed")
}

#[test]
fn imnida_is_three_syllables() {
    let words = raw_words("전화번호는 010-1234-5678 입니다.");
    let w = words.last().expect("word list is empty");
    assert!(w.morphs.len() >= 2);
    let m0 = &w.morphs[0];
    assert_eq!(m0.ch_tag, b'@');
    assert_eq!(m0.pyogi, b"iL");
    assert_eq!(m0.cvc, [13, 29, 17]);
    let m1 = &w.morphs[1];
    assert_eq!(m1.ch_tag, b'^');
    assert_eq!(m1.pyogi, b"Bnida");
    assert_eq!(m1.cvc, [4, 29, 1, 5, 3, 1],);
    let wc = w
        .morphs
        .iter()
        .take(2)
        .flat_map(|m| m.cvc.iter().copied())
        .collect::<Vec<u8>>();
    assert_eq!(wc, [13, 29, 17, 4, 29, 1, 5, 3, 1]);
}

#[test]
fn hamnida_unchanged() {
    let words = raw_words("감사합니다");
    let w = &words[0];
    assert_eq!(w.morphs.len(), 3);
    let m2 = &w.morphs[2];
    assert_eq!(m2.pyogi, b"Bnida");
    assert_eq!(m2.cvc, [1, 1, 19, 4, 29, 1, 5, 3, 1],);
}

#[test]
fn numeral_il_plus_imnida_unchanged() {
    let words = raw_words("일입니다");
    let w = &words[0];
    assert_eq!(w.morphs.len(), 3);
    let m0 = &w.morphs[0];
    assert_ne!(m0.ch_tag, b'@');
    assert_eq!(m0.pyogi, b"iL");
    assert_eq!(m0.cvc, [13, 29, 9]);
}

#[test]
fn imnida_survives_post() {
    let c = ctx();
    let words = analyze(&c, "전화번호는 010-1234-5678 입니다.").expect("analyze crashed");
    let w = words.last().expect("word list is empty");
    assert_eq!(w.morphs[0].cvc, [13, 29, 17]);
    assert_eq!(w.morphs[1].cvc, [4, 29, 1, 5, 3, 1]);
}
