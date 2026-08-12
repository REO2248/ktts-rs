#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PronSyllable {
    pub cvc: [u8; 3],
    pub word_idx: usize,
    pub is_word_start: bool,
    pub pos: u8,
    pub morph_idx: u8,
    pub morph_pos: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PronText {
    pub syllables: Vec<PronSyllable>,
    pub phoneme_codes: Vec<u8>,
    pub word_morphs: Vec<WordMorphs>,
    pub word_sen: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WordMorphs {
    pub pos: Vec<u8>,
    pub first_chars: Vec<u16>,
    pub surfaces: Vec<Vec<u16>>,
    pub source: Vec<u16>,
}

impl PronText {
    #[must_use]
    pub fn word_count(&self) -> usize {
        self.syllables
            .iter()
            .map(|s| s.word_idx)
            .max()
            .map_or(0, |m| m + 1)
    }
}
