use std::collections::HashMap;
use std::path::Path;

use ktts_dict::common::DataMap;
use ktts_dict::english::EnglishPyogiSet;
use ktts_dict::hanja::Hanja2Korea;
use ktts_dict::pronrule::PronRuleDict;
use ktts_dict::pronsec::SectionDict;
use ktts_kma::dict::EngDicts;

pub(crate) const PRON_DICT_FILE_RELS: [&str; 12] = [
    "PronDict/pronrule.bin",
    "PronDict/strpron.bin",
    "PronDict/prepron.bin",
    "PronDict/unipron.bin",
    "PronDict/UniMorphModify.bin",
    "EngDict/unienglishpron.bin",
    "EngDict/engsym.bin",
    "user.bin",
    "PronDict/unihanja2korea.bin",
    "EngDict/englishpyogi.dic",
    "EngDict/englishpyogi_hash.dic",
    "EngDict/englishpyogi_hash.bin",
];

#[derive(Debug)]
pub struct PronError(pub String);

impl std::fmt::Display for PronError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for PronError {}

impl From<std::io::Error> for PronError {
    fn from(e: std::io::Error) -> Self {
        Self(format!("io: {e}"))
    }
}

impl From<ktts_dict::common::DictError> for PronError {
    fn from(e: ktts_dict::common::DictError) -> Self {
        Self(format!("dict: {e}"))
    }
}

pub type PronResult<T> = Result<T, PronError>;

fn section_map(d: &SectionDict) -> HashMap<Vec<u8>, usize> {
    let mut m = HashMap::with_capacity(d.num_records());
    for i in 0..d.num_records() {
        if let Some(k) = d.key_bytes(i) {
            m.insert(k.to_vec(), i);
        }
    }
    m
}

#[derive(Debug)]
pub struct PronContext {
    pub pronrule: Option<PronRuleDict>,
    pub strpron: Option<SectionDict>,
    pub prepron: Option<SectionDict>,
    pub unipron: Option<SectionDict>,
    pub morphmodify: Option<SectionDict>,
    pub user: Option<SectionDict>,
    pub hanja: Option<Hanja2Korea>,
    pub eng: EngDicts,

    strpron_map: HashMap<Vec<u8>, usize>,
    unipron_map: HashMap<Vec<u8>, usize>,
    prepron_map: HashMap<Vec<u8>, usize>,
    morphmodify_map: HashMap<Vec<u8>, usize>,
    user_map: HashMap<Vec<u8>, usize>,
    pub loaded_files: Vec<String>,
}

impl PronContext {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            pronrule: None,
            strpron: None,
            prepron: None,
            unipron: None,
            morphmodify: None,
            user: None,
            hanja: None,
            eng: EngDicts::empty(),
            strpron_map: HashMap::new(),
            unipron_map: HashMap::new(),
            prepron_map: HashMap::new(),
            morphmodify_map: HashMap::new(),
            user_map: HashMap::new(),
            loaded_files: Vec::new(),
        }
    }

    /// Loads the pronunciation dictionaries from a directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary data is missing or malformed.
    pub fn load(klang_dic: &Path) -> PronResult<Self> {
        let mut files: DataMap = HashMap::new();
        for rel in PRON_DICT_FILE_RELS {
            if let Ok(data) = std::fs::read(klang_dic.join(rel)) {
                files.insert(format!("KLangDic/{rel}"), data);
            }
        }
        Self::load_bytes(&files)
    }

    /// Loads the pronunciation dictionaries from a data map.
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary data is missing or malformed.
    pub fn load_bytes(files: &DataMap) -> PronResult<Self> {
        let mut ctx = Self::empty();
        let get = |rel: &str| files.get(&format!("KLangDic/{rel}"));

        macro_rules! load_opt {
            ($rel:expr, $field:ident, $parser:path) => {{
                if let Some(data) = get($rel) {
                    let v = $parser(data)?;
                    ctx.loaded_files.push(format!("KLangDic/{}", $rel));
                    ctx.$field = Some(v);
                }
            }};
        }
        load_opt!(
            "PronDict/pronrule.bin",
            pronrule,
            ktts_dict::pronrule::parse
        );
        load_opt!(
            "PronDict/strpron.bin",
            strpron,
            ktts_dict::pronsec::parse_strpron
        );
        load_opt!(
            "PronDict/prepron.bin",
            prepron,
            ktts_dict::pronsec::parse_prepron
        );
        load_opt!(
            "PronDict/unipron.bin",
            unipron,
            ktts_dict::pronsec::parse_unipron
        );
        load_opt!(
            "PronDict/UniMorphModify.bin",
            morphmodify,
            ktts_dict::pronsec::parse_morphmodify
        );
        load_opt!("user.bin", user, ktts_dict::pronsec::parse_user);
        load_opt!(
            "PronDict/unihanja2korea.bin",
            hanja,
            ktts_dict::hanja::parse
        );
        let unienglishpron = get("EngDict/unienglishpron.bin")
            .map(|d| ktts_dict::pronsec::parse_unienglishpron(d))
            .transpose()?;
        let engsym = get("EngDict/engsym.bin")
            .map(|d| ktts_dict::pronsec::parse_engsym(d))
            .transpose()?;
        let english = match (
            get("EngDict/englishpyogi.dic"),
            get("EngDict/englishpyogi_hash.dic"),
            get("EngDict/englishpyogi_hash.bin"),
        ) {
            (Some(tb), Some(hd), Some(hb)) => {
                let table = ktts_dict::english::parse_pron_table(tb)?;
                let hash_dic = ktts_dict::english::parse_hash_dic(hd)?;
                let hash_bin = ktts_dict::english::parse_hash_bin(hb)?;
                Some(EnglishPyogiSet::new(table, hash_dic, hash_bin)?)
            }
            _ => None,
        };
        if unienglishpron.is_some() || engsym.is_some() || english.is_some() {
            ctx.loaded_files.push("EngDict(3 files)".to_string());
        }
        ctx.eng = EngDicts::from_parts(unienglishpron, engsym, english);
        ctx.strpron_map = ctx.strpron.as_ref().map(section_map).unwrap_or_default();
        ctx.unipron_map = ctx.unipron.as_ref().map(section_map).unwrap_or_default();
        ctx.prepron_map = ctx.prepron.as_ref().map(section_map).unwrap_or_default();
        ctx.morphmodify_map = ctx
            .morphmodify
            .as_ref()
            .map(section_map)
            .unwrap_or_default();
        ctx.user_map = ctx.user.as_ref().map(section_map).unwrap_or_default();
        Ok(ctx)
    }

    #[must_use]
    pub fn strpron_lookup(&self, key: &[u8]) -> Option<String> {
        let d = self.strpron.as_ref()?;
        let i = *self.strpron_map.get(key)?;
        d.value_string(i)
    }

    #[must_use]
    pub fn unipron_lookup(&self, key: &[u8]) -> Option<String> {
        let d = self.unipron.as_ref()?;
        let i = *self.unipron_map.get(key)?;
        d.value_string(i)
    }

    #[must_use]
    pub fn prepron_code(&self, key: &[u8]) -> Option<u8> {
        let d = self.prepron.as_ref()?;
        let i = *self.prepron_map.get(key)?;
        d.code(i)
    }

    #[must_use]
    pub fn morphmodify_code(&self, key: &[u8]) -> Option<u8> {
        let d = self.morphmodify.as_ref()?;
        let i = *self.morphmodify_map.get(key)?;
        d.code(i)
    }

    #[must_use]
    pub fn unienglishpron_lookup(&self, key: &[u8]) -> Option<String> {
        self.eng.unienglishpron_lookup(key)
    }

    #[must_use]
    pub fn engsym_code(&self, key: &[u8]) -> Option<u8> {
        self.eng.engsym_code(key)
    }

    #[must_use]
    pub fn user_lookup(&self, text: &[u8]) -> Option<(usize, String)> {
        let entries = self.user_entries()?;
        let mut best: Option<(usize, String)> = None;
        for (key, val) in &entries {
            let k = key.encode_utf16().collect::<Vec<_>>();
            let kbytes = u16_bytes(&k);
            if kbytes.len() <= text.len()
                && text[..kbytes.len()] == kbytes[..]
                && best.as_ref().is_none_or(|(bl, _)| kbytes.len() > *bl)
            {
                best = Some((kbytes.len(), val.clone()));
            }
        }
        best
    }

    #[must_use]
    pub fn user_entries(&self) -> Option<Vec<(String, String)>> {
        let d = self.user.as_ref()?;
        let keys = split_u16_pool(&d.key_pool);
        let vals = split_u16_pool(&d.value_pool);
        let n = keys.len().min(vals.len());
        Some((0..n).map(|i| (keys[i].clone(), vals[i].clone())).collect())
    }

    #[must_use]
    pub fn english_lookup(&self, word: &[u8]) -> Option<Vec<u8>> {
        self.eng.english_lookup(word)
    }

    #[must_use]
    pub fn hanja_get(&self, cp: u16) -> Option<u16> {
        self.hanja.as_ref().map(|h| h.get(cp))
    }
}

fn u16_bytes(v: &[u16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 2);
    for &c in v {
        out.extend_from_slice(&c.to_le_bytes());
    }
    out
}

fn split_u16_pool(pool: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos + 1 < pool.len() {
        let mut end = pos;
        while end + 1 < pool.len() && !(pool[end] == 0 && pool[end + 1] == 0) {
            end += 2;
        }
        if end > pos {
            out.push(
                pool[pos..end]
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect::<Vec<_>>()
                    .iter()
                    .map(|&c| char::from_u32(u32::from(c)).unwrap_or('?'))
                    .collect(),
            );
        }
        pos = end + 2;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_split() {
        let mut pool = Vec::new();
        for s in ["abc", "de"] {
            for c in s.encode_utf16() {
                pool.extend_from_slice(&c.to_le_bytes());
            }
            pool.extend_from_slice(&[0, 0]);
        }
        assert_eq!(split_u16_pool(&pool), vec!["abc", "de"]);
    }

    #[allow(clippy::type_complexity)]
    #[test]
    fn load_bytes_equiv_path_load() {
        let dir = std::path::PathBuf::from(
            std::env::var("KTTSDB_DIR")
                .expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
        )
        .join("KLangDic");
        let c1 = PronContext::load(&dir).expect("path load failed");
        let mut files: DataMap = HashMap::new();
        for rel in PRON_DICT_FILE_RELS {
            if let Ok(data) = std::fs::read(dir.join(rel)) {
                files.insert(format!("KLangDic/{rel}"), data);
            }
        }
        let c2 = PronContext::load_bytes(&files).expect("bytes load failed");

        let r1 = c1.pronrule.as_ref().expect("pronrule (path)");
        let r2 = c2.pronrule.as_ref().expect("pronrule (bytes)");
        assert_eq!(r1.rules.len(), r2.rules.len());

        let pairs: [(
            &dyn Fn(&PronContext, &[u8]) -> Option<Vec<u8>>,
            &dyn Fn(&PronContext) -> Option<&SectionDict>,
        ); 7] = [
            (&|c, k| c.strpron_lookup(k).map(String::into_bytes), &|c| {
                c.strpron.as_ref()
            }),
            (&|c, k| c.unipron_lookup(k).map(String::into_bytes), &|c| {
                c.unipron.as_ref()
            }),
            (&|c, k| c.prepron_code(k).map(|v| vec![v]), &|c| {
                c.prepron.as_ref()
            }),
            (&|c, k| c.morphmodify_code(k).map(|v| vec![v]), &|c| {
                c.morphmodify.as_ref()
            }),
            (
                &|c, k| c.unienglishpron_lookup(k).map(String::into_bytes),
                &|c| c.eng.unienglishpron.as_ref(),
            ),
            (&|c, k| c.engsym_code(k).map(|v| vec![v]), &|c| {
                c.eng.engsym.as_ref()
            }),
            (
                &|c, k| c.user_lookup(k).map(|(_, s)| s.into_bytes()),
                &|c| c.user.as_ref(),
            ),
        ];
        for (lookup, field) in pairs {
            let d1 = field(&c1);
            let d2 = field(&c2);
            assert_eq!(d1.is_some(), d2.is_some());
            if let (Some(d1), Some(d2)) = (d1, d2) {
                assert_eq!(d1.num_records(), d2.num_records());
                for i in 0..d1.num_records() {
                    let Some(k) = d1.key_bytes(i) else { continue };
                    assert_eq!(
                        lookup(&c1, k),
                        lookup(&c2, k),
                        "lookup mismatch for record {i}"
                    );
                }
            }
        }
        for w in [
            b"KCC".as_slice(),
            b"KOREA",
            b"SEOUL",
            b"hello",
            b"one",
            b"XYZQ",
        ] {
            assert_eq!(c1.english_lookup(w), c2.english_lookup(w), "english {w:?}");
        }
        for cp in (0x4e00u16..0x4e40).chain([0x4e8c, 0x4e09, 0x56db, 0x65e5, 0x6708]) {
            assert_eq!(c1.hanja_get(cp), c2.hanja_get(cp), "hanja {cp:#x}");
        }
        assert_eq!(c1.user_entries(), c2.user_entries());
        assert_eq!(c1.loaded_files.len(), c2.loaded_files.len());
    }
}
