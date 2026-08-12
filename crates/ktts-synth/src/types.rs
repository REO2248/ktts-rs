#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronSyllable {
    pub cvc: String,
    pub word_idx: usize,
    pub is_word_start: bool,
    pub pos: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronText {
    pub syllables: Vec<PronSyllable>,
    pub phoneme_codes: Vec<u8>,
    pub word_sen: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyllableTarget {
    pub dur: f32,
    pub ave_length: [u16; 3],
    pub f0: [f32; 12],
    pub tobi: f32,
    pub boundary: u8,
}
