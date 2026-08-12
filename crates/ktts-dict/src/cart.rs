use crate::common::{DictError, DictResult, Reader};

pub const LEAF_DURATION: usize = 1;
pub const LEAF_PITCH_F0: usize = 12;

#[derive(Debug, Clone, PartialEq)]
pub enum CartNodeKind {
    Leaf(Vec<f32>),
    Type1 { x: u8, y: u8 },
    Type2 { x: u8, set: Vec<u8> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CartNode {
    pub kind: CartNodeKind,
    pub left: Option<usize>,
    pub right: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CartTree {
    pub nodes: Vec<CartNode>,
    pub leaf_floats: usize,
}

impl CartTree {
    #[must_use]
    pub fn leaf_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| matches!(n.kind, CartNodeKind::Leaf(_)))
            .count()
    }
    #[must_use]
    pub fn internal_count(&self) -> usize {
        self.nodes.len() - self.leaf_count()
    }
    #[must_use]
    pub fn type1_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| matches!(n.kind, CartNodeKind::Type1 { .. }))
            .count()
    }
    #[must_use]
    pub fn type2_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| matches!(n.kind, CartNodeKind::Type2 { .. }))
            .count()
    }
    #[must_use]
    pub fn eval(&self, feat: &[f32]) -> Option<&[f32]> {
        let mut idx = 0usize;
        loop {
            let node = self.nodes.get(idx)?;
            match &node.kind {
                CartNodeKind::Leaf(v) => return Some(v),
                CartNodeKind::Type1 { x, y } => {
                    let f = *feat.get(*x as usize)?;
                    idx = if f > f32::from(*y) {
                        node.right?
                    } else {
                        node.left?
                    };
                }
                CartNodeKind::Type2 { x, set } => {
                    let f = *feat.get(*x as usize)?;
                    #[expect(clippy::float_cmp, reason = "exact CART threshold match (C port)")]
                    let hit = !f.is_nan() && set.iter().any(|&b| f == f32::from(b));
                    idx = if hit { node.left? } else { node.right? };
                }
            }
        }
    }
}

/// Parses a CART tree with the given leaf feature count.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse(data: &[u8], leaf_floats: usize) -> DictResult<CartTree> {
    let mut r = Reader::new(data);
    let mut nodes = Vec::new();
    parse_node(&mut r, leaf_floats, &mut nodes)?;
    if r.remaining() != 0 {
        return Err(DictError::new(
            format!("tree2 full consumption failed: {}B left", r.remaining()),
            r.pos,
        ));
    }
    Ok(CartTree { nodes, leaf_floats })
}

/// Parses the duration CART tree.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_duration(data: &[u8]) -> DictResult<CartTree> {
    parse(data, LEAF_DURATION)
}

/// Parses the `ToBI` CART tree.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_tobi(data: &[u8]) -> DictResult<CartTree> {
    parse(data, LEAF_DURATION)
}

/// Parses the pitch `f0` CART tree.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse_pitch_f0(data: &[u8]) -> DictResult<CartTree> {
    parse(data, LEAF_PITCH_F0)
}

fn parse_node(
    reader: &mut Reader<'_>,
    leaf_floats: usize,
    nodes: &mut Vec<CartNode>,
) -> DictResult<usize> {
    let node_type = reader.u8()?;
    let idx = nodes.len();
    match node_type {
        0 => {
            let mut v = Vec::with_capacity(leaf_floats);
            for _ in 0..leaf_floats {
                v.push(reader.f32()?);
            }
            nodes.push(CartNode {
                kind: CartNodeKind::Leaf(v),
                left: None,
                right: None,
            });
            Ok(idx)
        }
        1 => {
            let feat_idx = reader.u8()?;
            let thr_idx = reader.u8()?;
            nodes.push(CartNode {
                kind: CartNodeKind::Type1 {
                    x: feat_idx,
                    y: thr_idx,
                },
                left: None,
                right: None,
            });
            let left_idx = parse_node(reader, leaf_floats, nodes)?;
            let right_idx = parse_node(reader, leaf_floats, nodes)?;
            let node = &mut nodes[idx];
            node.left = Some(left_idx);
            node.right = Some(right_idx);
            Ok(idx)
        }
        2 => {
            let feat_idx = reader.u8()?;
            let set_len = reader.u8()?;
            let mut set = Vec::with_capacity(usize::from(set_len));
            for _ in 0..set_len {
                set.push(reader.u8()?);
            }
            nodes.push(CartNode {
                kind: CartNodeKind::Type2 { x: feat_idx, set },
                left: None,
                right: None,
            });
            let left_idx = parse_node(reader, leaf_floats, nodes)?;
            let right_idx = parse_node(reader, leaf_floats, nodes)?;
            let node = &mut nodes[idx];
            node.left = Some(left_idx);
            node.right = Some(right_idx);
            Ok(idx)
        }
        _ => Err(DictError::new(
            format!("unknown node type 0x{node_type:02x}"),
            reader.pos - 1,
        )),
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::float_cmp,
        reason = "oracle assertions use exact float equality"
    )]
    use super::*;

    fn tone_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(
            std::env::var("KTTSDB_DIR")
                .expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
        )
        .join("KSpeechDic")
        .join("woman")
        .join("tone")
    }

    fn read(p: &str) -> Vec<u8> {
        std::fs::read(p).unwrap_or_else(|e| panic!("read failed {p}: {e}"))
    }

    fn f32le(v: f32) -> [u8; 4] {
        v.to_le_bytes()
    }

    const TREES: &[(&str, usize, usize, usize, usize, usize, usize)] = &[
        ("Pitch_f0.tree2.bin", 12, 875, 437, 438, 81, 356),
        ("boundary_tobi.tree2.bin", 1, 3863, 1931, 1932, 380, 1551),
        (
            "non_boundary_tobi.tree2.bin",
            1,
            3291,
            1645,
            1646,
            488,
            1157,
        ),
        ("DURATION/di.tree2.bin", 1, 821, 410, 411, 21, 389),
        ("DURATION/glide.tree2.bin", 1, 795, 397, 398, 30, 367),
        ("DURATION/machal.tree2.bin", 1, 687, 343, 344, 18, 325),
        ("DURATION/mono.tree2.bin", 1, 863, 431, 432, 12, 419),
        ("DURATION/nasal.tree2.bin", 1, 905, 452, 453, 27, 425),
        ("DURATION/pachal.tree2.bin", 1, 845, 422, 423, 38, 384),
        ("DURATION/pahyul.tree2.bin", 1, 837, 418, 419, 26, 392),
    ];

    #[test]
    fn all_ten_trees_parse_and_consume() {
        for &(name, leaf_floats, total, internal, leaves, type1, type2) in TREES {
            let data = read(&format!("{}/{}", tone_dir().display(), name));
            let tree =
                parse(&data, leaf_floats).unwrap_or_else(|e| panic!("{name} parse failed: {e}"));
            assert_eq!(tree.nodes.len(), total, "{name} total node count");
            assert_eq!(
                tree.internal_count(),
                internal,
                "{name} internal node count"
            );
            assert_eq!(tree.leaf_count(), leaves, "{name} leaf count");
            assert_eq!(tree.type1_count(), type1, "{name} type1 count");
            assert_eq!(tree.type2_count(), type2, "{name} type2 count");
            assert_eq!(tree.leaf_floats, leaf_floats, "{name} leaf float count");
            assert_eq!(internal, leaves - 1, "{name} full binary tree");
        }
    }

    #[test]
    fn eval_type1_threshold() {
        let mut buf = vec![0x01, 5, 10, 0x00];
        buf.extend_from_slice(&f32le(1.0));
        buf.push(0x00);
        buf.extend_from_slice(&f32le(2.0));
        let tree = parse(&buf, 1).unwrap();
        assert_eq!(tree.nodes.len(), 3);
        let mut feat = [0f32; 41];
        feat[5] = 20.0;
        assert_eq!(tree.eval(&feat), Some(&[2.0f32][..]));
        feat[5] = 10.0;
        assert_eq!(tree.eval(&feat), Some(&[1.0f32][..]));
        feat[5] = 3.0;
        assert_eq!(tree.eval(&feat), Some(&[1.0f32][..]));
    }

    #[test]
    fn eval_type2_set() {
        let mut buf = vec![0x02, 17, 2, 3, 7, 0x00];
        buf.extend_from_slice(&f32le(1.0));
        buf.push(0x00);
        buf.extend_from_slice(&f32le(2.0));
        let tree = parse(&buf, 1).unwrap();
        let mut feat = [0f32; 41];
        feat[17] = 3.0;
        assert_eq!(tree.eval(&feat), Some(&[1.0f32][..]));
        feat[17] = 7.0;
        assert_eq!(tree.eval(&feat), Some(&[1.0f32][..]));
        feat[17] = 9.0;
        assert_eq!(tree.eval(&feat), Some(&[2.0f32][..]));
        feat[17] = f32::NAN;
        assert_eq!(tree.eval(&feat), Some(&[2.0f32][..]));
    }

    #[test]
    fn eval_pitch_f0_leaf_width() {
        let mut buf = Vec::new();
        buf.push(0x00);
        for i in 0u8..12 {
            buf.extend_from_slice(&f32le(100.0 + f32::from(i)));
        }
        let tree = parse(&buf, 12).unwrap();
        assert_eq!(tree.leaf_count(), 1);
        let v = tree.eval(&[0f32; 41]).unwrap();
        assert_eq!(v.len(), 12);
        assert_eq!(v[0], 100.0);
        assert_eq!(v[11], 111.0);
        assert!(parse(&buf, 1).is_err());
    }
}
