#[derive(Debug, Clone, PartialEq)]
pub struct Morph {
    pub cvc: Vec<u8>,
    pub pos: [u8; 2],
    pub prob: f64,
    pub surface_len: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordAnal {
    pub morphs: Vec<Morph>,
    pub w_byte_num: u16,
    pub word_cvc: Vec<u8>,
    pub b_word_sen: bool,
    pub b_word_sen_char: u8,
    pub b_sentence_end: bool,
    pub source: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronSyllable {
    pub cvc: Vec<u8>,
    pub word_idx: usize,
    pub is_word_start: bool,
    pub pos: [u8; 2],
    pub morph_idx: u8,
    pub morph_pos: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PronText {
    pub syllables: Vec<PronSyllable>,
    pub phoneme_codes: Vec<u8>,
    pub word_morphs: Vec<WordMorphInfo>,
    pub word_sen: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WordMorphInfo {
    pub pos: Vec<u8>,
    pub first_chars: Vec<u16>,
    pub surfaces: Vec<Vec<u16>>,
    pub source: Vec<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyllableTarget {
    pub dur: f32,
    pub ave_length: [u16; 3],
    pub f0: [f32; 12],
    pub tobi: f32,
    pub boundary: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoiceParams {
    pub speed: f32,
    pub pitch: f32,
    pub volume: f32,
}

impl Default for VoiceParams {
    fn default() -> Self {
        Self {
            speed: 1.0,
            pitch: 0.0,
            volume: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineError {
    Engine(&'static str, String),
    EmptyInput,
    EmptyOutput,
    BadParam(String),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Engine(name, msg) => write!(f, "engine '{name}': {msg}"),
            Self::EmptyInput => write!(f, "input text is empty"),
            Self::EmptyOutput => write!(f, "synthesis result is empty"),
            Self::BadParam(msg) => write!(f, "invalid parameter: {msg}"),
        }
    }
}

impl std::error::Error for PipelineError {}
