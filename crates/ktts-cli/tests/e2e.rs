#![expect(
    unused_crate_dependencies,
    reason = "deps used by other targets of this package"
)]
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "test fixtures: oracle values converted with intentional casts"
)]
use std::io::Write as _;
use std::path::{Path, PathBuf};

use ktts_cli::pipeline::run_pipeline;
use ktts_cli::types::VoiceParams;
use ktts_cli::wav::{SAMPLE_RATE, build_wav, parse_wav_header, rms};

use ktts_pron::kma_code::conv_uni_wan_to_cvc;
use ktts_pron::kma_types::{Morph, WordAnal};

fn fake_word_anal(text: &str) -> WordAnal {
    let mut cvc = Vec::new();
    for ch in text.chars() {
        let w = ch as u16;
        if (0xAC00..=0xD7A3).contains(&w) {
            cvc.extend_from_slice(&conv_uni_wan_to_cvc(w));
        } else {
            cvc.push(w as u8);
        }
    }
    WordAnal {
        morphs: vec![Morph {
            cvc: cvc.clone(),
            pos: [b'n', 0],
            prob: 0.0,
            surface_len: 1,
        }],
        w_byte_num: text.chars().count() * 2,
        word_cvc: cvc,
        source: text.chars().map(|c| c as u16).collect(),
        b_word_sen: false,
    }
}

fn data_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("KTTSDB_DIR").expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
    )
}

fn synthesize(text: &str, out: &Path) -> Vec<i16> {
    synthesize_with(text, &VoiceParams::default(), out)
}

fn synthesize_with(text: &str, params: &VoiceParams, out: &Path) -> Vec<i16> {
    let samples = run_pipeline(text, &data_dir(), params)
        .unwrap_or_else(|e| panic!("pipeline failed ({text}): {e}"));
    assert!(!samples.is_empty(), "synthesis result is empty: {text}");
    let wav = build_wav(&samples);
    std::fs::write(out, &wav).expect("WAV write");
    samples
}

fn check_wav(samples: &[i16], text: &str, min_dur_s: f64, max_dur_s: f64) {
    let wav = build_wav(samples);
    let info = parse_wav_header(&wav).expect("invalid WAV header");
    assert_eq!(info.sample_rate, 16000, "{text}: sample rate");
    assert_eq!(info.channels, 1, "{text}: channel count");
    assert_eq!(info.bits_per_sample, 16, "{text}: bit depth");
    assert_eq!(
        info.data_len as usize,
        samples.len() * 2,
        "{text}: data length"
    );

    let rms_v = rms(samples);
    assert!(rms_v > 0.0, "{text}: expected non-silence (RMS={rms_v})");

    let dur = samples.len() as f64 / f64::from(SAMPLE_RATE);
    assert!(
        dur >= min_dur_s && dur <= max_dur_s,
        "{text}: length out of range ({dur:.2}s, expected {min_dur_s:.2}-{max_dur_s:.2}s)"
    );
}

#[test]
#[ignore = "requires real data + engine crates. KTTSDB_DIR=... cargo test -p ktts-cli --test e2e -- --ignored"]
fn e2e_synthesize_hangul_sentences() {
    let dir = data_dir();
    assert!(
        dir.join("KSpeechDic").is_dir(),
        "invalid data directory: {}",
        dir.display()
    );
    eprintln!("E2E data directory: {}", dir.display());

    let out_dir = std::env::temp_dir().join("ktts_e2e");
    std::fs::create_dir_all(&out_dir).unwrap();

    let cases: &[(&str, f64, f64)] = &[
        ("안녕하세요", 0.2, 3.0),
        ("반갑습니다", 0.2, 3.0),
        ("오늘은 날씨가 좋습니다", 0.5, 8.0),
        (
            "조선어음성합성프로그람은 본문을 음성으로 읽어주는 프로그람입니다.",
            1.0,
            30.0,
        ),
        ("프로그람입니다.", 0.3, 6.0),
    ];

    for (i, (text, min_dur, max_dur)) in cases.iter().enumerate() {
        let out = out_dir.join(format!("case{}_{}.wav", i, text.chars().count()));
        let samples = synthesize(text, &out);
        eprintln!(
            "case{}: '{}' -> {} ({} samples, {:.2}s, RMS={:.1})",
            i,
            text,
            out.display(),
            samples.len(),
            samples.len() as f64 / f64::from(SAMPLE_RATE),
            rms(&samples)
        );
        check_wav(&samples, text, *min_dur, *max_dur);
    }
}

#[test]
#[ignore = "requires real data + engine crates. KTTSDB_DIR=... cargo test -p ktts-cli --test e2e -- --ignored"]
fn e2e_chain_pron_prosody_synth_without_kma() {
    let dir = data_dir();
    let klang = dir.join("KLangDic");
    let speech = dir.join("KSpeechDic").join("woman");

    let pron_ctx = ktts_pron::load_pron_dicts(&klang).expect("pron dictionary load");
    let prosody_ctx = ktts_prosody::load_prosody_dicts(&speech).expect("prosody dictionary load");
    let synth_ctx =
        ktts_synth::load_synth_db(&dir.join("KSpeechDic"), "woman").expect("synth DB load");

    let text = "안녕하세요";
    let words = vec![fake_word_anal(text)];
    let pron = ktts_pron::pronounce(&pron_ctx, &words).expect("pronounce");
    assert_eq!(
        pron.syllables.len(),
        5,
        "expected 5 syllables: {:?}",
        pron.syllables
    );

    let pp = ktts_prosody::PronText {
        syllables: pron
            .syllables
            .iter()
            .map(|s| {
                let mut cvc = [1u8; 3];
                let n = s.cvc.len().min(3);
                cvc[..n].copy_from_slice(&s.cvc[..n]);
                ktts_prosody::PronSyllable {
                    cvc,
                    word_idx: s.word_idx,
                    is_word_start: s.is_word_start,
                    pos: s.pos[0],
                    morph_idx: s.morph_idx,
                    morph_pos: s.morph_pos,
                }
            })
            .collect(),
        phoneme_codes: pron.phoneme_codes.clone(),
        word_morphs: pron
            .word_morphs
            .iter()
            .map(|w| ktts_prosody::WordMorphs {
                pos: w.pos.clone(),
                first_chars: w.first_chars.clone(),
                surfaces: w.surfaces.clone(),
                source: w.source.clone(),
            })
            .collect(),
        word_sen: vec![],
    };
    let targets = ktts_prosody::prosody(&prosody_ctx, &pp).expect("prosody");
    assert_eq!(targets.len(), 5);

    let sp = ktts_synth::PronText {
        syllables: pron
            .syllables
            .iter()
            .map(|s| ktts_synth::PronSyllable {
                cvc: String::from_utf8_lossy(&s.cvc).into_owned(),
                word_idx: s.word_idx,
                is_word_start: s.is_word_start,
                pos: s.pos[0],
            })
            .collect(),
        phoneme_codes: pron.phoneme_codes.clone(),
        word_sen: vec![],
    };
    let st: Vec<ktts_synth::SyllableTarget> = targets
        .iter()
        .map(|t| ktts_synth::SyllableTarget {
            dur: t.dur,
            ave_length: t.ave_length,
            f0: t.f0,
            tobi: t.tobi,
            boundary: t.boundary,
        })
        .collect();
    let samples = ktts_synth::synthesize(&synth_ctx, &sp, &st).expect("synthesize");
    assert!(!samples.is_empty());

    let out = std::env::temp_dir().join("ktts_e2e_chain_no_kma.wav");
    std::fs::write(&out, build_wav(&samples)).unwrap();
    check_wav(&samples, text, 0.2, 4.0);
    eprintln!(
        "chain-no-kma OK: {} samples ({:.2}s, RMS={:.1}) -> {}",
        samples.len(),
        samples.len() as f64 / f64::from(SAMPLE_RATE),
        rms(&samples),
        out.display()
    );
}

#[test]
#[ignore = "requires real data. KTTSDB_DIR=... cargo test -p ktts-cli --test e2e -- --ignored"]
fn e2e_stdin_input_matches_arg_input() {
    let dir = data_dir();
    let text = "오늘은 날씨가 좋습니다";

    let arg_out = std::process::Command::new(env!("CARGO_BIN_EXE_ktts"))
        .args(["-d", dir.to_str().unwrap(), text])
        .output()
        .expect("run ktts with TEXT args");
    assert!(
        arg_out.status.success(),
        "args path failed: {}",
        String::from_utf8_lossy(&arg_out.stderr)
    );

    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_ktts"))
        .args(["-d", dir.to_str().unwrap()])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn ktts with piped stdin");
    child
        .stdin
        .take()
        .expect("stdin handle")
        .write_all(text.as_bytes())
        .expect("write text to stdin");
    let stdin_out = child.wait_with_output().expect("wait for ktts");
    assert!(
        stdin_out.status.success(),
        "stdin path failed: {}",
        String::from_utf8_lossy(&stdin_out.stderr)
    );
    assert_eq!(
        stdin_out.stdout, arg_out.stdout,
        "stdin input must produce byte-identical WAV to TEXT args"
    );
}

#[test]
#[ignore = "requires real data. KTTSDB_DIR=... cargo test -p ktts-cli --test e2e -- --ignored"]
fn e2e_no_input_is_usage_error() {
    let dir = data_dir();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_ktts"))
        .args(["-d", dir.to_str().unwrap()])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("run ktts without input");
    assert_eq!(
        out.status.code(),
        Some(2),
        "expected usage-error exit code 2, got {:?}",
        out.status.code()
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("no input text"), "stderr: {stderr}");
}

#[cfg(feature = "embed")]
#[test]
#[ignore = "requires real data + embed build. KTTSDB_DIR=... cargo test -p ktts-cli --test e2e --features embed -- --ignored"]
fn e2e_embedded_default_matches_explicit_data_dir() {
    let dir = data_dir();
    let text = "오늘은 날씨가 좋습니다";

    let embedded = std::process::Command::new(env!("CARGO_BIN_EXE_ktts"))
        .args([text])
        .output()
        .expect("run embedded ktts without -d");
    assert!(
        embedded.status.success(),
        "embedded path failed: {}",
        String::from_utf8_lossy(&embedded.stderr)
    );

    let with_dir = std::process::Command::new(env!("CARGO_BIN_EXE_ktts"))
        .args(["-d", dir.to_str().unwrap(), text])
        .output()
        .expect("run ktts with -d");
    assert!(
        with_dir.status.success(),
        "dir path failed: {}",
        String::from_utf8_lossy(&with_dir.stderr)
    );

    assert_eq!(
        embedded.stdout, with_dir.stdout,
        "embedded dictionaries must produce byte-identical WAV to -d kttsdb"
    );
}

#[test]
#[ignore = "requires real data. KTTSDB_DIR=... cargo test -p ktts-cli --test e2e -- --ignored"]
fn e2e_default_params_byte_identical_to_explicit_defaults() {
    let text = "안녕하세요";
    let out = std::env::temp_dir().join("ktts_e2e_params.wav");
    let base = synthesize(text, &out);
    let explicit = synthesize_with(
        text,
        &VoiceParams {
            speed: 1.0,
            pitch: 0.0,
            volume: 1.0,
        },
        &out,
    );
    assert_eq!(base, explicit, "explicit defaults must not change output");
}

#[test]
#[ignore = "requires real data. KTTSDB_DIR=... cargo test -p ktts-cli --test e2e -- --ignored"]
fn e2e_voice_params_change_output() {
    let text = "오늘은 날씨가 좋습니다";
    let out = std::env::temp_dir().join("ktts_e2e_params.wav");
    let base = synthesize(text, &out);

    let fast = synthesize_with(
        text,
        &VoiceParams {
            speed: 2.0,
            ..VoiceParams::default()
        },
        &out,
    );
    assert_ne!(base, fast, "speed 2.0 must change the output");

    let pitched = synthesize_with(
        text,
        &VoiceParams {
            pitch: 1.0,
            ..VoiceParams::default()
        },
        &out,
    );
    assert_ne!(base, pitched, "pitch 1.0 must change the output");

    let quiet = synthesize_with(
        text,
        &VoiceParams {
            volume: 0.5,
            ..VoiceParams::default()
        },
        &out,
    );
    assert_ne!(base, quiet, "volume 0.5 must change the output");
    assert!(
        rms(&quiet) < rms(&base),
        "volume 0.5 must lower RMS (base={:.1}, quiet={:.1})",
        rms(&base),
        rms(&quiet)
    );

    eprintln!(
        "params OK: base {}s (RMS={:.1}), speed2 {}s, pitch1 RMS={:.1}, vol0.5 RMS={:.1}",
        base.len() as f64 / f64::from(SAMPLE_RATE),
        rms(&base),
        fast.len() as f64 / f64::from(SAMPLE_RATE),
        rms(&pitched),
        rms(&quiet)
    );
}
