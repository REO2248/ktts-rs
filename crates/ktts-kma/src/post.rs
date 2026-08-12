use crate::code;
use crate::ma::MaWord;
use crate::tables;
use crate::{Morph, WordAnal};

#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
pub fn get_word_info(words: &[MaWord]) -> Vec<WordAnal> {
    let mut out = Vec::with_capacity(words.len());
    for w in words {
        let b_word_sen = w.source.last().is_some_and(|&c| tables::is_sen_symbol(c));
        let mut w_byte_num = w.source.len();
        let mut morphs: Vec<Morph> = w
            .morphs
            .clone()
            .into_iter()
            .map(|m| Morph {
                cvc: m.cvc,
                pos: [m.ch_tag, 0],
                prob: m.prob,
                surface_len: code::get_kchar_count(&m.pyogi) as u8,
            })
            .collect();
        if b_word_sen && w.source.last() != Some(&0x2c) {
            w_byte_num = w_byte_num.saturating_sub(1);
            if morphs.len() > 1 {
                while morphs.last().is_some_and(|m| m.cvc.is_empty()) {
                    morphs.pop();
                }
                let is_sym = morphs.last().is_some_and(|m| {
                    m.cvc.len() == 1 && tables::is_sen_symbol(u16::from(m.cvc[0]))
                });
                if is_sym {
                    morphs.pop();
                }
            }
        }
        let mut word_cvc: Vec<u8> = Vec::with_capacity(w_byte_num * 3);
        for &c in w.source.iter().take(w_byte_num) {
            if tables::is_uni_korean_code(c) {
                word_cvc.extend_from_slice(&code::conv_uni_code_to_cvc(c));
            } else {
                word_cvc.extend_from_slice(&[0, 0, 0]);
            }
        }
        out.push(WordAnal {
            morphs,
            w_byte_num: w_byte_num as u16,
            word_cvc,
            b_word_sen,
            b_sentence_end: w.b_sentence_end,
            source: w.source.clone(),
        });
    }
    out
}
