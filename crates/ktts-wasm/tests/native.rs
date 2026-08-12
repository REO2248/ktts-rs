#![expect(
    unused_crate_dependencies,
    reason = "deps used by other targets of this package"
)]
#![expect(
    clippy::cast_precision_loss,
    reason = "test fixtures: sample counts to f64"
)]
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ktts_wasm::KttsEngine;
use ktts_wasm::wav::{parse_pcm16, rms};
use sha2::Digest;

fn build_datamap(dir: &Path) -> HashMap<String, Vec<u8>> {
    let mut map = HashMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let mut entries: Vec<_> = std::fs::read_dir(&d)
            .unwrap_or_else(|e| panic!("read_dir {}: {e}", d.display()))
            .collect::<Result<_, _>>()
            .unwrap();
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p
                    .strip_prefix(dir)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                let data =
                    std::fs::read(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()));
                map.insert(rel, data);
            }
        }
    }
    map
}

fn data_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("KTTSDB_DIR").expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut h = sha2::Sha256::new();
    h.update(bytes);
    let mut out = String::with_capacity(64);
    for b in h.finalize() {
        let _ = write!(out, "{b:02x}");
    }
    out
}

fn new_loaded_engine() -> KttsEngine {
    let dir = data_dir();
    assert!(
        dir.join("InfoDic.wdic").exists(),
        "data not found: {}",
        dir.display()
    );
    let map = build_datamap(&dir);
    let mut engine = KttsEngine::new();
    engine
        .set_data_impl(map)
        .unwrap_or_else(|e| panic!("set_data_impl: {e}"));
    assert!(engine.is_ready());
    engine
}

fn assert_valid_wav(wav: &[u8], label: &str) {
    assert!(
        wav.len() >= 44,
        "{label}: less than 44B (len={})",
        wav.len()
    );
    assert_eq!(&wav[0..4], b"RIFF", "{label}: RIFF magic mismatch");
    assert_eq!(&wav[8..12], b"WAVE", "{label}: WAVE magic mismatch");
    assert_eq!(&wav[12..16], b"fmt ", "{label}: fmt mismatch");
    assert_eq!(&wav[36..40], b"data", "{label}: data mismatch");
    assert_eq!(
        u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]),
        16000
    );
    assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 1);
    assert_eq!(u16::from_le_bytes([wav[34], wav[35]]), 16);

    let samples = parse_pcm16(wav).unwrap_or_else(|| panic!("{label}: PCM16 parse failed"));
    assert!(
        samples.len() > 1600,
        "{label}: too few samples ({})",
        samples.len()
    );
    let r = rms(&samples);
    assert!(r > 0.0, "{label}: silence (RMS=0)");
    assert!(r < 20000.0, "{label}: RMS abnormally large ({r})");
    println!(
        "{label}: {} samples ({:.2}s), RMS={r:.1}",
        samples.len(),
        samples.len() as f64 / 16000.0
    );
}

#[test]
fn synthesize_annyonghaseyo_returns_non_silent_wav() {
    let mut engine = new_loaded_engine();
    let wav = engine
        .synthesize_impl("안녕하세요", 1.0, 0.0, 1.0)
        .unwrap_or_else(|e| panic!("synthesize_impl: {e}"));
    assert_valid_wav(&wav, "안녕하세요 (default)");
}

#[test]
fn synthesize_with_params_variations() {
    let mut engine = new_loaded_engine();
    let wav1 = engine
        .synthesize_impl("안녕하세요", 1.5, 0.2, 1.5)
        .unwrap_or_else(|e| panic!("synthesize_impl (fast): {e}"));
    assert_valid_wav(&wav1, "안녕하세요 (speed=1.5)");
    let wav2 = engine
        .synthesize_impl("안녕하세요", 0.8, -0.1, 0.5)
        .unwrap_or_else(|e| panic!("synthesize_impl (slow): {e}"));
    assert_valid_wav(&wav2, "안녕하세요 (speed=0.8)");
}

#[test]
fn synthesize_longer_sentence() {
    let mut engine = new_loaded_engine();
    let wav = engine
        .synthesize_impl("안녕하세요. 반갑습니다.", 1.0, 0.0, 1.0)
        .unwrap_or_else(|e| panic!("synthesize_impl: {e}"));
    assert_valid_wav(&wav, "2 sentences");
}

#[cfg(feature = "embed")]
#[test]
fn embedded_engine_is_ready_and_synthesizes() {
    // The dictionary data is baked into the binary: no KTTSDB_DIR needed.
    let mut engine = KttsEngine::embedded().unwrap_or_else(|e| panic!("embedded engine: {e:?}"));
    assert!(engine.is_ready());
    let wav = engine
        .synthesize_impl("안녕하세요", 1.0, 0.0, 1.0)
        .unwrap_or_else(|e| panic!("synthesize_impl: {e}"));
    assert_valid_wav(&wav, "embedded engine");
}

#[test]
fn set_data_missing_dicts_errors() {
    let mut map = HashMap::new();
    map.insert("InfoDic.wdic".to_string(), vec![0u8; 16]);
    let mut engine = KttsEngine::new();
    let err = engine.set_data_impl(map).unwrap_err();
    assert!(!err.is_empty());
    assert!(!engine.is_ready());
    println!("error as expected: {err}");
}

#[test]
fn empty_input_yields_empty_wav() {
    let mut engine = new_loaded_engine();
    let wav = engine
        .synthesize_impl("   ", 1.0, 0.0, 1.0)
        .unwrap_or_else(|e| panic!("synthesize_impl: {e}"));
    assert_eq!(wav.len(), 44);
    assert_eq!(&wav[0..4], b"RIFF");
}

#[test]
fn anthem_4line_bit_identical_to_oracle() {
    use ktts_wasm::wav::parse_pcm16;
    let mut engine = new_loaded_engine();
    let wav = engine
        .synthesize_impl(
            "아침은 빛나라 이 강산\n은금에 자원도 가득한\n이 세상 아름다운 내 조국\n반만년 오랜 력사에",
            1.0,
            0.0,
            1.0,
        )
        .expect("synthesize_impl");
    let samples = parse_pcm16(&wav).expect("pcm16");
    assert_eq!(samples.len(), 165_670);
    let bytes: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
    let digest = sha256_hex(&bytes);
    assert_eq!(
        digest,
        "a6917c5648e8d6c5164745d32161964bb598c059e66521b9115b1c52a9eff237",
    );
}

#[test]
fn anthem_1line_bit_identical_to_oracle() {
    use ktts_wasm::wav::parse_pcm16;
    let mut engine = new_loaded_engine();
    let wav = engine
        .synthesize_impl(
            "아침은 빛나라 이 강산 은금에 자원도 가득한 이 세상 아름다운 내 조국 반만년 오랜 력사에",
            1.0,
            0.0,
            1.0,
        )
        .expect("synthesize_impl");
    let samples = parse_pcm16(&wav).expect("pcm16");
    assert_eq!(samples.len(), 133_855);
    let bytes: Vec<u8> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
    let digest = sha256_hex(&bytes);
    assert_eq!(
        digest,
        "8ceb46f84bf09c4fe9c3e20890d1a1268d91ab0ba09c6836f9ea2f3637d99654",
    );
}
