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
fn irr_b_areumdaun() {
    let c = ctx();
    let words = analyze(&c, "아름다운").expect("analyze");
    assert_eq!(words.len(), 1);
    let w = &words[0];
    assert_eq!(w.morphs.len(), 3);
    assert_eq!(w.morphs[0].pos[0], b'2');
    assert_eq!(w.morphs[1].pos[0], b'D');
    assert_eq!(w.morphs[2].pos[0], b'_');
    assert_eq!(w.morphs[0].cvc, vec![13, 3, 1, 7, 27, 17]);
    assert_eq!(w.morphs[1].cvc, vec![5, 3, 1, 13, 20, 1]);
    assert_eq!(w.morphs[2].cvc, vec![1, 1, 5]);
}

#[test]
fn irr_b_gomaun() {
    let c = ctx();
    let words = analyze(&c, "고마운").expect("analyze");
    let w = &words[0];
    assert_eq!(w.morphs.len(), 2);
    assert_eq!(w.morphs[0].pos[0], b'C');
    assert_eq!(w.morphs[1].pos[0], b'_');
    assert_eq!(w.morphs[0].cvc, vec![2, 13, 1, 8, 3, 1, 13, 20, 1]);
    assert_eq!(w.morphs[1].cvc, vec![1, 1, 5]);
}

#[test]
fn irr_b_normal_word_unchanged() {
    let c = ctx();
    let words = analyze(&c, "가득한").expect("analyze");
    let w = &words[0];
    assert_eq!(w.morphs.len(), 3);
    assert_eq!(w.morphs[0].cvc, vec![2, 3, 1, 5, 27, 2]);
    assert_eq!(w.morphs[1].cvc, vec![20, 3, 1]);
    assert_eq!(w.morphs[2].cvc, vec![1, 1, 5]);
}

#[test]
fn irr_b_spelling_input_unchanged() {
    let c = ctx();
    let words = analyze(&c, "아름답").expect("analyze");
    let w = &words[0];
    for m in &w.morphs {
        let _ = m.cvc.clone();
    }
}
