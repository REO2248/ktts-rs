#![expect(
    unused_crate_dependencies,
    reason = "deps used by other targets of this package"
)]
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "test fixtures: oracle values converted with intentional casts"
)]
use ktts_synth::{context, pron::PronText, prosody::SyllableTarget};

fn speech_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(
        std::env::var("KTTSDB_DIR").expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
    )
    .join("KSpeechDic")
    .join("woman")
}

fn synth_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(
        std::env::var("KTTSDB_DIR").expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
    )
    .join("KSpeechDic")
}

fn syl(cvc: [u8; 3], word_idx: usize) -> ktts_synth::pron::PronSyllable {
    ktts_synth::pron::PronSyllable {
        cvc: String::from_utf8(cvc.to_vec()).unwrap(),
        word_idx,
        is_word_start: word_idx == 0,
        pos: b'0',
    }
}

fn hello_pron() -> PronText {
    let raw: [[u8; 3]; 5] = [
        [13, 3, 5],
        [4, 11, 23],
        [20, 3, 1],
        [13, 10, 1],
        [13, 14, 1],
    ];
    PronText {
        syllables: raw
            .iter()
            .enumerate()
            .map(|(i, &c)| syl(c, i / 2))
            .collect(),
        phoneme_codes: vec![],
        word_sen: vec![],
    }
}

fn rms(pcm: &[i16]) -> f64 {
    if pcm.is_empty() {
        return 0.0;
    }
    let sum: f64 = pcm.iter().map(|&s| f64::from(s) * f64::from(s)).sum();
    (sum / pcm.len() as f64).sqrt()
}

#[test]
fn prosody_to_synth_ave_length_chain() {
    let pctx = ktts_prosody::load_prosody_dicts(&speech_dir()).expect("prosody dictionary");
    let raw_cvc: [[u8; 3]; 5] = [
        [13, 3, 5],
        [4, 11, 23],
        [20, 3, 1],
        [13, 10, 1],
        [13, 14, 1],
    ];
    let pp = ktts_prosody::PronText {
        syllables: raw_cvc
            .iter()
            .enumerate()
            .map(|(i, &c)| ktts_prosody::PronSyllable {
                cvc: c,
                word_idx: i / 2,
                is_word_start: i % 2 == 0,
                pos: b'0',
                morph_idx: (i / 2) as u8,
                morph_pos: (i % 2) as u8,
            })
            .collect(),
        phoneme_codes: vec![],
        word_morphs: vec![],
        word_sen: vec![],
    };
    let targets = ktts_prosody::prosody(&pctx, &pp).expect("prosody()");
    assert_eq!(targets.len(), 5);

    for (i, t) in targets.iter().enumerate() {
        let sum: u32 = t.ave_length.iter().map(|&v| u32::from(v)).sum();
        assert!(
            (sum as f32 / 16.0 - t.dur).abs() < 1.0,
            "[{i}] ave_length {:?} and dur {} are inconsistent",
            t.ave_length,
            t.dur
        );
        for &v in &t.ave_length {
            assert!(v <= 32767, "[{i}] ave_length too large: {v}");
        }
    }
    assert_eq!(targets[0].ave_length[0], 0);
    assert!(targets[0].ave_length[1] > 0 && targets[0].ave_length[2] > 0);
    assert!(targets[2].ave_length[0] > 0);
    assert_eq!(targets[2].ave_length[2], 0);

    let mut sctx = ktts_synth::load_synth_db(&synth_dir(), "woman").expect("synth DB");
    let st: Vec<SyllableTarget> = targets
        .iter()
        .map(|t| SyllableTarget {
            dur: t.dur,
            ave_length: t.ave_length,
            f0: t.f0,
            tobi: t.tobi,
            boundary: t.boundary,
        })
        .collect();
    let pcm = ktts_synth::synthesize(&sctx, &hello_pron(), &st).expect("synthesis");
    assert!(!pcm.is_empty());
    assert!(rms(&pcm) > 0.0);
    let peak = pcm.iter().map(|&s| s.abs()).max().unwrap_or(0);
    assert!(peak > 500, "peak {peak} too small");

    let phrase = context::build_phrase(&hello_pron(), &st).expect("build_phrase");
    for (i, l) in phrase.letters.iter().enumerate() {
        assert_eq!(
            l.ave_length, st[i].ave_length,
            "letter[{i}] ave_length does not match target"
        );
    }
    let _ = &mut sctx;
}
