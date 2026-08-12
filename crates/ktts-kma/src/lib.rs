#![allow(clippy::needless_range_loop)]

pub mod charstr;
pub mod code;
pub mod dict;
pub mod digits;
pub mod eng_tables;
pub mod english;
pub mod ma;
pub mod post;
pub mod preproc;
pub mod tables;

use std::path::Path;

pub type DataMap = ktts_dict::common::DataMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KmaError(pub String);

impl std::fmt::Display for KmaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for KmaError {}

impl From<ktts_dict::common::DictError> for KmaError {
    fn from(e: ktts_dict::common::DictError) -> Self {
        Self(format!("dict: {e}"))
    }
}

pub type KmaResult<T> = Result<T, KmaError>;

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
    pub b_sentence_end: bool,
    pub source: Vec<u16>,
}

#[derive(Debug)]
pub struct KmaContext {
    pub d: dict::KmaDicts,
    pub(crate) tts_check_sentence_cut: bool,
}

/// Loads the KMA dictionaries from a directory.
///
/// # Errors
///
/// Returns an error if the input is invalid.
pub fn load_kma_dicts(dir: &Path) -> KmaResult<KmaContext> {
    let mut files: DataMap = std::collections::HashMap::new();
    for rel in dict::KMA_DICT_FILE_RELS {
        if let Ok(data) = std::fs::read(dir.join(rel)) {
            files.insert(format!("KLangDic/{rel}"), data);
        }
    }
    load_kma_dicts_bytes(&files)
}

/// Loads the KMA dictionaries from a data map.
///
/// # Errors
///
/// Returns an error if the input is invalid.
pub fn load_kma_dicts_bytes(files: &DataMap) -> KmaResult<KmaContext> {
    Ok(KmaContext {
        d: dict::KmaDicts::load_bytes(files)?,
        tts_check_sentence_cut: false,
    })
}

impl KmaContext {
    #[must_use]
    pub const fn eng_dicts(&self) -> &dict::EngDicts {
        &self.d.eng
    }
    #[must_use]
    pub const fn prepron(&self) -> &std::collections::HashMap<Vec<u16>, u8> {
        &self.d.prepron
    }
    #[must_use]
    pub const fn strpron(&self) -> &std::collections::HashMap<Vec<u16>, Vec<u8>> {
        &self.d.strpron
    }
    #[must_use]
    pub const fn unipron(&self) -> &std::collections::HashMap<Vec<u16>, Vec<u8>> {
        &self.d.unipron
    }
}

/// Morphologically analyzes a UTF-8 text.
///
/// # Errors
///
/// Returns an error if the input is invalid.
pub fn analyze(ctx: &KmaContext, text: &str) -> KmaResult<Vec<WordAnal>> {
    let input: Vec<u16> = text.encode_utf16().collect();
    let words = ma::klp_proc_all(ctx, &input)?;
    Ok(post::get_word_info(&words))
}
