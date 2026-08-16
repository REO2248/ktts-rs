use crate::{ProsodyError, ProsodyResult};
use ktts_dict::birule::{self, BIProbDic, BIRuleFile};
use ktts_dict::cart::CartTree;
use ktts_dict::common::DataMap;
use std::path::Path;

pub const DURATION_TREE_NAMES: [&str; 7] =
    ["pahyul", "pachal", "machal", "nasal", "glide", "mono", "di"];

const PROSODY_DICT_FILE_RELS: [&str; 13] = [
    "Break/BIRule.bin",
    "Break/BIProb_hash.bin",
    "Break/BIProb_hash.dic",
    "tone/DURATION/pahyul.tree2.bin",
    "tone/DURATION/pachal.tree2.bin",
    "tone/DURATION/machal.tree2.bin",
    "tone/DURATION/nasal.tree2.bin",
    "tone/DURATION/glide.tree2.bin",
    "tone/DURATION/mono.tree2.bin",
    "tone/DURATION/di.tree2.bin",
    "tone/boundary_tobi.tree2.bin",
    "tone/non_boundary_tobi.tree2.bin",
    "tone/Pitch_f0.tree2.bin",
];

#[derive(Debug, Clone)]
pub struct ProsodyContext {
    pub birule: BIRuleFile,
    pub biprob: BIProbDic,
    pub dur_trees: [CartTree; 7],
    pub bound_tobi: CartTree,
    pub non_bound_tobi: CartTree,
    pub pitch_f0: CartTree,
    pub use_birule: bool,
}

impl ProsodyContext {
    #[must_use]
    pub const fn with_birule(mut self, on: bool) -> Self {
        self.use_birule = on;
        self
    }

    #[must_use]
    pub const fn data_dir(&self) -> &'static str {
        "KSpeechDic/<speaker>"
    }
}

/// Loads the prosody dictionaries from a directory.
///
/// # Errors
///
/// Returns an error if a dictionary file is missing or malformed.
pub fn load_prosody_dicts(dir: &Path) -> ProsodyResult<ProsodyContext> {
    let voice = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("speaker")
        .to_string();
    let mut files: DataMap = std::collections::HashMap::new();
    for rel in PROSODY_DICT_FILE_RELS {
        if let Ok(data) = std::fs::read(dir.join(rel)) {
            files.insert(format!("KSpeechDic/{voice}/{rel}"), data);
        }
    }
    load_prosody_dicts_bytes(&files, &voice)
}

/// Loads the prosody dictionaries from a data map.
///
/// # Errors
///
/// Returns an error if a dictionary file is missing or malformed.
pub fn load_prosody_dicts_bytes(files: &DataMap, voice: &str) -> ProsodyResult<ProsodyContext> {
    load_bytes_inner(files, &format!("KSpeechDic/{voice}/"))
}

fn load_bytes_inner(files: &DataMap, prefix: &str) -> ProsodyResult<ProsodyContext> {
    let get = |rel: &str| -> ProsodyResult<Vec<u8>> {
        files
            .get(&format!("{prefix}{rel}"))
            .cloned()
            .ok_or_else(|| ProsodyError(format!("{rel}: key missing from data map")))
    };

    let rule_data = get("Break/BIRule.bin")?;
    let birule =
        birule::parse_rule(&rule_data).map_err(|e| ProsodyError(format!("BIRule.bin: {e}")))?;
    let prob_bin_data = get("Break/BIProb_hash.bin")?;
    let prob_dic_data = get("Break/BIProb_hash.dic")?;
    let prob_bin = birule::parse_prob_bin(&prob_bin_data)
        .map_err(|e| ProsodyError(format!("BIProb_hash.bin: {e}")))?;
    let biprob = birule::parse_prob_dic(&prob_dic_data, &prob_bin.buckets)
        .map_err(|e| ProsodyError(format!("BIProb_hash.dic: {e}")))?;

    let mut dur_trees: [CartTree; 7] = [(); 7].map(|()| CartTree {
        nodes: Vec::new(),
        leaf_floats: 1,
    });
    for (i, name) in DURATION_TREE_NAMES.iter().enumerate() {
        let data = get(&format!("tone/DURATION/{name}.tree2.bin"))?;
        dur_trees[i] = ktts_dict::cart::parse_duration(&data)
            .map_err(|e| ProsodyError(format!("{name}: {e}")))?;
    }
    let bound_data = get("tone/boundary_tobi.tree2.bin")?;
    let bound_tobi = ktts_dict::cart::parse_tobi(&bound_data)
        .map_err(|e| ProsodyError(format!("boundary_tobi: {e}")))?;
    let non_bound_data = get("tone/non_boundary_tobi.tree2.bin")?;
    let non_bound_tobi = ktts_dict::cart::parse_tobi(&non_bound_data)
        .map_err(|e| ProsodyError(format!("non_boundary_tobi: {e}")))?;
    let pitch_data = get("tone/Pitch_f0.tree2.bin")?;
    let pitch_f0 = ktts_dict::cart::parse_pitch_f0(&pitch_data)
        .map_err(|e| ProsodyError(format!("Pitch_f0: {e}")))?;

    Ok(ProsodyContext {
        birule,
        biprob,
        dur_trees,
        bound_tobi,
        non_bound_tobi,
        pitch_f0,
        use_birule: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn woman_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(
            std::env::var("KTTSDB_DIR")
                .expect("set KTTSDB_DIR to the dictionary data (kttsdb) directory"),
        )
        .join("KSpeechDic")
        .join("woman")
    }

    #[test]
    fn load_full_context() {
        let ctx = load_prosody_dicts(&woman_dir()).expect("dictionary load failed");
        assert_eq!(ctx.birule.w_rule_num, 132);
        assert_eq!(ctx.birule.w_attrib_num, 79);
        assert_eq!(ctx.birule.attributes.len(), 79);
        assert_eq!(ctx.birule.attrib_item_count(), 453);
        assert_eq!(ctx.birule.rules.len(), 132);
        assert_eq!(ctx.birule.morph_count(), 304);
        assert_eq!(ctx.biprob.key_count, 56_153);
        let expected = [
            ("pahyul", 837usize),
            ("pachal", 845),
            ("machal", 687),
            ("nasal", 905),
            ("glide", 795),
            ("mono", 863),
            ("di", 821),
        ];
        for (i, (name, total)) in expected.iter().enumerate() {
            assert_eq!(
                ctx.dur_trees[i].nodes.len(),
                *total,
                "DURATION/{name} node count"
            );
        }
        assert_eq!(ctx.bound_tobi.nodes.len(), 3863, "boundary_tobi");
        assert_eq!(ctx.non_bound_tobi.nodes.len(), 3291, "non_boundary_tobi");
        assert_eq!(ctx.pitch_f0.nodes.len(), 875, "Pitch_f0");
        for t in &ctx.dur_trees {
            for n in &t.nodes {
                match &n.kind {
                    ktts_dict::cart::CartNodeKind::Type1 { x, .. }
                    | ktts_dict::cart::CartNodeKind::Type2 { x, .. } => assert!(*x <= 16),
                    ktts_dict::cart::CartNodeKind::Leaf(_) => {}
                }
            }
        }
        for t in [&ctx.bound_tobi, &ctx.non_bound_tobi] {
            for n in &t.nodes {
                match &n.kind {
                    ktts_dict::cart::CartNodeKind::Type1 { x, .. }
                    | ktts_dict::cart::CartNodeKind::Type2 { x, .. } => {
                        assert!((5..=31).contains(x));
                    }
                    ktts_dict::cart::CartNodeKind::Leaf(_) => {}
                }
            }
        }
        for n in &ctx.pitch_f0.nodes {
            match &n.kind {
                ktts_dict::cart::CartNodeKind::Type1 { x, .. }
                | ktts_dict::cart::CartNodeKind::Type2 { x, .. } => assert!((5..=40).contains(x)),
                ktts_dict::cart::CartNodeKind::Leaf(_) => {}
            }
        }
    }

    fn x_hist(t: &CartTree) -> std::collections::BTreeMap<u8, usize> {
        let mut m = std::collections::BTreeMap::new();
        for n in &t.nodes {
            match &n.kind {
                ktts_dict::cart::CartNodeKind::Type1 { x, .. }
                | ktts_dict::cart::CartNodeKind::Type2 { x, .. } => *m.entry(*x).or_insert(0) += 1,
                ktts_dict::cart::CartNodeKind::Leaf(_) => {}
            }
        }
        m
    }

    fn hist_from(entries: &[(u8, usize)]) -> std::collections::BTreeMap<u8, usize> {
        entries.iter().copied().collect()
    }

    #[test]
    #[expect(clippy::too_many_lines, reason = "large data-driven test")]
    fn tree_x_usage_matches_ph2_tree_section3() {
        let ctx = load_prosody_dicts(&woman_dir()).expect("dictionary load failed");
        let dur_expected: [&[(u8, usize)]; 7] = [
            &[
                (0, 63),
                (1, 35),
                (2, 91),
                (3, 8),
                (4, 13),
                (5, 18),
                (6, 8),
                (7, 14),
                (8, 18),
                (9, 15),
                (10, 17),
                (11, 29),
                (12, 40),
                (13, 23),
                (14, 23),
                (15, 2),
                (16, 1),
            ],
            &[
                (0, 50),
                (1, 13),
                (2, 62),
                (3, 8),
                (4, 7),
                (5, 20),
                (6, 23),
                (7, 19),
                (8, 34),
                (9, 23),
                (10, 30),
                (11, 26),
                (12, 45),
                (13, 24),
                (14, 26),
                (15, 7),
                (16, 5),
            ],
            &[
                (0, 61),
                (1, 5),
                (2, 53),
                (3, 6),
                (4, 11),
                (5, 15),
                (6, 10),
                (7, 25),
                (8, 21),
                (9, 12),
                (10, 23),
                (11, 27),
                (12, 39),
                (13, 17),
                (14, 15),
                (15, 2),
                (16, 1),
            ],
            &[
                (0, 100),
                (1, 27),
                (2, 70),
                (3, 8),
                (4, 11),
                (5, 18),
                (6, 18),
                (7, 19),
                (8, 16),
                (9, 11),
                (10, 21),
                (11, 28),
                (12, 42),
                (13, 36),
                (14, 21),
                (15, 4),
                (16, 2),
            ],
            &[
                (0, 54),
                (1, 3),
                (2, 45),
                (3, 12),
                (4, 13),
                (5, 26),
                (6, 19),
                (7, 26),
                (8, 38),
                (9, 25),
                (10, 34),
                (11, 21),
                (12, 36),
                (13, 15),
                (14, 20),
                (15, 8),
                (16, 2),
            ],
            &[
                (0, 108),
                (1, 60),
                (2, 99),
                (3, 13),
                (4, 13),
                (5, 6),
                (6, 9),
                (7, 13),
                (8, 10),
                (9, 3),
                (10, 7),
                (11, 15),
                (12, 40),
                (13, 23),
                (14, 7),
                (15, 4),
                (16, 1),
            ],
            &[
                (0, 64),
                (1, 61),
                (2, 93),
                (3, 13),
                (4, 5),
                (5, 17),
                (6, 21),
                (7, 14),
                (8, 12),
                (9, 9),
                (10, 11),
                (11, 18),
                (12, 26),
                (13, 25),
                (14, 14),
                (15, 2),
                (16, 5),
            ],
        ];
        for (i, exp) in dur_expected.iter().enumerate() {
            assert_eq!(
                x_hist(&ctx.dur_trees[i]),
                hist_from(exp),
                "DURATION/{} X usage list",
                DURATION_TREE_NAMES[i]
            );
        }
        let boundary_exp: &[(u8, usize)] = &[
            (5, 143),
            (6, 71),
            (7, 27),
            (8, 193),
            (9, 187),
            (10, 251),
            (11, 63),
            (12, 11),
            (13, 58),
            (14, 44),
            (15, 36),
            (16, 13),
            (17, 278),
            (18, 166),
            (19, 58),
            (20, 16),
            (21, 9),
            (22, 2),
            (23, 18),
            (24, 90),
            (25, 61),
            (26, 47),
            (29, 88),
            (31, 1),
        ];
        assert_eq!(
            x_hist(&ctx.bound_tobi),
            hist_from(boundary_exp),
            "boundary_tobi X usage list"
        );
        let non_bound_exp: &[(u8, usize)] = &[
            (5, 58),
            (6, 38),
            (7, 64),
            (8, 74),
            (9, 66),
            (10, 96),
            (11, 62),
            (12, 95),
            (13, 72),
            (14, 44),
            (15, 14),
            (16, 9),
            (17, 149),
            (18, 212),
            (19, 70),
            (20, 27),
            (21, 30),
            (22, 8),
            (23, 36),
            (24, 76),
            (25, 38),
            (26, 28),
            (27, 57),
            (28, 6),
            (29, 123),
            (30, 92),
            (31, 1),
        ];
        assert_eq!(
            x_hist(&ctx.non_bound_tobi),
            hist_from(non_bound_exp),
            "non_boundary_tobi X usage list"
        );
        let pitch_exp: &[(u8, usize)] = &[
            (5, 4),
            (6, 3),
            (7, 5),
            (8, 15),
            (9, 3),
            (10, 7),
            (11, 7),
            (12, 22),
            (13, 9),
            (14, 4),
            (17, 98),
            (18, 87),
            (19, 26),
            (20, 16),
            (21, 2),
            (22, 4),
            (23, 17),
            (24, 10),
            (25, 9),
            (26, 5),
            (27, 27),
            (29, 16),
            (30, 10),
            (33, 3),
            (36, 1),
            (37, 16),
            (38, 5),
            (39, 4),
            (40, 2),
        ];
        assert_eq!(
            x_hist(&ctx.pitch_f0),
            hist_from(pitch_exp),
            "Pitch_f0 X usage list"
        );
    }
}
