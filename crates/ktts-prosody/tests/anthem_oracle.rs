#![expect(
    unused_crate_dependencies,
    reason = "deps used by other targets of this package"
)]
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    reason = "test fixtures: oracle values converted with intentional casts"
)]
use ktts_prosody::{PronSyllable, PronText, WordMorphs, load_prosody_dicts, prosody};

fn woman_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(
        std::env::var("KTTSDB_DIR")
            .expect("set KTTSDB_DIR to the dictionary data (kttsdb) directory"),
    )
    .join("KSpeechDic")
    .join("woman")
}

#[test]
fn anthem_1line_f0_and_length_match_oracle() {
    let ctx = load_prosody_dicts(&woman_dir())
        .expect("dictionary load failed")
        .with_birule(true);
    let text = anthem_text();
    let out = prosody(&ctx, &text).expect("prosody prediction failed");
    assert_eq!(out.len(), 36);
    for (i, t) in out.iter().enumerate() {
        let ave = [t.ave_length[0], t.ave_length[1], t.ave_length[2]];
        assert_eq!(ave, ORACLE_LEN[i], "syl {i}: ave_length");
        let codes: [u16; 12] = std::array::from_fn(|k| {
            let f = f64::from(t.f0[k]);
            if f <= 0.0 {
                0
            } else {
                (16000.0 / f + 0.5).floor() as u16
            }
        });
        assert_eq!(codes, ORACLE_PITCH[i], "syl {i}: pitch codes");
    }
    for (i, &b) in ORACLE_BND.iter().enumerate() {
        assert_eq!(out[i].boundary, b, "syl {i}: boundary (bnd)");
    }
}

fn anthem_text() -> PronText {
    PronText {
        syllables: vec![
            PronSyllable {
                cvc: [13, 3, 1],
                word_idx: 0,
                is_word_start: true,
                pos: 51,
                morph_idx: 0,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [16, 29, 1],
                word_idx: 0,
                is_word_start: false,
                pos: 51,
                morph_idx: 0,
                morph_pos: 1,
            },
            PronSyllable {
                cvc: [8, 27, 5],
                word_idx: 0,
                is_word_start: false,
                pos: 93,
                morph_idx: 1,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [9, 29, 5],
                word_idx: 1,
                is_word_start: true,
                pos: 64,
                morph_idx: 0,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [4, 3, 1],
                word_idx: 1,
                is_word_start: false,
                pos: 64,
                morph_idx: 0,
                morph_pos: 1,
            },
            PronSyllable {
                cvc: [7, 3, 1],
                word_idx: 1,
                is_word_start: false,
                pos: 103,
                morph_idx: 1,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [13, 29, 1],
                word_idx: 2,
                is_word_start: true,
                pos: 70,
                morph_idx: 0,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [2, 3, 23],
                word_idx: 3,
                is_word_start: true,
                pos: 48,
                morph_idx: 0,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [11, 3, 5],
                word_idx: 3,
                is_word_start: false,
                pos: 48,
                morph_idx: 0,
                morph_pos: 1,
            },
            PronSyllable {
                cvc: [13, 27, 5],
                word_idx: 4,
                is_word_start: true,
                pos: 48,
                morph_idx: 0,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [2, 27, 1],
                word_idx: 4,
                is_word_start: false,
                pos: 48,
                morph_idx: 0,
                morph_pos: 1,
            },
            PronSyllable {
                cvc: [8, 10, 1],
                word_idx: 4,
                is_word_start: false,
                pos: 87,
                morph_idx: 1,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [14, 3, 1],
                word_idx: 5,
                is_word_start: true,
                pos: 48,
                morph_idx: 0,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [13, 21, 5],
                word_idx: 5,
                is_word_start: false,
                pos: 48,
                morph_idx: 0,
                morph_pos: 1,
            },
            PronSyllable {
                cvc: [5, 13, 1],
                word_idx: 5,
                is_word_start: false,
                pos: 93,
                morph_idx: 1,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [2, 3, 1],
                word_idx: 6,
                is_word_start: true,
                pos: 50,
                morph_idx: 0,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [5, 27, 1],
                word_idx: 6,
                is_word_start: false,
                pos: 50,
                morph_idx: 0,
                morph_pos: 1,
            },
            PronSyllable {
                cvc: [17, 3, 5],
                word_idx: 6,
                is_word_start: false,
                pos: 68,
                morph_idx: 1,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [13, 29, 1],
                word_idx: 7,
                is_word_start: true,
                pos: 70,
                morph_idx: 0,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [11, 10, 1],
                word_idx: 8,
                is_word_start: true,
                pos: 48,
                morph_idx: 0,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [11, 3, 23],
                word_idx: 8,
                is_word_start: false,
                pos: 48,
                morph_idx: 0,
                morph_pos: 1,
            },
            PronSyllable {
                cvc: [13, 3, 1],
                word_idx: 9,
                is_word_start: true,
                pos: 50,
                morph_idx: 0,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [7, 27, 17],
                word_idx: 9,
                is_word_start: false,
                pos: 50,
                morph_idx: 0,
                morph_pos: 1,
            },
            PronSyllable {
                cvc: [5, 3, 1],
                word_idx: 9,
                is_word_start: false,
                pos: 68,
                morph_idx: 1,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [13, 20, 5],
                word_idx: 9,
                is_word_start: false,
                pos: 95,
                morph_idx: 2,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [4, 4, 1],
                word_idx: 10,
                is_word_start: true,
                pos: 70,
                morph_idx: 0,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [14, 13, 1],
                word_idx: 11,
                is_word_start: true,
                pos: 48,
                morph_idx: 0,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [2, 20, 2],
                word_idx: 11,
                is_word_start: false,
                pos: 48,
                morph_idx: 0,
                morph_pos: 1,
            },
            PronSyllable {
                cvc: [9, 3, 5],
                word_idx: 12,
                is_word_start: true,
                pos: 48,
                morph_idx: 0,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [8, 3, 5],
                word_idx: 12,
                is_word_start: false,
                pos: 48,
                morph_idx: 0,
                morph_pos: 1,
            },
            PronSyllable {
                cvc: [4, 11, 5],
                word_idx: 12,
                is_word_start: false,
                pos: 48,
                morph_idx: 0,
                morph_pos: 2,
            },
            PronSyllable {
                cvc: [13, 13, 1],
                word_idx: 13,
                is_word_start: true,
                pos: 67,
                morph_idx: 0,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [7, 4, 5],
                word_idx: 13,
                is_word_start: false,
                pos: 67,
                morph_idx: 0,
                morph_pos: 1,
            },
            PronSyllable {
                cvc: [7, 11, 2],
                word_idx: 14,
                is_word_start: true,
                pos: 48,
                morph_idx: 0,
                morph_pos: 0,
            },
            PronSyllable {
                cvc: [12, 3, 1],
                word_idx: 14,
                is_word_start: false,
                pos: 48,
                morph_idx: 0,
                morph_pos: 1,
            },
            PronSyllable {
                cvc: [13, 10, 1],
                word_idx: 14,
                is_word_start: false,
                pos: 87,
                morph_idx: 1,
                morph_pos: 0,
            },
        ],
        phoneme_codes: vec![],
        word_morphs: vec![
            WordMorphs {
                pos: vec![51, 93],
                first_chars: vec![0xca31, 0xc159],
                surfaces: vec![],
                source: vec![],
            },
            WordMorphs {
                pos: vec![64, 103],
                first_chars: vec![0xc3dd, 0xbc69],
                surfaces: vec![],
                source: vec![],
            },
            WordMorphs {
                pos: vec![70],
                first_chars: vec![0xcd09],
                surfaces: vec![],
                source: vec![],
            },
            WordMorphs {
                pos: vec![48],
                first_chars: vec![0xb103],
                surfaces: vec![],
                source: vec![],
            },
            WordMorphs {
                pos: vec![48, 87],
                first_chars: vec![0xccd5, 0xbf79],
                surfaces: vec![],
                source: vec![],
            },
            WordMorphs {
                pos: vec![48, 93],
                first_chars: vec![0xcc7d, 0xb8e9],
                surfaces: vec![],
                source: vec![],
            },
            WordMorphs {
                pos: vec![50, 68, 95],
                first_chars: vec![0xb0ed, 0xd365, 0x20],
                surfaces: vec![],
                source: vec![],
            },
            WordMorphs {
                pos: vec![70],
                first_chars: vec![0xcd09],
                surfaces: vec![],
                source: vec![],
            },
            WordMorphs {
                pos: vec![48],
                first_chars: vec![0xc65d],
                surfaces: vec![],
                source: vec![],
            },
            WordMorphs {
                pos: vec![50, 68, 95],
                first_chars: vec![0xca31, 0xb7d1, 0xcc11],
                surfaces: vec![],
                source: vec![],
            },
            WordMorphs {
                pos: vec![70],
                first_chars: vec![0xb5a1],
                surfaces: vec![],
                source: vec![],
            },
            WordMorphs {
                pos: vec![48],
                first_chars: vec![0xcd95],
                surfaces: vec![],
                source: vec![],
            },
            WordMorphs {
                pos: vec![48],
                first_chars: vec![0xc105],
                surfaces: vec![],
                source: vec![],
            },
            WordMorphs {
                pos: vec![67, 95],
                first_chars: vec![0xcb49, 0x20],
                surfaces: vec![],
                source: vec![],
            },
            WordMorphs {
                pos: vec![48, 87],
                first_chars: vec![0xbd4a, 0xcaf5],
                surfaces: vec![],
                source: vec![],
            },
        ],
        word_sen: vec![],
    }
}

const ORACLE_LEN: [[u16; 3]; 36] = [
    [0, 2356, 0],
    [2207, 757, 0],
    [945, 788, 3292],
    [1129, 799, 684],
    [745, 1176, 0],
    [569, 2746, 0],
    [0, 2548, 0],
    [1173, 900, 2126],
    [1929, 895, 2244],
    [0, 1255, 1779],
    [610, 775, 0],
    [1087, 1808, 0],
    [1926, 1295, 0],
    [0, 2197, 1709],
    [716, 2335, 0],
    [1419, 1453, 0],
    [690, 737, 0],
    [2048, 711, 1630],
    [0, 2556, 0],
    [2424, 972, 0],
    [1977, 955, 1934],
    [0, 2317, 0],
    [562, 784, 1562],
    [545, 1460, 0],
    [0, 793, 1771],
    [1010, 1813, 0],
    [1715, 1573, 0],
    [783, 2635, 0],
    [1884, 1658, 1141],
    [936, 1309, 1358],
    [590, 1530, 2244],
    [0, 2148, 0],
    [562, 1255, 2112],
    [591, 2271, 0],
    [1832, 1178, 0],
    [0, 3985, 0],
];

const ORACLE_PITCH: [[u16; 12]; 36] = [
    [97, 92, 92, 91, 91, 90, 90, 90, 89, 89, 89, 89],
    [88, 88, 88, 88, 88, 88, 88, 88, 88, 89, 89, 89],
    [89, 90, 90, 90, 90, 90, 91, 91, 91, 93, 95, 96],
    [100, 98, 97, 96, 97, 96, 97, 97, 97, 97, 97, 97],
    [97, 97, 97, 97, 97, 98, 98, 98, 98, 98, 98, 98],
    [98, 98, 98, 98, 98, 98, 98, 97, 98, 100, 101, 102],
    [102, 91, 89, 89, 89, 89, 90, 90, 90, 91, 92, 95],
    [100, 97, 95, 95, 96, 95, 95, 95, 95, 94, 94, 94],
    [94, 94, 94, 93, 93, 93, 93, 93, 94, 95, 97, 98],
    [100, 96, 94, 93, 94, 95, 95, 95, 95, 95, 96, 96],
    [96, 96, 96, 97, 97, 97, 97, 98, 98, 99, 99, 99],
    [100, 100, 101, 101, 101, 102, 102, 104, 105, 106, 108, 108],
    [104, 102, 102, 101, 101, 100, 100, 101, 101, 101, 101, 101],
    [101, 100, 100, 100, 100, 99, 99, 98, 97, 96, 96, 95],
    [94, 94, 93, 92, 92, 91, 90, 92, 95, 98, 98, 96],
    [106, 102, 100, 99, 97, 97, 97, 96, 96, 95, 95, 94],
    [94, 93, 93, 92, 91, 91, 91, 91, 91, 91, 91, 91],
    [91, 91, 91, 90, 90, 90, 90, 91, 91, 92, 93, 94],
    [95, 91, 91, 90, 90, 90, 89, 90, 90, 91, 94, 96],
    [95, 89, 88, 90, 91, 91, 92, 93, 94, 94, 95, 96],
    [97, 98, 99, 99, 100, 101, 102, 103, 104, 105, 107, 107],
    [105, 102, 100, 98, 98, 98, 98, 97, 97, 97, 97, 97],
    [97, 97, 97, 97, 96, 96, 97, 97, 97, 98, 98, 98],
    [98, 99, 99, 99, 99, 99, 99, 99, 99, 98, 98, 98],
    [98, 98, 97, 97, 97, 97, 96, 97, 98, 100, 103, 104],
    [100, 98, 99, 99, 98, 99, 100, 101, 101, 102, 104, 105],
    [98, 94, 94, 96, 97, 97, 97, 97, 97, 96, 96, 96],
    [96, 96, 95, 95, 95, 95, 95, 95, 96, 97, 99, 100],
    [101, 98, 98, 97, 98, 99, 99, 99, 99, 99, 99, 99],
    [99, 99, 99, 99, 99, 99, 99, 98, 98, 98, 97, 97],
    [97, 96, 96, 96, 95, 95, 95, 95, 95, 96, 98, 99],
    [104, 101, 99, 99, 99, 99, 99, 100, 100, 100, 100, 100],
    [101, 101, 101, 101, 102, 102, 102, 103, 103, 105, 108, 110],
    [112, 110, 110, 109, 110, 111, 111, 110, 110, 110, 110, 110],
    [110, 110, 110, 110, 110, 110, 111, 112, 114, 115, 116, 118],
    [119, 121, 122, 124, 125, 127, 129, 131, 129, 127, 124, 117],
];

const ORACLE_BND: [u8; 36] = [
    0, 0, 10, 0, 0, 20, 10, 0, 11, 0, 0, 11, 0, 0, 20, 0, 0, 10, 10, 0, 10, 0, 0, 0, 10, 10, 0, 11,
    0, 0, 10, 0, 10, 0, 0, 21,
];
