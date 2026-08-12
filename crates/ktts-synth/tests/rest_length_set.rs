#![expect(
    unused_crate_dependencies,
    reason = "deps used by other targets of this package"
)]
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    reason = "test fixtures: oracle values converted with intentional casts"
)]
use ktts_synth::pron::{PronSyllable, PronText};
use ktts_synth::prosody::SyllableTarget;

fn data_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(
        std::env::var("KTTSDB_DIR").expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
    )
}

fn ctx() -> ktts_synth::SynthContext {
    ktts_synth::load_synth_db(&data_dir().join("KSpeechDic"), "woman").expect("load_synth_db")
}

fn zero_runs(pcm: &[i16]) -> Vec<usize> {
    let mut runs = Vec::new();
    let mut run = 0usize;
    for &s in pcm {
        if s == 0 {
            run += 1;
        } else {
            if run > 0 {
                runs.push(run);
            }
            run = 0;
        }
    }
    if run > 0 {
        runs.push(run);
    }
    runs
}

fn tgt(dur: f32, boundary: u8) -> SyllableTarget {
    SyllableTarget {
        dur,
        ave_length: [
            (dur * 16.0 * 0.3) as u16,
            (dur * 16.0 * 0.5) as u16,
            (dur * 16.0 * 0.2) as u16,
        ],
        f0: [200.0; 12],
        tobi: 0.0,
        boundary,
    }
}

fn anthem_text() -> ktts_prosody::PronText {
    type SylSpec = (u8, u8, u8, usize, bool, u8, u8, u8);
    let s: &[SylSpec] = &[
        (13, 3, 1, 0, true, 51, 0, 0),
        (16, 29, 1, 0, false, 51, 0, 1),
        (8, 27, 5, 0, false, 93, 1, 0),
        (9, 29, 5, 1, true, 64, 0, 0),
        (4, 3, 1, 1, false, 64, 0, 1),
        (7, 3, 1, 1, false, 103, 1, 0),
        (13, 29, 1, 2, true, 70, 0, 0),
        (2, 3, 23, 3, true, 48, 0, 0),
        (11, 3, 5, 3, false, 48, 0, 1),
        (13, 27, 5, 4, true, 48, 0, 0),
        (2, 27, 1, 4, false, 48, 0, 1),
        (8, 10, 1, 4, false, 87, 1, 0),
        (14, 3, 1, 5, true, 48, 0, 0),
        (13, 21, 5, 5, false, 48, 0, 1),
        (5, 13, 1, 5, false, 93, 1, 0),
        (2, 3, 1, 6, true, 50, 0, 0),
        (5, 27, 1, 6, false, 50, 0, 1),
        (17, 3, 5, 6, false, 68, 1, 0),
        (13, 29, 1, 7, true, 70, 0, 0),
        (11, 10, 1, 8, true, 48, 0, 0),
        (11, 3, 23, 8, false, 48, 0, 1),
        (13, 3, 1, 9, true, 50, 0, 0),
        (7, 27, 17, 9, false, 50, 0, 1),
        (5, 3, 1, 9, false, 68, 1, 0),
        (13, 20, 5, 9, false, 68, 2, 0),
        (4, 4, 1, 10, true, 70, 0, 0),
        (14, 13, 1, 11, true, 48, 0, 0),
        (2, 20, 2, 11, false, 48, 0, 1),
        (9, 3, 5, 12, true, 48, 0, 0),
        (8, 3, 5, 12, false, 48, 0, 1),
        (4, 11, 5, 12, false, 48, 0, 2),
        (13, 13, 1, 13, true, 67, 0, 0),
        (7, 4, 5, 13, false, 67, 0, 1),
        (7, 11, 2, 14, true, 48, 0, 0),
        (12, 3, 1, 14, false, 48, 0, 1),
        (13, 10, 1, 14, false, 87, 1, 0),
    ];
    let syllables = s
        .iter()
        .map(
            |&(c0, c1, c2, w, start, pos, mi, mp)| ktts_prosody::PronSyllable {
                cvc: [c0, c1, c2],
                word_idx: w,
                is_word_start: start,
                pos,
                morph_idx: mi,
                morph_pos: mp,
            },
        )
        .collect();
    let phoneme_codes: Vec<u8> = s
        .iter()
        .flat_map(|&(c0, c1, c2, _, _, _, _, _)| [c0, c1, c2])
        .collect();
    let word_morphs = vec![
        (vec![b'3', b']'], vec![0xc544u16, 0xc740]),
        (vec![b'@', b'g'], vec![0xbe5b, 0xb77c]),
        (vec![b'F'], vec![0xc774]),
        (vec![b'0'], vec![0xac15]),
        (vec![b'0', b'W'], vec![0xc740, 0xc5d0]),
        (vec![b'0', b']'], vec![0xc790, 0xb3c4]),
        (vec![b'2', b'D', b'_'], vec![0xac00, 0xd558, 0x3134]),
        (vec![b'F'], vec![0xc774]),
        (vec![b'0'], vec![0xc138]),
        (vec![b'2', b'D', b'_'], vec![0xc544, 0xb2f5, 0x3134]),
        (vec![b'F'], vec![0xb0b4]),
        (vec![b'0'], vec![0xc870]),
        (vec![b'0'], vec![0xbc18]),
        (vec![b'C', b'_'], vec![0xc624, 0x3134]),
        (vec![b'0', b'W'], vec![0xb825, 0xc5d0]),
    ]
    .into_iter()
    .map(|(pos, first_chars)| ktts_prosody::WordMorphs {
        pos,
        first_chars,
        surfaces: vec![],
        source: vec![],
    })
    .collect();
    ktts_prosody::PronText {
        syllables,
        phoneme_codes,
        word_morphs,
        word_sen: vec![],
    }
}

fn to_synth_text(p: &ktts_prosody::PronText) -> PronText {
    PronText {
        syllables: p
            .syllables
            .iter()
            .map(|s| PronSyllable {
                cvc: s.cvc.iter().map(|&b| b as char).collect(),
                word_idx: s.word_idx,
                is_word_start: s.is_word_start,
                pos: s.pos,
            })
            .collect(),
        phoneme_codes: p.phoneme_codes.clone(),
        word_sen: p.word_sen.clone(),
    }
}

#[test]
fn anthem_rest_structure_matches_oracle() {
    let text = anthem_text();
    assert_eq!(text.syllables.len(), 36);
    let pctx = ktts_prosody::load_prosody_dicts(&data_dir().join("KSpeechDic").join("woman"))
        .expect("load_prosody_dicts");
    let targets = ktts_prosody::prosody(&pctx, &text).expect("prosody");
    assert_eq!(targets.len(), 36);
    let word_end: [usize; 15] = [2, 5, 6, 8, 11, 14, 17, 18, 20, 24, 25, 27, 30, 32, 35];
    let expected_bnd: [u8; 15] = [10, 20, 10, 11, 11, 20, 10, 10, 10, 10, 10, 11, 10, 10, 21];
    for (i, &e) in expected_bnd.iter().enumerate() {
        assert_eq!(
            targets[word_end[i]].boundary, e,
            "word {i} word-end boundary value does not match the original engine's bBreakInfo"
        );
    }
    let tgts: Vec<SyllableTarget> = targets
        .into_iter()
        .map(|t| SyllableTarget {
            dur: t.dur,
            ave_length: t.ave_length,
            f0: t.f0,
            tobi: t.tobi,
            boundary: t.boundary,
        })
        .collect();
    let s_text = to_synth_text(&text);
    let c = ctx();
    let pcm = ktts_synth::synthesize(&c, &s_text, &tgts).expect("synthesis");
    let runs = zero_runs(&pcm);
    let sent_end = runs.last().copied().unwrap_or(0);
    assert!(
        (10000..=12000).contains(&sent_end),
        "sentence-end rest {sent_end} (expected 11000). All zero runs: {runs:?}"
    );
    let mut internal: Vec<usize> = runs[..runs.len() - 1].to_vec();
    internal.retain(|&r| r >= 900);
    internal.sort_unstable();
    let expected: Vec<usize> = vec![1000, 1000, 1000, 4200, 4200];
    assert_eq!(
        internal.len(),
        expected.len(),
        "the number of internal rests in the anthem does not match the original engine's measurement. All zero runs: {runs:?}"
    );
    for (i, (&got, &exp)) in internal.iter().zip(expected.iter()).enumerate() {
        assert!(
            (got as isize - exp as isize).abs() <= 2,
            "internal rest {i} is {got} instead of expected {exp}. All zero runs: {runs:?}"
        );
    }
}

fn internal_runs(syllables: &[(u8, u8, u8)], word1_len: usize, bnd: u8) -> Vec<usize> {
    let mut text_syl = Vec::new();
    let mut phoneme_codes = Vec::new();
    for (i, &(c0, c1, c2)) in syllables.iter().enumerate() {
        text_syl.push(PronSyllable {
            cvc: format!("{}{}{}", c0 as char, c1 as char, c2 as char),
            word_idx: usize::from(i >= word1_len),
            is_word_start: i == 0 || i == word1_len,
            pos: b'0',
        });
        phoneme_codes.extend_from_slice(&[c0, c1, c2]);
    }
    let text = PronText {
        syllables: text_syl,
        phoneme_codes,
        word_sen: vec![],
    };
    let mut targets: Vec<SyllableTarget> = syllables
        .iter()
        .enumerate()
        .map(|(i, _)| tgt(150.0, if i == word1_len - 1 { bnd } else { 0 }))
        .collect();
    let n = targets.len();
    targets[n - 1].boundary = 0x15;
    let c = ctx();
    let pcm = ktts_synth::synthesize(&c, &text, &targets).expect("synthesis");
    let runs = zero_runs(&pcm);
    let mut internal: Vec<usize> = runs[..runs.len().saturating_sub(1)].to_vec();
    internal.retain(|&r| r >= 900);
    internal
}

#[test]
fn jong_connected_unmasked_cho_gets_1000() {
    let runs = internal_runs(&[(14, 13, 1), (2, 20, 2), (20, 3, 1), (4, 3, 1)], 2, 0x0b);
    assert!(
        runs.iter().any(|&r| (900..=1300).contains(&r)),
        "조국|하나 must get a 1000 rest (fJong==-2 + cho >= 0x14 → 1000). Internal rests: {runs:?}"
    );
}

#[test]
fn jong_connected_masked_cho_no_rest() {
    let runs = internal_runs(&[(14, 13, 1), (2, 20, 2), (7, 3, 1), (8, 11, 5)], 2, 0x0b);
    assert!(
        !runs.iter().any(|&r| (900..=1300).contains(&r)),
        "조국|라면 must not get a rest (cho 7 in mask 0x2192 → 0). Internal rests: {runs:?}"
    );
}

#[test]
fn non_connected_jong_no_rest() {
    let runs = internal_runs(&[(2, 3, 23), (11, 3, 5), (20, 3, 1), (4, 3, 1)], 2, 0x0b);
    assert!(
        !runs.iter().any(|&r| (900..=1300).contains(&r)),
        "강산|하나 must not get a rest (fJong=1 → wAveLength 0). Internal rests: {runs:?}"
    );
}
