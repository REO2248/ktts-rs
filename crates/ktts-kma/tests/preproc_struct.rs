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

fn check_no_empty(words: &[ktts_kma::WordAnal]) {
    assert!(!words.is_empty());
    for w in words {
        assert!(w.w_byte_num > 0);
        assert!(!w.morphs.is_empty());
    }
}

fn check_digit_cvc(words: &[ktts_kma::WordAnal]) -> usize {
    let mut n = 0;
    for w in words {
        for m in &w.morphs {
            if m.pos[0] == b'H' {
                assert!(!m.cvc.is_empty());
                n += 1;
            }
        }
    }
    assert!(n > 0);
    n
}

#[test]
fn mixed_korean_digit_korean() {
    let c = ctx();
    for text in [
        "제17차",
        "제17차 전국프로그람경연",
        "95년 10월 17차 1등",
        "전화번호 010-1234-5678",
        "오늘은 3시 25분입니다",
        "1995년 10월 17일",
    ] {
        let words = analyze(&c, text).expect("analyze crashed");
        check_no_empty(&words);
        check_digit_cvc(&words);
    }
}

#[test]
fn structural_corpus() {
    let c = ctx();
    let corpus = [
        "1995년 10월 17일",
        "95년 10월 17차 1등",
        "제17차 전국프로그람경연",
        "오늘은 3시 25분입니다",
        "전화번호 010-1234-5678",
        "12월 25일은 3시 30분에 시작합니다",
        "10m 떨어진 곳에 5kg 상자가 있다",
        "가격은 1,500원 입니다",
        "3.14는 원주율입니다",
        "1995.10.17",
        "2000년 1월 1일 0시 0분",
        "0.5리터",
        "주체94(2005)년 4월 15일",
        "8.15해방절과 6.25전쟁",
        "100% 성과를 거두었다",
        "안녕하세요. 오늘은 날씨가 좋습니다. 내일은 비가 올 것 같습니다.",
        "ABC는 알파벳입니다",
        "10시 25분 30초에 출발합니다",
        "第17次",
        "010-1234-5678",
        "95.5점을 받았다",
    ];
    for text in corpus {
        let words = analyze(&c, text).unwrap_or_else(|e| panic!("analyze crashed: {text}: {e}"));
        check_no_empty(&words);
    }
}

#[test]
fn digit_reading_is_korean() {
    let c = ctx();
    let words = analyze(&c, "제17차").expect("analyze");
    let mut found = false;
    for w in &words {
        for m in &w.morphs {
            if m.pos[0] == b'H' {
                assert!(!m.cvc.is_empty());
                found = true;
            }
        }
    }
    assert!(found);
}

#[test]
fn sentence_split_no_gaps() {
    let c = ctx();
    let text = "안녕하세요. 오늘은 좋은 날입니다! 내일은 어떨까요?";
    let words = analyze(&c, text).expect("analyze");
    check_no_empty(&words);
    let sen = words.iter().filter(|w| w.b_word_sen).count();
    assert_eq!(
        sen, 3,
        "number of sentence-ending marks: {sen} (expected 3)"
    );
    let mut joined = String::new();
    for w in &words {
        for m in &w.morphs {
            let wan = ktts_kma::code::conv_cvc_to_uni_wan(&m.cvc);
            joined.push_str(&String::from_utf16_lossy(&wan));
        }
    }
    for key in ["안녕하세요", "오늘", "좋은", "내일"] {
        assert!(
            joined.contains(key),
            "missed: {key} not found in {joined:?}"
        );
    }
}

#[test]
fn newline_sentence_split_marks_sentence_end() {
    let c = ctx();
    let text = "아침은 빛나라 이 강산\n은금에 자원도 가득한";
    let words = analyze(&c, text).expect("analyze");
    assert_eq!(words.len(), 7, "word count 4+3=7: {words:?}");
    let ends: Vec<usize> = words
        .iter()
        .enumerate()
        .filter(|(_, w)| w.b_sentence_end)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(ends, vec![3, 6], "sentence-end flag positions: {ends:?}");
    assert_eq!(String::from_utf16_lossy(&words[3].source), "강산",);
    assert_eq!(String::from_utf16_lossy(&words[6].source), "가득한",);
    assert!(!words[0].b_sentence_end && !words[1].b_sentence_end && !words[2].b_sentence_end);
    assert!(!words[4].b_sentence_end && !words[5].b_sentence_end);
    let words = analyze(&c, "아침은 빛나라 이 강산").expect("analyze");
    let ends: Vec<usize> = words
        .iter()
        .enumerate()
        .filter(|(_, w)| w.b_sentence_end)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(ends, vec![3], "no newline: only the last word: {ends:?}");
    let words = analyze(&c, "\n\n아침은\n\n은금에 자원도\n").expect("analyze");
    let ends: Vec<usize> = words
        .iter()
        .enumerate()
        .filter(|(_, w)| w.b_sentence_end)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        ends,
        vec![0, 2],
        "sentence-end positions after skipping blank lines: {ends:?}"
    );
    assert_eq!(String::from_utf16_lossy(&words[0].source), "아침은");
    assert_eq!(String::from_utf16_lossy(&words[2].source), "자원도");
}

#[test]
fn digit_merge_acceptance_strings() {
    use ktts_kma::ma;
    let c = ctx();
    for text in [
        "95년",
        "1등",
        "제17차",
        "010-1234",
        "010-1234-5678",
        "1,234",
        "123,456,789",
    ] {
        let input: Vec<u16> = text.encode_utf16().collect();
        let words = ma::klp_proc_all(&c, &input)
            .unwrap_or_else(|e| panic!("klp_proc_all crashed: {text}: {e}"));
        assert!(!words.is_empty(), "{text}: word list is empty");
        let has_h = words.iter().any(|w| {
            w.morphs.iter().any(|m| m.ch_tag == b'H')
                || (w.b_str_type == 3 && w.morphs.first().map(|m| m.ch_tag) == Some(b'H'))
        });
        assert!(
            has_h,
            "{text}: no digit-reading morph (H) was generated: {:?}",
            words
                .iter()
                .map(|w| String::from_utf16_lossy(&w.source))
                .collect::<Vec<_>>()
        );
    }
    for text in ["1,234", "123,456,789"] {
        let input: Vec<u16> = text.encode_utf16().collect();
        let words = ma::klp_proc_all(&c, &input).expect("analyze");
        let srcs: Vec<String> = words
            .iter()
            .map(|w| String::from_utf16_lossy(&w.source))
            .collect();
        assert!(
            !srcs.iter().any(|s| s == text),
            "{text}: comma-separated digits remain unmerged: {srcs:?}"
        );
    }
}
