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
    pub w_byte_num: usize,
    pub word_cvc: Vec<u8>,
    pub source: Vec<u16>,
    pub b_word_sen: bool,
}

impl WordAnal {
    #[must_use]
    pub fn cvc(&self) -> Vec<u8> {
        if self.word_cvc.is_empty() {
            let mut v = Vec::new();
            for m in &self.morphs {
                v.extend_from_slice(&m.cvc);
            }
            v
        } else {
            self.word_cvc.clone()
        }
    }

    #[must_use]
    pub fn tags(&self) -> Vec<u8> {
        self.morphs.iter().map(|m| m.pos[0]).collect()
    }

    #[must_use]
    pub const fn is_symbol_morph(m: &Morph) -> bool {
        matches!(
            m.pos[0],
            b'L' | b'M' | b'N' | b'O' | b'P' | b'Q' | b'R' | b'S' | b'I' | b'J' | b'K'
        )
    }
}
