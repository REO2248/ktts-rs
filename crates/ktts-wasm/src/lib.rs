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
    engine: Option<ktts_engine::Engine>,
    gender: String,
}

impl KttsEngine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            engine: None,
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
        self.engine =
            Some(ktts_engine::Engine::load(files, &self.gender).map_err(|e| e.to_string())?);
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
    pub fn synthesize_impl(
        &mut self,
        text: &str,
        speed: f32,
        pitch: f32,
        volume: f32,
    ) -> Result<Vec<u8>, String> {
        let engine = self
            .engine
            .as_ref()
            .ok_or_else(|| "set_data not called".to_string())?;
        let samples = engine
            .synthesize(
                text,
                &ktts_engine::VoiceParams {
                    speed,
                    pitch,
                    volume,
                },
            )
            .map_err(|e| e.to_string())?;
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
        self.engine.is_some()
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
    use super::*;

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
