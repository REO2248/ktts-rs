#![expect(
    unused_crate_dependencies,
    reason = "deps used by other targets of this package"
)]
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
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

fn t5_text() -> PronText {
    let syl: [(u8, u8, u8, u8, u8, u8); 14] = [
        (2, 3, 1, 0, 0, 57),
        (4, 3, 1, 0, 1, 57),
        (5, 3, 1, 0, 2, 57),
        (7, 3, 1, 0, 3, 57),
        (8, 3, 1, 0, 4, 57),
        (9, 3, 1, 0, 5, 57),
        (11, 3, 1, 0, 6, 57),
        (13, 3, 1, 0, 7, 57),
        (14, 3, 1, 0, 8, 57),
        (16, 3, 1, 0, 9, 57),
        (17, 3, 1, 1, 0, 57),
        (18, 3, 1, 2, 0, 49),
        (19, 3, 1, 2, 1, 49),
        (20, 3, 1, 3, 0, 55),
    ];
    PronText {
        syllables: syl
            .iter()
            .enumerate()
            .map(|(i, &(cho, jung, jong, mi, mp, pos))| PronSyllable {
                cvc: [cho, jung, jong],
                word_idx: 0,
                is_word_start: i == 0,
                pos,
                morph_idx: mi,
                morph_pos: mp,
            })
            .collect(),
        phoneme_codes: vec![],
        word_morphs: vec![WordMorphs {
            pos: vec![57, 57, 49, 55],
            first_chars: vec![0xac00, 0xce74, 0xd0c0, 0xd558],
            surfaces: vec![],
            source: vec![],
        }],
        word_sen: vec![],
    }
}

const ORACLE_LEN: [[u16; 3]; 14] = [
    [1195, 1453, 0],
    [822, 1564, 0],
    [716, 1068, 0],
    [569, 1190, 0],
    [1170, 1564, 0],
    [716, 1360, 0],
    [1721, 1339, 0],
    [0, 1829, 0],
    [1398, 1351, 0],
    [2267, 717, 0],
    [1694, 757, 0],
    [1878, 1127, 0],
    [2067, 776, 0],
    [859, 3985, 0],
];

const ORACLE_PITCH: [[u16; 12]; 14] = [
    [97, 92, 91, 90, 90, 90, 89, 89, 88, 88, 88, 87],
    [87, 86, 86, 86, 85, 85, 85, 85, 85, 85, 85, 84],
    [84, 84, 84, 84, 84, 84, 85, 85, 85, 85, 86, 86],
    [86, 86, 87, 87, 87, 87, 87, 87, 87, 87, 88, 88],
    [88, 88, 88, 88, 88, 88, 88, 88, 89, 89, 89, 89],
    [89, 89, 89, 89, 90, 90, 90, 90, 90, 90, 90, 90],
    [90, 90, 90, 90, 90, 90, 90, 90, 90, 90, 90, 90],
    [90, 90, 90, 90, 90, 90, 91, 91, 91, 91, 92, 92],
    [92, 93, 93, 94, 94, 94, 95, 95, 95, 95, 95, 96],
    [96, 96, 96, 97, 97, 97, 97, 97, 97, 97, 98, 98],
    [98, 98, 98, 98, 99, 99, 99, 100, 100, 101, 101, 102],
    [102, 103, 104, 104, 105, 105, 105, 106, 106, 106, 106, 107],
    [107, 107, 108, 108, 109, 109, 111, 112, 114, 115, 117, 119],
    [120, 122, 124, 125, 127, 129, 131, 130, 128, 126, 125, 118],
];

#[test]
fn t5_alphabet_f0_and_length_match_oracle() {
    let ctx = load_prosody_dicts(&woman_dir())
        .expect("dictionary load failed")
        .with_birule(true);
    let text = t5_text();
    let out = prosody(&ctx, &text).expect("prosody prediction failed");
    assert_eq!(out.len(), 14);
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
}
