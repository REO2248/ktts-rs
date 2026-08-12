#![expect(
    unused_crate_dependencies,
    reason = "deps used by other targets of this package"
)]
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::suboptimal_flops,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::float_cmp,
    reason = "test fixtures: oracle values converted with intentional casts"
)]
use ktts_synth::context;
use ktts_synth::pron::{PronSyllable, PronText};
use ktts_synth::prosody::SyllableTarget;

fn data_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(
        std::env::var("KTTSDB_DIR").expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
    )
    .join("KSpeechDic")
}

fn ctx() -> ktts_synth::SynthContext {
    ktts_synth::load_synth_db(&data_dir(), "woman").expect("load_synth_db")
}

fn syl(cvc: &str, word: usize, start: bool) -> PronSyllable {
    PronSyllable {
        cvc: cvc.to_string(),
        word_idx: word,
        is_word_start: start,
        pos: 0,
    }
}

fn tgt_for(cvc: &str, dur: f32, f0: f32) -> SyllableTarget {
    let total = dur * 16.0;
    let bytes: Vec<u8> = cvc.bytes().collect();
    let has_cho = bytes
        .first()
        .is_some_and(|&b| b != 0 && b != 1 && b != 0x0d);
    let has_jong = bytes.get(2).is_some_and(|&b| b != 0 && b != 1);
    let mut out = [0u16; 3];
    if has_cho {
        out[0] = (total * 0.3 + 0.5) as u16;
    }
    out[1] = (total * (0.5 + if has_cho { 0.0 } else { 0.3 } + if has_jong { 0.0 } else { 0.2 })
        + 0.5) as u16;
    if has_jong {
        out[2] = (total * 0.2 + 0.5) as u16;
    }
    SyllableTarget {
        dur,
        ave_length: out,
        f0: [f0; 12],
        tobi: 0.0,
        boundary: 0,
    }
}

fn voiced_len(pcm: &[i16]) -> usize {
    let mut first = pcm.len();
    let mut last = 0usize;
    for (i, &s) in pcm.iter().enumerate() {
        if s != 0 {
            if i < first {
                first = i;
            }
            last = i;
        }
    }
    if last >= first && last != 0 {
        last - first + 1
    } else {
        0
    }
}

fn check_audio(pcm: &[i16], target_dur_ms: f32, tolerance_ms: f32) {
    assert!(!pcm.is_empty());
    let total_ms = pcm.len() as f32 / 16.0;
    let v = voiced_len(pcm);
    assert!(v > 0);
    let voiced_ms = v as f32 / 16.0;
    assert!(
        (voiced_ms - target_dur_ms).abs() <= tolerance_ms,
        "voiced length {voiced_ms}ms deviates from target {target_dur_ms}ms ± {tolerance_ms}ms (total {total_ms}ms)"
    );
    let peak = pcm.iter().map(|&s| s.abs()).max().unwrap_or(0);
    assert!(peak <= 32000, "peak {peak} exceeds ±32000");
    assert!(peak > 1000, "peak {peak} too small (nearly silent)");
}

#[test]
fn synth_db_load_and_first_unit() {
    let c = ctx();
    let db = &c;
    let _ = db;
    assert_eq!(ktts_dict::synthdb::uraw_to_pcm(0xc1u8 as i8), 1820);
    let rec0 = &c.db_ref().idx.units[0].records[0];
    let pcm = c.db_ref().pcm_segment(rec0).expect("first record decode");
    assert_eq!(pcm.len(), rec0.w_pcm_size as usize);
    let peak = pcm.iter().map(|&s| s.abs()).max().unwrap_or(0);
    assert!(peak > 1000, "peak {peak} of first unit too small");
}

#[test]
fn single_vowel_a_fast_path() {
    let text = PronText {
        syllables: vec![syl("\x0d\x03\x01", 0, true)],
        phoneme_codes: vec![0x0d, 0x03, 0x01],
        word_sen: vec![],
    };
    let targets = vec![tgt_for("\x0d\x03\x01", 150.0, 200.0)];
    let c = ctx();
    let pcm = ktts_synth::synthesize(&c, &text, &targets).expect("synthesis");
    assert_eq!(c.params().pitch, 1.0);
    assert_eq!(c.params().speed, 1.0);
    check_audio(&pcm, 130.0, 70.0);
    let total_ms = pcm.len() as f32 / 16.0;
    let v = voiced_len(&pcm) as f32 / 16.0;
    assert!(
        (total_ms - (v + 687.0)).abs() < 60.0,
        "total {total_ms}ms does not match voiced {v}ms + sentence-end rest 687ms"
    );
}

#[test]
fn single_vowel_a_slow_path_pitch() {
    let text = PronText {
        syllables: vec![syl("\x0d\x03\x01", 0, true)],
        phoneme_codes: vec![0x0d, 0x03, 0x01],
        word_sen: vec![],
    };
    let targets = vec![tgt_for("\x0d\x03\x01", 150.0, 200.0)];
    let mut c = ctx();
    c.set_pitch(200);
    assert!(c.params().pitch != 1.0);
    let pcm = ktts_synth::synthesize(&c, &text, &targets).expect("synthesis (slow path)");
    check_audio(&pcm, 150.0, 60.0);
}

#[test]
fn single_vowel_a_slow_path_speed() {
    let text = PronText {
        syllables: vec![syl("\x0d\x03\x01", 0, true)],
        phoneme_codes: vec![0x0d, 0x03, 0x01],
        word_sen: vec![],
    };
    let targets = vec![tgt_for("\x0d\x03\x01", 150.0, 200.0)];
    let mut c = ctx();
    c.set_speed(150);
    assert!(c.params().speed != 1.0);
    let pcm = ktts_synth::synthesize(&c, &text, &targets).expect("synthesis (slow path)");
    check_audio(&pcm, 100.0, 60.0);
}

#[test]
fn two_syllable_word() {
    let text = PronText {
        syllables: vec![syl("\x0d\x03\x05", 0, true), syl("\x0d\x0b\x17", 0, false)],
        phoneme_codes: vec![0x0d, 0x03, 0x05, 0x0d, 0x0b, 0x17],
        word_sen: vec![],
    };
    let targets = vec![
        tgt_for("\x0d\x03\x05", 120.0, 210.0),
        tgt_for("\x0d\x0b\x17", 140.0, 190.0),
    ];
    let c = ctx();
    let pcm = ktts_synth::synthesize(&c, &text, &targets).expect("synthesis");
    let v = voiced_len(&pcm) as f32 / 16.0;
    assert!(
        v > 150.0,
        "voiced length of 2 syllables {v}ms (target 260ms)"
    );
    assert!(
        v < 800.0,
        "voiced length of 2 syllables {v}ms is abnormally long"
    );
    let peak = pcm.iter().map(|&s| s.abs()).max().unwrap_or(0);
    assert!(peak > 1000 && peak <= 32000, "peak {peak}");
}

#[test]
fn word_boundary_rest() {
    let text = PronText {
        syllables: vec![syl("\x0d\x03\x05", 0, true), syl("\x0d\x0b\x17", 1, true)],
        phoneme_codes: vec![0x0d, 0x03, 0x05, 0x0d, 0x0b, 0x17],
        word_sen: vec![],
    };
    let targets = vec![
        tgt_for("\x0d\x03\x05", 100.0, 200.0),
        tgt_for("\x0d\x0b\x17", 100.0, 200.0),
    ];
    let c = ctx();
    let pcm = ktts_synth::synthesize(&c, &text, &targets).expect("synthesis");
    let mut runs: Vec<usize> = Vec::new();
    let mut run = 0usize;
    for &s in &pcm {
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
    let sent_end = runs.pop().unwrap_or(0);
    assert!(
        (9000..=13000).contains(&sent_end),
        "sentence-end rest zero run {sent_end} samples (expected 11000)"
    );
    let internal_max = runs.iter().copied().max().unwrap_or(0);
    if internal_max >= 900 {
        assert!(
            (1000..=1200).contains(&internal_max) || (4100..=4300).contains(&internal_max),
            "word-boundary rest zero run {internal_max} samples (expected 1000 or 4200)"
        );
    }
    let v = voiced_len(&pcm) as f32 / 16.0;
    assert!(
        v > 120.0,
        "voiced length {v}ms too short (both words must be synthesized)"
    );
    assert!(v < 1200.0, "voiced length {v}ms is abnormally long");
}

#[test]
fn long_sentence() {
    let text = PronText {
        syllables: vec![
            syl("\x0d\x03\x05", 0, true),
            syl("\x0d\x0b\x17", 0, false),
            syl("\x14\x03\x17", 0, false),
            syl("\x0b\x0a", 0, false),
            syl("\x0d\x13", 1, true),
            syl("\x02\x03\x11", 2, true),
            syl("\x0b\x03", 2, false),
            syl("\x14\x03\x13", 2, false),
            syl("\x04\x1d", 2, false),
            syl("\x05\x03", 2, false),
        ],
        phoneme_codes: vec![
            0x0d, 0x03, 0x05, 0x0d, 0x0b, 0x17, 0x14, 0x03, 0x17, 0x0b, 0x0a, 0x01, 0x0d, 0x13,
            0x01, 0x02, 0x03, 0x11, 0x0b, 0x03, 0x01, 0x14, 0x03, 0x13, 0x04, 0x1d, 0x01, 0x05,
            0x03, 0x01,
        ],
        word_sen: vec![],
    };
    let cvcs: [&str; 10] = [
        "\x0d\x03\x05",
        "\x0d\x0b\x17",
        "\x14\x03\x17",
        "\x0b\x0a",
        "\x0d\x13",
        "\x02\x03\x11",
        "\x0b\x03",
        "\x14\x03\x13",
        "\x04\x1d",
        "\x05\x03",
    ];
    let targets: Vec<SyllableTarget> = (0..10)
        .map(|i| tgt_for(cvcs[i], 120.0 - i as f32 * 2.0, 210.0 - i as f32 * 3.0))
        .collect();
    let c = ctx();
    let pcm = ktts_synth::synthesize(&c, &text, &targets).expect("synthesis");
    let v = voiced_len(&pcm) as f32 / 16.0;
    assert!(v > 900.0, "voiced length of 10 syllables {v}ms");
    assert!(!pcm.is_empty());
}

#[test]
fn psola_slow_path_output_differs_from_fast() {
    let text = PronText {
        syllables: vec![syl("\x0d\x03\x05", 0, true)],
        phoneme_codes: vec![0x0d, 0x03, 0x05],
        word_sen: vec![],
    };
    let targets = vec![tgt_for("\x0d\x03\x05", 150.0, 200.0)];
    let c = ctx();
    let fast = ktts_synth::synthesize(&c, &text, &targets).expect("fast path");
    let mut c2 = ctx();
    c2.set_pitch(120);
    let slow = ktts_synth::synthesize(&c2, &text, &targets).expect("slow path");
    let diff = fast.iter().zip(slow.iter()).filter(|(a, b)| a != b).count();
    assert!(
        diff > 100,
        "slow path output is too identical to fast path (diff {diff} samples)"
    );
}

#[test]
fn arbitrary_cvc_no_panic_no_silence() {
    let cases: &[(&str, &str)] = &[
        (
            "\x11\x0e\x17",
            "쾅: ㅋㅘㅇ (ㅇ final consonant — silent in the old implementation)",
        ),
        ("\x0d\x0f\x05", "왠: ㅇㅙㄴ (ㅙ → ㅚ normalization)"),
        ("\x0d\x0f\x01", "왜: ㅇㅙ (ㅙ only)"),
        (
            "\x02\x03\x02",
            "각: ㄱㅏㄱ (ㄱ final consonant fJong=-2 → selection skipped)",
        ),
        ("\x14\x03\x13", "합: ㅎㅏㅂ (ㅂ final consonant fJong=-2)"),
        ("\x0e\x03\x08", "갇: ㄱㅏㄷ (ㄷ final consonant fJong=-2)"),
        (
            "\x01\x01\x05",
            "ㄴ: standalone consonant (jung=0x21 pseudo vowel)",
        ),
        ("\x02\x01\x01", "ㄱ: cho only"),
        ("\x01\x01\x01", "empty syllable (1,1,1)"),
        ("\x0d\x03\x05\x0d\x0b\x17", "안녕 2 syllables"),
        ("\x0b\x1d\x13", "싫: ㅅㅣㅂ"),
        (
            "\x0d\x07\x0d",
            "얽: ㅇㅓㄺ (double final ㄺ → representative ㄱ)",
        ),
        ("\x11\x04\x11", "캠: ㅋㅐㅁ"),
        ("\x14\x1a\x17", "휭: ㅎㅠㅇ"),
    ];
    let c = ctx();
    for (cvc, label) in cases {
        let n = cvc.len() / 3 + usize::from(cvc.len() % 3 != 0);
        let segs: Vec<String> = (0..n)
            .map(|i| {
                if cvc.len() == 3 || cvc.len() == 1 {
                    cvc.to_string()
                } else {
                    cvc[i * 3..i * 3 + 3].to_string()
                }
            })
            .collect();
        let text = PronText {
            syllables: segs
                .iter()
                .enumerate()
                .map(|(i, seg)| syl(seg, 0, i == 0))
                .collect(),
            phoneme_codes: cvc.bytes().collect(),
            word_sen: vec![],
        };
        let targets: Vec<SyllableTarget> = segs
            .iter()
            .enumerate()
            .map(|(i, seg)| tgt_for(seg, 130.0, 190.0 + i as f32 * 5.0))
            .collect();
        let pcm = ktts_synth::synthesize(&c, &text, &targets)
            .unwrap_or_else(|e| panic!("{label}: synthesis error: {e}"));
        let v = voiced_len(&pcm) as f32 / 16.0;
        let peak = pcm.iter().map(|&s| s.abs()).max().unwrap_or(0);
        assert!(v > 20.0, "{label}: silent or too short (voiced {v}ms)");
        assert!(peak > 500, "{label}: peak {peak} too small");
    }
}

#[test]
fn wae_normalizes_to_oe_unit() {
    let c = ctx();
    let text = PronText {
        syllables: vec![syl("\x0d\x0f\x01", 0, true)],
        phoneme_codes: vec![0x0d, 0x0f, 0x01],
        word_sen: vec![],
    };
    let targets = vec![tgt_for("\x0d\x0f\x01", 130.0, 190.0)];
    let pcm = ktts_synth::synthesize(&c, &text, &targets).expect("왜 synthesis");
    let v = voiced_len(&pcm) as f32 / 16.0;
    assert!(v > 20.0, "왜 is silent (voiced {v}ms)");
    let phrase = context::build_phrase(&text, &targets).unwrap();
    assert_eq!(
        phrase.letters[0].cvc,
        [0, 0x32, 0],
        "ㅙ must be normalized to ㅚ"
    );
}
