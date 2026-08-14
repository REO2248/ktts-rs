use std::collections::HashMap;

use wasm_bindgen::prelude::*;

#[cfg(test)]
use sha2 as _;

pub type DataMap = ktts_dict::common::DataMap;

#[cfg(feature = "embed")]
pub mod embedded;

pub mod wav {
    pub const SAMPLE_RATE: u32 = 16000;
    pub const CHANNELS: u16 = 1;
    pub const BITS_PER_SAMPLE: u16 = 16;

    #[must_use]
    /// Builds a WAV file from PCM samples.
    ///
    /// # Panics
    ///
    /// Panics if the sample count does not fit in `u32`.
    pub fn build_wav(samples: &[i16]) -> Vec<u8> {
        let data_len = u32::try_from(samples.len() * 2).expect("WAV data length fits u32");
        let mut out = Vec::with_capacity(44 + samples.len() * 2);
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + data_len).to_le_bytes());
        out.extend_from_slice(b"WAVE");
        out.extend_from_slice(b"fmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&CHANNELS.to_le_bytes());
        out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
        out.extend_from_slice(
            &(SAMPLE_RATE * u32::from(CHANNELS) * (u32::from(BITS_PER_SAMPLE) / 8)).to_le_bytes(),
        );
        out.extend_from_slice(&(CHANNELS * BITS_PER_SAMPLE / 8).to_le_bytes());
        out.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_len.to_le_bytes());
        for &s in samples {
            out.extend_from_slice(&s.to_le_bytes());
        }
        out
    }

    #[must_use]
    pub fn rms(samples: &[i16]) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum: f64 = samples.iter().map(|&s| f64::from(s) * f64::from(s)).sum();
        #[expect(
            clippy::cast_precision_loss,
            reason = "sample count to f64 is exact for real audio"
        )]
        (sum / samples.len() as f64).sqrt()
    }

    #[must_use]
    pub fn parse_pcm16(bytes: &[u8]) -> Option<Vec<i16>> {
        if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
            return None;
        }
        let data_len = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]) as usize;
        if bytes.len() < 44 + data_len || !data_len.is_multiple_of(2) {
            return None;
        }
        Some(
            bytes[44..44 + data_len]
                .chunks_exact(2)
                .map(|c| i16::from_le_bytes([c[0], c[1]]))
                .collect(),
        )
    }

    #[cfg(test)]
    mod tests {
        #![expect(
            clippy::float_cmp,
            reason = "oracle assertions use exact float equality"
        )]
        use super::*;

        #[test]
        fn header_layout_is_44_bytes_and_correct() {
            let wav = build_wav(&[]);
            assert_eq!(wav.len(), 44);
            assert_eq!(&wav[0..4], b"RIFF");
            assert_eq!(&wav[8..12], b"WAVE");
            assert_eq!(&wav[36..40], b"data");
            assert_eq!(u32::from_le_bytes([wav[4], wav[5], wav[6], wav[7]]), 36);
            assert_eq!(u16::from_le_bytes([wav[20], wav[21]]), 1);
            assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 1);
            assert_eq!(
                u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]),
                16000
            );
            assert_eq!(
                u32::from_le_bytes([wav[28], wav[29], wav[30], wav[31]]),
                32000
            );
            assert_eq!(u16::from_le_bytes([wav[32], wav[33]]), 2);
            assert_eq!(u16::from_le_bytes([wav[34], wav[35]]), 16);
        }

        #[test]
        fn roundtrip_pcm16() {
            let samples = [1i16, -2, 32767, -32768, 0];
            let wav = build_wav(&samples);
            assert_eq!(parse_pcm16(&wav), Some(samples.to_vec()));
            assert_eq!(parse_pcm16(&wav[..43]), None);
            assert_eq!(parse_pcm16(b"not a wav at all........"), None);
        }

        #[test]
        fn rms_zero_for_silence_positive_for_tone() {
            assert_eq!(rms(&[]), 0.0);
            assert_eq!(rms(&[0, 0, 0]), 0.0);
            assert!(rms(&[1000, -1000, 1000, -1000]) > 0.0);
        }
    }
}

fn conv_kma_word(w: ktts_kma::WordAnal) -> ktts_pron::kma_types::WordAnal {
    ktts_pron::kma_types::WordAnal {
        morphs: w
            .morphs
            .into_iter()
            .map(|m| ktts_pron::kma_types::Morph {
                cvc: m.cvc,
                pos: m.pos,
                prob: m.prob,
                surface_len: m.surface_len,
            })
            .collect(),
        w_byte_num: w.w_byte_num as usize,
        word_cvc: w.word_cvc,
        source: w.source,
        b_word_sen: w.b_word_sen,
    }
}

#[cfg(test)]
fn conv_pron_to_prosody(p: &ktts_pron::PronText) -> ktts_prosody::PronText {
    let syllables = p
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
        .collect();
    ktts_prosody::PronText {
        syllables,
        phoneme_codes: p.phoneme_codes.clone(),
        word_morphs: p
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
    }
}

fn conv_pron_to_synth(p: &ktts_pron::PronText) -> ktts_synth::PronText {
    let syllables: Vec<ktts_synth::PronSyllable> = p
        .syllables
        .iter()
        .map(|s| ktts_synth::PronSyllable {
            cvc: s.cvc.iter().map(|&b| b as char).collect(),
            word_idx: s.word_idx,
            is_word_start: s.is_word_start,
            pos: s.pos[0],
        })
        .collect();
    ktts_synth::PronText {
        syllables,
        phoneme_codes: p.phoneme_codes.clone(),
        word_sen: vec![],
    }
}

fn prosody_sentence(
    prosody: &ktts_prosody::ProsodyContext,
    pron: &ktts_pron::PronText,
    word_sen: &[u8],
    w_start: usize,
    w_end: usize,
) -> Result<Vec<ktts_synth::SyllableTarget>, String> {
    let syllables: Vec<ktts_prosody::PronSyllable> = pron
        .syllables
        .iter()
        .filter(|s| s.word_idx >= w_start && s.word_idx <= w_end)
        .map(|s| {
            let mut cvc = [1u8; 3];
            let n = s.cvc.len().min(3);
            cvc[..n].copy_from_slice(&s.cvc[..n]);
            ktts_prosody::PronSyllable {
                cvc,
                word_idx: s.word_idx - w_start,
                is_word_start: s.is_word_start,
                pos: s.pos[0],
                morph_idx: s.morph_idx,
                morph_pos: s.morph_pos,
            }
        })
        .collect();
    let word_morphs = pron
        .word_morphs
        .get(w_start..=w_end)
        .map(|wm| {
            wm.iter()
                .map(|w| ktts_prosody::WordMorphs {
                    pos: w.pos.clone(),
                    first_chars: w.first_chars.clone(),
                    surfaces: w.surfaces.clone(),
                    source: w.source.clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    let sub = ktts_prosody::PronText {
        syllables,
        phoneme_codes: vec![],
        word_morphs,
        word_sen: word_sen
            .get(w_start..=w_end)
            .map(<[u8]>::to_vec)
            .unwrap_or_default(),
    };
    let raw = ktts_prosody::prosody(prosody, &sub).map_err(|e| format!("prosody: {e}"))?;
    Ok(raw.into_iter().map(conv_target_to_synth).collect())
}

const fn conv_target_to_synth(t: ktts_prosody::SyllableTarget) -> ktts_synth::SyllableTarget {
    ktts_synth::SyllableTarget {
        dur: t.dur,
        ave_length: t.ave_length,
        f0: t.f0,
        tobi: t.tobi,
        boundary: t.boundary,
    }
}

fn js_to_datamap(files: &JsValue) -> Result<DataMap, JsValue> {
    use wasm_bindgen::JsCast;

    let obj = js_sys::Object::try_from(files)
        .ok_or_else(|| JsValue::from_str("files must be an object ({ path: Uint8Array })"))?;
    let keys = js_sys::Object::keys(obj);
    let mut map = HashMap::new();
    for i in 0..keys.length() {
        let key = keys.get(i);
        let key_str = key
            .as_string()
            .ok_or_else(|| JsValue::from_str("data map key is not a string"))?;
        let val = js_sys::Reflect::get(obj, &key)?;
        if val.is_undefined() || val.is_null() {
            return Err(JsValue::from_str(&format!("{key_str}: no value")));
        }
        let arr: js_sys::Uint8Array = val
            .dyn_into()
            .map_err(|_| JsValue::from_str(&format!("{key_str}: not a Uint8Array")))?;
        map.insert(key_str, arr.to_vec());
    }
    if map.is_empty() {
        return Err(JsValue::from_str("data map is empty"));
    }
    Ok(map)
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct KttsEngine {
    kma: Option<ktts_kma::KmaContext>,
    pron: Option<ktts_pron::PronContext>,
    prosody: Option<ktts_prosody::ProsodyContext>,
    synth: Option<ktts_synth::SynthContext>,
    gender: String,
}

impl KttsEngine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            kma: None,
            pron: None,
            prosody: None,
            synth: None,
            gender: "woman".to_string(),
        }
    }

    /// Loads the dictionary data map into the engine.
    ///
    /// The map is consumed: the synthesis DB (the bulk of the data) is moved
    /// out of it instead of being cloned.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is invalid.
    pub fn set_data_impl(&mut self, files: DataMap) -> Result<(), String> {
        let kma = ktts_kma::load_kma_dicts_bytes(&files)
            .map_err(|e| format!("ktts-kma: load_kma_dicts_bytes: {e}"))?;
        let pron = ktts_pron::load_pron_dicts_bytes(&files)
            .map_err(|e| format!("ktts-pron: load_pron_dicts_bytes: {e}"))?;
        let prosody = ktts_prosody::load_prosody_dicts_bytes(&files, &self.gender)
            .map_err(|e| format!("ktts-prosody: load_prosody_dicts_bytes: {e}"))?;
        let synth = ktts_synth::load_synth_db_bytes(files, &self.gender)
            .map_err(|e| format!("ktts-synth: load_synth_db_bytes: {e}"))?;
        self.kma = Some(kma);
        self.pron = Some(pron);
        self.prosody = Some(prosody);
        self.synth = Some(synth);
        Ok(())
    }

    /// Loads the dictionaries embedded into this binary (embed feature).
    ///
    /// # Errors
    ///
    /// Returns an error if the embedded data is malformed.
    #[cfg(feature = "embed")]
    pub fn set_embedded_impl(&mut self) -> Result<(), String> {
        let files = embedded::datamap()?;
        self.set_data_impl(files)
    }

    /// Synthesizes a text into WAV bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the engine is not loaded or synthesis fails.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "C port: sentence marker byte truncation; param scaling to engine units"
    )]
    pub fn synthesize_impl(
        &mut self,
        text: &str,
        speed: f32,
        pitch: f32,
        volume: f32,
    ) -> Result<Vec<u8>, String> {
        let kma = self
            .kma
            .as_ref()
            .ok_or_else(|| "set_data not called (kma)".to_string())?;
        let pron = self
            .pron
            .as_ref()
            .ok_or_else(|| "set_data not called (pron)".to_string())?;
        let prosody = self
            .prosody
            .as_ref()
            .ok_or_else(|| "set_data not called (prosody)".to_string())?;
        let synth = self
            .synth
            .as_mut()
            .ok_or_else(|| "set_data not called (synth)".to_string())?;

        let raw_words = ktts_kma::analyze(kma, text).map_err(|e| format!("kma analyze: {e}"))?;
        let word_sen: Vec<u8> = raw_words
            .iter()
            .map(|w| {
                if w.b_word_sen {
                    w.source.last().copied().unwrap_or(0) as u8
                } else {
                    0
                }
            })
            .collect();
        let words: Vec<ktts_pron::kma_types::WordAnal> =
            raw_words.iter().cloned().map(conv_kma_word).collect();

        let raw_pron =
            ktts_pron::pronounce(pron, &words).map_err(|e| format!("pron pronounce: {e}"))?;

        let mut targets: Vec<ktts_synth::SyllableTarget> = Vec::new();
        let mut w_start = 0usize;
        let word_num = raw_words.len();
        for (wi, w) in raw_words.iter().enumerate() {
            if w.b_sentence_end {
                targets.extend(prosody_sentence(
                    prosody, &raw_pron, &word_sen, w_start, wi,
                )?);
                w_start = wi + 1;
            }
        }
        if w_start < word_num {
            targets.extend(prosody_sentence(
                prosody,
                &raw_pron,
                &word_sen,
                w_start,
                word_num - 1,
            )?);
        }

        // One shot: setting the params individually would reset the others.
        // Params left at their API default keep the InfoDic.wdic values, so
        // each call is independent of the previous one.
        synth.set_params(ktts_synth::setting::IniParams::from_api(
            synth.base_ini_params(),
            speed,
            pitch,
            volume,
        ));

        let synth_input = conv_pron_to_synth(&raw_pron);
        if targets.is_empty() {
            return Ok(wav::build_wav(&[]));
        }
        let samples = ktts_synth::synthesize(synth, &synth_input, &targets)
            .map_err(|e| format!("synth synthesize: {e}"))?;
        Ok(wav::build_wav(&samples))
    }
}

impl Default for KttsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl KttsEngine {
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn wasm_new() -> Self {
        Self::new()
    }

    /// Creates an engine with the dictionaries embedded into this binary
    /// (embed feature build).
    ///
    /// # Errors
    ///
    /// Rejects if the embedded data is malformed.
    #[cfg(feature = "embed")]
    pub fn embedded() -> Result<Self, JsValue> {
        let mut engine = Self::new();
        engine
            .set_embedded_impl()
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(engine)
    }

    pub fn set_gender(&mut self, gender: &str) {
        self.gender = gender.to_string();
    }

    #[must_use]
    pub fn gender(&self) -> String {
        self.gender.clone()
    }

    /// Loads the dictionary data map (JS object of `Uint8Array` files) into the engine.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is invalid.
    pub fn set_data(&mut self, files: &JsValue) -> Result<(), JsValue> {
        let map = js_to_datamap(files)?;
        self.set_data_impl(map).map_err(|e| JsValue::from_str(&e))
    }

    #[must_use]
    #[expect(
        clippy::missing_const_for_fn,
        reason = "wasm_bindgen requires non-const exported methods"
    )]
    pub fn is_ready(&self) -> bool {
        self.kma.is_some() && self.pron.is_some() && self.prosody.is_some() && self.synth.is_some()
    }

    /// Synthesizes a text into a WAV file (JS entry point).
    ///
    /// # Errors
    ///
    /// Returns an error if the data is invalid.
    pub fn synthesize(
        &mut self,
        text: &str,
        speed: f32,
        pitch: f32,
        volume: f32,
    ) -> Result<Vec<u8>, JsValue> {
        self.synthesize_impl(text, speed, pitch, volume)
            .map_err(|e| JsValue::from_str(&e))
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::float_cmp,
        reason = "oracle assertions use exact float equality"
    )]
    use super::*;

    #[test]
    fn kma_word_conversion_carries_b_word_sen() {
        let w = ktts_kma::WordAnal {
            morphs: vec![ktts_kma::Morph {
                cvc: vec![13, 3, 2],
                pos: [b'n', 0],
                prob: -1.5,
                surface_len: 1,
            }],
            w_byte_num: 2,
            word_cvc: vec![13, 3, 2],
            b_word_sen: true,
            b_sentence_end: false,
            source: vec![0xC548],
        };
        let pw = conv_kma_word(w);
        assert_eq!(pw.morphs.len(), 1);
        assert_eq!(pw.morphs[0].cvc, vec![13, 3, 2]);
        assert_eq!(pw.morphs[0].pos, [b'n', 0]);
        assert_eq!(pw.w_byte_num, 2usize);
        assert_eq!(pw.word_cvc, vec![13, 3, 2]);
        assert!(pw.b_word_sen);
    }

    #[test]
    fn pron_to_prosody_syllable_conversion() {
        let p = ktts_pron::PronText {
            syllables: vec![
                ktts_pron::PronSyllable {
                    cvc: vec![13, 3, 2],
                    word_idx: 0,
                    is_word_start: true,
                    pos: [b'n', 0],
                    morph_idx: 0,
                    morph_pos: 0,
                },
                ktts_pron::PronSyllable {
                    cvc: vec![b'.'],
                    word_idx: 1,
                    is_word_start: true,
                    pos: [b'L', 0],
                    morph_idx: 0,
                    morph_pos: 0,
                },
            ],
            phoneme_codes: vec![13, 3, 2, b'.'],
            word_morphs: vec![],
        };
        let pp = conv_pron_to_prosody(&p);
        assert_eq!(pp.syllables.len(), 2);
        assert_eq!(pp.syllables[0].cvc, [13, 3, 2]);
        assert_eq!(pp.syllables[0].pos, b'n');
        assert_eq!(pp.syllables[1].cvc, [b'.', 1, 1]);
        assert_eq!(pp.syllables[1].pos, b'L');
        assert_eq!(pp.phoneme_codes, vec![13, 3, 2, b'.']);
    }

    #[test]
    fn pron_to_synth_passes_raw_cvc_unchanged() {
        let p = ktts_pron::PronText {
            syllables: vec![ktts_pron::PronSyllable {
                cvc: vec![13, 3, 2],
                word_idx: 0,
                is_word_start: true,
                pos: [b'n', 0],
                morph_idx: 0,
                morph_pos: 0,
            }],
            phoneme_codes: vec![13, 3, 2],
            word_morphs: vec![],
        };
        let sp = conv_pron_to_synth(&p);
        assert_eq!(sp.syllables.len(), 1);
        assert_eq!(sp.syllables[0].cvc, "\r\x03\x02");
        assert_eq!(sp.syllables[0].pos, b'n');
        assert_eq!(sp.phoneme_codes, vec![13, 3, 2]);
    }

    #[test]
    fn target_conversion_copies_fields() {
        let t = ktts_prosody::SyllableTarget {
            dur: 123.5,
            ave_length: [1, 2, 3],
            f0: [100.0; 12],
            tobi: 0.7,
            boundary: 0x15,
        };
        let ts = conv_target_to_synth(t);
        assert_eq!(ts.dur, 123.5);
        assert_eq!(ts.ave_length, [1, 2, 3]);
        assert_eq!(ts.f0, [100.0; 12]);
        assert_eq!(ts.tobi, 0.7);
        assert_eq!(ts.boundary, 0x15);
    }

    #[test]
    fn engine_requires_set_data() {
        let mut engine = KttsEngine::new();
        assert!(!engine.is_ready());
        let err = engine
            .synthesize_impl("안녕하세요", 1.0, 0.0, 1.0)
            .unwrap_err();
        assert!(err.contains("set_data"), "got: {err}");
        let err2 = engine.synthesize_impl("   ", 1.0, 0.0, 1.0).unwrap_err();
        assert!(err2.contains("set_data"), "got: {err2}");
    }

    #[test]
    fn gender_default_woman() {
        let mut engine = KttsEngine::new();
        assert_eq!(engine.gender(), "woman");
        engine.set_gender("man");
        assert_eq!(engine.gender(), "man");
    }
}
