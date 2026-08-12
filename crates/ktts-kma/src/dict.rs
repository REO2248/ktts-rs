use crate::{KmaError, KmaResult};
use ktts_dict::common::DataMap;
use ktts_dict::kmorph::{self, KmorphDict, SKeyItem};
use ktts_dict::posngram::{self, PosNGram, PosUniGram};
use ktts_dict::pronsec::{self, RecordValue, SectionDict};
use ktts_dict::tosplit::{self, IrregulePredDic, ToSplitDic};
use ktts_dict::wordgram::{self, NameGram, WordTriGram};
use std::collections::HashMap;
use std::path::Path;

pub(crate) const KMA_DICT_FILE_RELS: [&str; 22] = [
    "KMPADict/kmorph_hash.bin",
    "KMPADict/kmorph_hash.dic",
    "KMPADict/kmorph.dic",
    "KMPADict/POSUniGram.bin",
    "KMPADict/POSBigram.bin",
    "KMPADict/POSTrigram.bin",
    "KMPADict/WordTriGram.bin",
    "KMPADict/WordTriGram.dic",
    "KMPADict/HKNamegram.bin",
    "KMPADict/HCBigram.bin",
    "KMPADict/ToSplit.bin",
    "KMPADict/IrregulePred.bin",
    "PronDict/prepron.bin",
    "PronDict/unipron.bin",
    "PronDict/strpron.bin",
    "PronDict/unihanja2korea.bin",
    "user.bin",
    "EngDict/unienglishpron.bin",
    "EngDict/engsym.bin",
    "EngDict/englishpyogi.dic",
    "EngDict/englishpyogi_hash.dic",
    "EngDict/englishpyogi_hash.bin",
];

#[derive(Debug, Clone)]
pub struct UserDic {
    entries: Vec<(Vec<u16>, Vec<u16>)>,
    tree: Vec<(u32, i32, i32)>,
}

impl UserDic {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            entries: Vec::new(),
            tree: Vec::new(),
        }
    }

    /// Loads the user dictionary from a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary data is missing or malformed.
    pub fn load(path: &Path) -> KmaResult<Self> {
        let data = std::fs::read(path).map_err(|e| KmaError(format!("user.bin: {e}")))?;
        Self::load_bytes(&data)
    }

    /// Loads the user dictionary from bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary data is missing or malformed.
    pub fn load_bytes(data: &[u8]) -> KmaResult<Self> {
        let sd = pronsec::parse_user(data).map_err(|e| KmaError(format!("user.bin: {e}")))?;
        Self::from_section(&sd)
    }

    #[expect(
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        reason = "C port: index/math casts with wrap semantics"
    )]
    /// Builds the user dictionary from a parsed section.
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary data is missing or malformed.
    pub fn from_section(sd: &SectionDict) -> KmaResult<Self> {
        let n_rec = sd.num_records();
        let n_tree = sd.num_tree_nodes();
        if n_tree < n_rec {
            return Err(KmaError(format!(
                "user.bin: tree node count {n_tree} < record count {n_rec}"
            )));
        }
        let mut entries = Vec::with_capacity(n_rec);
        for (i, n) in sd.tree.iter().take(n_rec).enumerate() {
            let src = pool_u16_str(&sd.key_pool, n.value as usize).ok_or_else(|| {
                KmaError(format!(
                    "user.bin: record {i} key reference {}(u16) is outside the pool",
                    n.value
                ))
            })?;
            let tgt = pool_u16_str(&sd.value_pool, n.left as usize).ok_or_else(|| {
                KmaError(format!(
                    "user.bin: record {i} value reference {}(u16) is outside the pool",
                    n.left
                ))
            })?;
            entries.push((src, tgt));
        }
        let mut tree = Vec::with_capacity(n_tree);
        for n in sd.tree.iter().skip(n_rec) {
            tree.push((n.value, n.left, n.right));
        }
        for r in &sd.records {
            let left = match r.value {
                RecordValue::PoolRef(v) => v as i32,
                _ => -1,
            };
            tree.push((r.key_ref, left, r.next));
        }
        if tree.len() != n_tree {
            return Err(KmaError(format!(
                "user.bin: tree node count mismatch (expected {n_tree}, got {})",
                tree.len()
            )));
        }
        Ok(Self { entries, tree })
    }

    #[must_use]
    pub fn phrase_lookup(&self, key: &[u16]) -> Option<usize> {
        if self.entries.is_empty() || self.tree.is_empty() {
            return None;
        }
        let idx = aptree_search(&self.tree, key)?;
        let (src, _) = &self.entries[idx];
        if wcscmp_left(key, src) == 0 {
            Some(idx)
        } else {
            None
        }
    }

    #[must_use]
    pub fn entry(&self, idx: usize) -> (&[u16], &[u16]) {
        let (s, t) = &self.entries[idx];
        (s, t)
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn test_bit_w(key: &[u16], bit_place: u32) -> bool {
    let idx = (bit_place >> 4) as usize;
    if idx >= key.len() {
        return false;
    }
    let mask = 0x8000u16 >> (bit_place & 0xf);
    key[idx] & mask != 0
}

#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn aptree_search(tree: &[(u32, i32, i32)], key: &[u16]) -> Option<usize> {
    let mut node = tree.len() - 1;
    loop {
        let (value, left, right) = tree[node];
        if left == -1 && right == -1 {
            return Some(value as usize);
        }
        if test_bit_w(key, value) {
            node = right as usize;
        } else {
            node = left as usize;
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "C port: index/math casts with wrap semantics"
)]
fn wcscmp_left(one: &[u16], two: &[u16]) -> i32 {
    if one.len() < two.len() {
        return 1;
    }
    let mut i = 0usize;
    while i < two.len() && one[i] == two[i] {
        i += 1;
    }
    if i == two.len() { 0 } else { (i + 1) as i32 }
}

fn pool_u16_str(pool: &[u8], off: usize) -> Option<Vec<u16>> {
    let start = off.checked_mul(2)?;
    let b = pool.get(start..)?;
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 1 < b.len() {
        let u = u16::from_le_bytes([b[i], b[i + 1]]);
        if u == 0 {
            break;
        }
        out.push(u);
        i += 2;
    }
    Some(out)
}

#[derive(Debug, Clone)]
pub struct KAnalInfo {
    pub irr_type: u8,
    pub ch_pumsa: u8,
    pub ch_con_type: u8,
    pub d_part_prob: f64,
    pub d_word_prob: f64,
    pub un_to_info: u32,
    pub irr_string: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct EngDicts {
    pub unienglishpron: Option<SectionDict>,
    pub engsym: Option<SectionDict>,
    pub english: Option<ktts_dict::english::EnglishPyogiSet>,
    unienglishpron_map: HashMap<Vec<u8>, usize>,
    engsym_map: HashMap<Vec<u8>, usize>,
}

impl EngDicts {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            unienglishpron: None,
            engsym: None,
            english: None,
            unienglishpron_map: HashMap::new(),
            engsym_map: HashMap::new(),
        }
    }

    #[must_use]
    pub fn from_parts(
        unienglishpron: Option<SectionDict>,
        engsym: Option<SectionDict>,
        english: Option<ktts_dict::english::EnglishPyogiSet>,
    ) -> Self {
        let unienglishpron_map = unienglishpron
            .as_ref()
            .map(|d| {
                let mut m = HashMap::with_capacity(d.num_records());
                for i in 0..d.num_records() {
                    if let Some(k) = d.key_bytes(i) {
                        m.insert(k.to_vec(), i);
                    }
                }
                m
            })
            .unwrap_or_default();
        let engsym_map = engsym
            .as_ref()
            .map(|d| {
                let mut m = HashMap::with_capacity(d.num_records());
                for i in 0..d.num_records() {
                    if let Some(k) = d.key_bytes(i) {
                        m.insert(k.to_vec(), i);
                    }
                }
                m
            })
            .unwrap_or_default();
        Self {
            unienglishpron,
            engsym,
            english,
            unienglishpron_map,
            engsym_map,
        }
    }

    #[must_use]
    pub fn unienglishpron_lookup(&self, key: &[u8]) -> Option<String> {
        let d = self.unienglishpron.as_ref()?;
        let i = *self.unienglishpron_map.get(key)?;
        d.value_string(i)
    }

    #[must_use]
    pub fn engsym_code(&self, key: &[u8]) -> Option<u8> {
        let d = self.engsym.as_ref()?;
        let i = *self.engsym_map.get(key)?;
        d.code(i)
    }

    #[must_use]
    pub fn english_lookup(&self, word: &[u8]) -> Option<Vec<u8>> {
        let set = self.english.as_ref()?;
        set.lookup(word).map(<[u8]>::to_vec)
    }

    #[must_use]
    pub const fn is_loaded(&self) -> bool {
        self.unienglishpron.is_some() && self.engsym.is_some() && self.english.is_some()
    }
}

#[derive(Debug)]
pub struct KmaDicts {
    pub kmorph: KmorphDict<'static>,
    pub pos: PosNGram<'static>,
    pub wordtri: WordTriGram<'static>,
    pub namegram: NameGram<'static>,
    pub hcbigram: wordgram::CharGram<'static>,
    pub tosplit: ToSplitDic,
    pub irr_pred: IrregulePredDic,
    offset_to_pattern: HashMap<u32, usize>,
    pub prepron: HashMap<Vec<u16>, u8>,
    pub unipron: HashMap<Vec<u16>, Vec<u8>>,
    pub strpron: HashMap<Vec<u16>, Vec<u8>>,
    pub user_dic: UserDic,
    pub hanja: ktts_dict::hanja::Hanja2Korea,
    pub eng: EngDicts,
}

fn leak(v: Vec<u8>) -> &'static [u8] {
    Box::leak(v.into_boxed_slice())
}

impl KmaDicts {
    /// Loads all KMA dictionaries from a directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary data is missing or malformed.
    pub fn load(dir: &Path) -> KmaResult<Self> {
        let mut files: DataMap = HashMap::new();
        for rel in KMA_DICT_FILE_RELS {
            if let Ok(data) = std::fs::read(dir.join(rel)) {
                files.insert(format!("KLangDic/{rel}"), data);
            }
        }
        Self::load_bytes(&files)
    }

    /// Loads all KMA dictionaries from a data map.
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary data is missing or malformed.
    pub fn load_bytes(files: &DataMap) -> KmaResult<Self> {
        load_bytes_inner(files, "KLangDic/")
    }
}

fn load_bytes_inner(files: &DataMap, prefix: &str) -> KmaResult<KmaDicts> {
    let get = |rel: &str| -> KmaResult<Vec<u8>> {
        files
            .get(&format!("{prefix}{rel}"))
            .cloned()
            .ok_or_else(|| KmaError(format!("{rel}: no key in the data map")))
    };
    let kmp = "KMPADict/";
    let kmorph = kmorph::parse(
        leak(get(&format!("{kmp}kmorph_hash.bin"))?),
        leak(get(&format!("{kmp}kmorph_hash.dic"))?),
        leak(get(&format!("{kmp}kmorph.dic"))?),
    )
    .map_err(|e| KmaError(format!("kmorph: {e}")))?;
    let pos = posngram::parse(
        leak(get(&format!("{kmp}POSUniGram.bin"))?),
        leak(get(&format!("{kmp}POSBigram.bin"))?),
        leak(get(&format!("{kmp}POSTrigram.bin"))?),
    )
    .map_err(|e| KmaError(format!("posngram: {e}")))?;
    let wordtri = wordgram::parse(
        leak(get(&format!("{kmp}WordTriGram.bin"))?),
        leak(get(&format!("{kmp}WordTriGram.dic"))?),
    )
    .map_err(|e| KmaError(format!("wordgram: {e}")))?;
    let namegram = wordgram::parse_hkname(leak(get(&format!("{kmp}HKNamegram.bin"))?))
        .map_err(|e| KmaError(format!("hknamegram: {e}")))?;
    let hcbigram = wordgram::parse_hc(leak(get(&format!("{kmp}HCBigram.bin"))?))
        .map_err(|e| KmaError(format!("hcbigram: {e}")))?;
    let tosplit = tosplit::parse_to_split(&get(&format!("{kmp}ToSplit.bin"))?)
        .map_err(|e| KmaError(format!("tosplit: {e}")))?;
    let irr_pred = tosplit::parse_irregule_pred(&get(&format!("{kmp}IrregulePred.bin"))?)
        .map_err(|e| KmaError(format!("irregulepred: {e}")))?;
    let offset_to_pattern = wordtri
        .dic_offsets
        .iter()
        .enumerate()
        .map(|(i, &o)| (o, i))
        .collect();
    let prepron = load_section_map_bytes(
        files,
        prefix,
        "PronDict/prepron.bin",
        pronsec::parse_prepron,
        |sec, i| sec.code(i).map(|c| (key_u16(sec, i), c)),
    );
    let unipron = load_section_map_bytes(
        files,
        prefix,
        "PronDict/unipron.bin",
        pronsec::parse_unipron,
        |sec, i| {
            sec.value_string(i)
                .map(|v| (key_u16(sec, i), v.into_bytes()))
        },
    );
    let strpron = load_section_map_bytes(
        files,
        prefix,
        "PronDict/strpron.bin",
        pronsec::parse_strpron,
        |sec, i| {
            sec.value_string(i)
                .map(|v| (key_u16(sec, i), v.into_bytes()))
        },
    );
    let hanja = ktts_dict::hanja::parse(&get("PronDict/unihanja2korea.bin")?)
        .map_err(|e| KmaError(format!("unihanja2korea.bin: {e}")))?;
    let user_dic = match files
        .get(&format!("{prefix}user.bin"))
        .map(|d| UserDic::load_bytes(d))
    {
        Some(Ok(u)) => u,
        Some(Err(e)) => {
            eprintln!(
                "[ktts-kma] warning: cannot load user.bin ({e}) — continuing without the user dictionary"
            );
            UserDic::empty()
        }
        None => UserDic::empty(),
    };
    let eng = load_eng_dicts(files, prefix);
    Ok(KmaDicts {
        kmorph,
        pos,
        wordtri,
        namegram,
        hcbigram,
        tosplit,
        irr_pred,
        offset_to_pattern,
        prepron,
        unipron,
        strpron,
        user_dic,
        hanja,
        eng,
    })
}

fn load_eng_dicts(files: &DataMap, prefix: &str) -> EngDicts {
    let get = |rel: &str| files.get(&format!("{prefix}{rel}"));
    let Some(unienglishpron) = get("EngDict/unienglishpron.bin").and_then(|d| {
        pronsec::parse_unienglishpron(d)
            .map_err(|e| eprintln!("[ktts-kma] warning: unienglishpron.bin: {e}"))
            .ok()
    }) else {
        return EngDicts::empty();
    };
    let Some(engsym) = get("EngDict/engsym.bin").and_then(|d| {
        pronsec::parse_engsym(d)
            .map_err(|e| eprintln!("[ktts-kma] warning: engsym.bin: {e}"))
            .ok()
    }) else {
        return EngDicts::empty();
    };
    let Some(english) = (|| {
        let table = ktts_dict::english::parse_pron_table(get("EngDict/englishpyogi.dic")?).ok()?;
        let hash_dic =
            ktts_dict::english::parse_hash_dic(get("EngDict/englishpyogi_hash.dic")?).ok()?;
        let hash_bin =
            ktts_dict::english::parse_hash_bin(get("EngDict/englishpyogi_hash.bin")?).ok()?;
        ktts_dict::english::EnglishPyogiSet::new(table, hash_dic, hash_bin).ok()
    })() else {
        return EngDicts::empty();
    };
    EngDicts::from_parts(Some(unienglishpron), Some(engsym), Some(english))
}

fn load_section_map_bytes<T>(
    files: &DataMap,
    prefix: &str,
    rel: &str,
    parse: fn(&[u8]) -> Result<SectionDict, ktts_dict::common::DictError>,
    f: impl Fn(&SectionDict, usize) -> Option<(Vec<u16>, T)>,
) -> HashMap<Vec<u16>, T> {
    let Some(data) = files.get(&format!("{prefix}{rel}")) else {
        return HashMap::new();
    };
    let Ok(sec) = parse(data) else {
        return HashMap::new();
    };
    let mut m = HashMap::new();
    for i in 0..sec.num_records() {
        if let Some((k, v)) = f(&sec, i) {
            m.insert(k, v);
        }
    }
    m
}

fn key_u16(sec: &SectionDict, rec: usize) -> Vec<u16> {
    match sec.key_bytes(rec) {
        Some(b) if sec.key_unit == pronsec::PoolUnit::U16 => b
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect(),
        Some(b) => b.iter().map(|&x| u16::from(x)).collect(),
        None => Vec::new(),
    }
}

impl KmaDicts {
    #[must_use]
    pub fn search_kma_dict(&self, key: &[u8]) -> Vec<KAnalInfo> {
        let Some(rec) = self.kmorph.lookup(key) else {
            return Vec::new();
        };
        let uni = &self.pos.uni;
        let mut out: Vec<KAnalInfo> = Vec::with_capacity(rec.items.len());
        for it in &rec.items {
            let n_freq = it.n_freq.max(2);
            let ch_pumsa = it.ch_pos;
            let ch_con_type = if crate::tables::is_k_voice_yong_yon(ch_pumsa) {
                it.ch_irr_pred
            } else {
                b'0'
            };
            let un_to_info = if crate::tables::is_to(ch_pumsa) {
                it.un_ui_link
            } else {
                0
            };
            out.push(KAnalInfo {
                irr_type: b'T',
                ch_pumsa,
                ch_con_type,
                d_part_prob: f64::from(n_freq),
                d_word_prob: get_word_prob(uni, key, ch_pumsa, n_freq),
                un_to_info,
                irr_string: Vec::new(),
            });
        }
        if !rec.text.is_empty() {
            out.push(KAnalInfo {
                irr_type: b'R',
                ch_pumsa: 0,
                ch_con_type: 0,
                d_part_prob: 0.0,
                d_word_prob: 0.0,
                un_to_info: 0,
                irr_string: rec.text.to_vec(),
            });
        }
        out
    }

    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "C port: index/math casts with wrap semantics"
    )]
    pub fn pos_bigram(&self, ltag: u8, rtag: u8) -> f64 {
        let li = ltag.wrapping_sub(b'0') as usize;
        let ri = rtag.wrapping_sub(b'0') as usize;
        self.pos.bi.get(li as u8, ri as u8)
    }

    #[must_use]
    pub fn trigram(&self, key: &[u8]) -> Option<f64> {
        self.pos.tri.lookup(key).map(f64::from)
    }

    #[must_use]
    pub fn wordgram_lookup(&self, key: &[u8], ch_type: u8) -> Option<u32> {
        let mut k = Vec::with_capacity(key.len() + 1);
        k.push(ch_type);
        k.extend_from_slice(key);
        let i = self.wordtri.lookup(&k)?;
        Some(self.wordtri.dic_offsets[i])
    }

    #[must_use]
    pub fn search_pattern(
        &self,
        offset: u32,
        ims: [Option<&crate::ma::MorphNode>; 3],
        ini_ll_str: &[u8],
        ini_str: &[u8],
        gb_ini_ll_tag: u8,
        gb_ini_tag: u8,
    ) -> f64 {
        let Some(&pi) = self.offset_to_pattern.get(&offset) else {
            return 0.0;
        };
        let patterns = &self.wordtri.patterns[pi];
        for p in patterns {
            let mut ok = true;
            for (slot, node) in p.pyogi.iter().zip(ims.iter()) {
                if slot.is_empty() {
                    continue;
                }
                match node {
                    None => {
                        if slot.as_slice() != ini_ll_str && slot.as_slice() != ini_str {
                            ok = false;
                            break;
                        }
                    }
                    Some(n) => {
                        if slot.as_slice() != n.pyogi.as_slice() {
                            ok = false;
                            break;
                        }
                    }
                }
            }
            if ok {
                for (slot, node) in p.pos.iter().zip(ims.iter()) {
                    if *slot == 0 {
                        continue;
                    }
                    match node {
                        None => {
                            if *slot != gb_ini_ll_tag && *slot != gb_ini_tag {
                                ok = false;
                                break;
                            }
                        }
                        Some(n) => {
                            if *slot != n.ch_tag {
                                ok = false;
                                break;
                            }
                        }
                    }
                }
            }
            if ok {
                return f64::from(p.prob);
            }
        }
        0.0
    }

    #[must_use]
    pub fn namegram(&self, key: &[u16]) -> bool {
        let key_bytes: Vec<u8> = key.iter().flat_map(|w| w.to_le_bytes()).collect();
        self.namegram
            .aptree
            .search_w(key)
            .is_some_and(|i| self.namegram.aptree.key_w_at(i).starts_with(&key_bytes))
    }

    #[must_use]
    pub fn namegram_match_len(&self, key: &[u16]) -> usize {
        let key_bytes: Vec<u8> = key.iter().flat_map(|w| w.to_le_bytes()).collect();
        self.namegram.aptree.search_w(key).map_or(0, |i| {
            let e = self.namegram.aptree.key_w_at(i);
            if key_bytes.starts_with(e) {
                e.len() / 2
            } else {
                0
            }
        })
    }

    #[must_use]
    pub fn chargram(&self, key: &[u16]) -> Option<f32> {
        self.hcbigram.lookup(key)
    }

    #[must_use]
    pub fn irr_pred_search(&self, stem: &[u8]) -> Option<Vec<u8>> {
        let sec = self.irr_pred.section(stem.len().saturating_sub(1));
        let mut lo = 0usize;
        let mut hi = sec.len();
        while lo < hi {
            let mid = usize::midpoint(lo, hi);
            let name = sec[mid].stem.as_bytes();
            let cmp = stem.cmp(&name[..name.len().min(stem.len())]);
            match cmp {
                std::cmp::Ordering::Equal => {
                    return Some(sec[mid].conditions().to_vec());
                }
                std::cmp::Ordering::Less => hi = mid,
                std::cmp::Ordering::Greater => lo = mid + 1,
            }
        }
        None
    }

    #[must_use]
    pub fn to_struct_search(&self, pyogi: &[u8], n_cmp: usize) -> u8 {
        let sec = self.tosplit.section(n_cmp.saturating_sub(1));
        let mut lo = 0usize;
        let mut hi = sec.len();
        while lo < hi {
            let mid = usize::midpoint(lo, hi);
            let name = sec[mid].name.as_bytes();
            let cmp =
                pyogi[..pyogi.len().min(name.len())].cmp(&name[..name.len().min(pyogi.len())]);
            match cmp {
                std::cmp::Ordering::Equal => return sec[mid].pos,
                std::cmp::Ordering::Less => hi = mid,
                std::cmp::Ordering::Greater => lo = mid + 1,
            }
        }
        0
    }

    #[must_use]
    pub const fn uni_pron_lookup(&self, _key: &[u16]) -> bool {
        false
    }
}

#[must_use]
pub fn get_word_prob(uni: &PosUniGram, word: &[u8], tag: u8, n_bindo: u32) -> f64 {
    posngram::get_word_prob(uni, word, tag, n_bindo)
}

#[must_use]
pub const fn key_item_ch_pos(it: &SKeyItem) -> u8 {
    it.ch_pos
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::float_cmp,
        reason = "oracle assertions use exact float equality"
    )]
    use super::*;

    fn data_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(
            std::env::var("KTTSDB_DIR")
                .expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
        )
        .join("KLangDic")
    }

    #[test]
    fn user_dic_real_file() {
        let u = UserDic::load(&data_dir().join("user.bin")).expect("failed to load user.bin");
        assert_eq!(u.len(), 21);
        let u16s = |s: &str| s.encode_utf16().collect::<Vec<u16>>();
        assert_eq!(
            u.entry(0),
            (
                u16s("D.P.R.K").as_slice(),
                u16s("조선민주주의인민공화국").as_slice()
            )
        );
        assert_eq!(
            u.entry(6),
            (u16s("MS-DOS").as_slice(), u16s("엠에쓰도스").as_slice())
        );
        assert_eq!(
            u.entry(14),
            (u16s("C++").as_slice(), u16s("씨쁠라스 쁠라스").as_slice())
        );
        assert_eq!(
            u.entry(17),
            (u16s("KCC").as_slice(), u16s("조선콤퓨터쎈터").as_slice())
        );
        assert_eq!(
            u.entry(20),
            (u16s("<5027>").as_slice(), u16s("오공이칠").as_slice())
        );
    }

    #[test]
    fn user_dic_phrase_lookup() {
        let u = UserDic::load(&data_dir().join("user.bin")).expect("failed to load user.bin");
        let u16s = |s: &str| s.encode_utf16().collect::<Vec<u16>>();
        let idx = u.phrase_lookup(&u16s("D.P.R.K")).expect("D.P.R.K match");
        assert_eq!(u.entry(idx).1, u16s("조선민주주의인민공화국").as_slice());
        let idx = u
            .phrase_lookup(&u16s("D.P.R.K와"))
            .expect("D.P.R.K와 match");
        assert_eq!(u.entry(idx).1, u16s("조선민주주의인민공화국").as_slice());
        assert!(u.phrase_lookup(&u16s("MS-DOS는")).is_some());
        assert!(u.phrase_lookup(&u16s("내MS-DOS")).is_none());
        assert!(u.phrase_lookup(&u16s("XYZ")).is_none());
        assert!(u.phrase_lookup(&u16s("")).is_none());
        let idx = u.phrase_lookup(&u16s("KCC컴퓨터")).expect("KCC match");
        assert_eq!(u.entry(idx).1, u16s("조선콤퓨터쎈터").as_slice());
        let idx = u.phrase_lookup(&u16s("sp")).expect("sp match");
        assert_eq!(u.entry(idx).1, u16s("에쓰피").as_slice());
    }

    #[test]
    fn user_dic_empty() {
        let u = UserDic::empty();
        assert!(u.is_empty());
        assert!(
            u.phrase_lookup(&"D.P.R.K".encode_utf16().collect::<Vec<u16>>())
                .is_none()
        );
    }

    #[test]
    fn load_and_lookup() {
        let d = KmaDicts::load(&data_dir()).expect("failed to load the dictionary");
        let a = d.search_kma_dict(b"aNnye*");
        let two = a.iter().find(|x| x.ch_pumsa == b'2').expect("'2' entry");
        assert_eq!(two.d_part_prob, 386.0);
        assert!(
            (two.d_word_prob - (-9.9591)).abs() < 1e-3,
            "dWordProb={}",
            two.d_word_prob
        );
        let ha = d.search_kma_dict(b"ha");
        let dd = ha.iter().find(|x| x.ch_pumsa == b'D').expect("'D' entry");
        assert!(
            (dd.d_word_prob - (-0.020_202_7)).abs() < 1e-4,
            "{}",
            dd.d_word_prob
        );
        let s9yo = d.search_kma_dict(b"s9yo");
        let hat = s9yo.iter().find(|x| x.ch_pumsa == b'^').expect("'^' entry");
        assert!(
            (hat.d_word_prob - (-8.421)).abs() < 1e-3,
            "{}",
            hat.d_word_prob
        );
        assert_eq!(hat.un_to_info, 0x1c8);
        assert!((d.pos_bigram(b'j', b'2') - (-6.983_98)).abs() < 1e-4);
        assert!((d.pos_bigram(b'^', b'k') - (-3.730_054)).abs() < 1e-4);
        assert!((d.trigram(b"j2D").unwrap() - (-1.268_457)).abs() < 1e-4);
        assert!((d.trigram(b"2D^").unwrap() - (-2.261_763)).abs() < 1e-4);
        let conds = d.irr_pred_search(b"s9").expect("s9 stem");
        assert_eq!(conds, vec![0x0d, 0x13]);
        assert_eq!(d.tosplit.total_entries(), 572);
        assert_eq!(d.irr_pred.total_entries(), 343);
    }

    #[test]
    fn load_bytes_equiv_path_load() {
        let dir = data_dir();
        let d1 = KmaDicts::load(&dir).expect("failed to load via path");
        let mut files: DataMap = HashMap::new();
        for rel in KMA_DICT_FILE_RELS {
            if let Ok(data) = std::fs::read(dir.join(rel)) {
                files.insert(format!("KLangDic/{rel}"), data);
            }
        }
        let d2 = KmaDicts::load_bytes(&files).expect("failed to load via bytes");

        for key in [b"aNnye*".as_slice(), b"ha", b"s9yo", b"gajog", b"jus9"] {
            let a = d1.search_kma_dict(key);
            let b = d2.search_kma_dict(key);
            assert_eq!(
                a.len(),
                b.len(),
                "key {:?}: candidate count",
                String::from_utf8_lossy(key)
            );
            for (x, y) in a.iter().zip(b.iter()) {
                assert_eq!(x.irr_type, y.irr_type);
                assert_eq!(x.ch_pumsa, y.ch_pumsa);
                assert_eq!(x.ch_con_type, y.ch_con_type);
                assert_eq!(x.d_part_prob, y.d_part_prob);
                assert!(
                    (x.d_word_prob - y.d_word_prob).abs() < 1e-12,
                    "dWordProb: {} vs {}",
                    x.d_word_prob,
                    y.d_word_prob
                );
                assert_eq!(x.un_to_info, y.un_to_info);
                assert_eq!(x.irr_string, y.irr_string);
            }
        }
        assert_eq!(d1.pos_bigram(b'j', b'2'), d2.pos_bigram(b'j', b'2'));
        assert_eq!(d1.trigram(b"j2D"), d2.trigram(b"j2D"));
        assert_eq!(d1.trigram(b"2D^"), d2.trigram(b"2D^"));
        assert_eq!(d1.irr_pred_search(b"s9"), d2.irr_pred_search(b"s9"));
        assert_eq!(d1.to_struct_search(b"i", 1), d2.to_struct_search(b"i", 1));
        assert_eq!(
            d1.wordgram_lookup(b"ha", b'2'),
            d2.wordgram_lookup(b"ha", b'2')
        );
        let u16s = |s: &str| s.encode_utf16().collect::<Vec<u16>>();
        assert_eq!(d1.namegram(&u16s("김일성")), d2.namegram(&u16s("김일성")));
        assert_eq!(
            d1.namegram_match_len(&u16s("김일성")),
            d2.namegram_match_len(&u16s("김일성"))
        );
        assert_eq!(d1.chargram(&u16s("한국")), d2.chargram(&u16s("한국")));
        assert_eq!(d1.prepron, d2.prepron);
        assert_eq!(d1.unipron, d2.unipron);
        assert_eq!(d1.strpron, d2.strpron);
        assert_eq!(d1.user_dic.len(), d2.user_dic.len());
        for i in 0..d1.user_dic.len() {
            assert_eq!(d1.user_dic.entry(i), d2.user_dic.entry(i), "user entry {i}");
        }
        assert_eq!(
            d1.user_dic.phrase_lookup(&u16s("D.P.R.K와")),
            d2.user_dic.phrase_lookup(&u16s("D.P.R.K와"))
        );
    }
}
