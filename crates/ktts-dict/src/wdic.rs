use crate::common::{DictError, DictResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wdic {
    pub entries: Vec<(Vec<u8>, Vec<u8>)>,
}

impl Wdic {
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.entries
            .iter()
            .find(|(k, _)| k == key.as_bytes())
            .map(|(_, v)| v.as_slice())
    }
    #[must_use]
    pub fn get_str(&self, key: &str) -> Option<String> {
        self.get(key)
            .map(|v| String::from_utf8_lossy(v).into_owned())
    }
}

fn trim_ascii(mut s: &[u8]) -> &[u8] {
    while let [first, rest @ ..] = s {
        if first.is_ascii_whitespace() {
            s = rest;
        } else {
            break;
        }
    }
    while let [rest @ .., last] = s {
        if last.is_ascii_whitespace() {
            s = rest;
        } else {
            break;
        }
    }
    s
}

/// Parses the word dictionary text.
///
/// # Errors
///
/// Returns an error if the data is malformed or truncated.
pub fn parse(data: &[u8]) -> DictResult<Wdic> {
    let mut entries = Vec::new();
    let mut off = 0usize;
    for (ln, line) in data.split(|&b| b == b'\n').enumerate() {
        let line_off = off;
        off += line.len() + 1;
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        let line = trim_ascii(line);
        if line.is_empty() {
            continue;
        }
        if line.first() != Some(&b'[') {
            return Err(DictError::new(
                format!(
                    "INI format violation (line {}): first char is not '['",
                    ln + 1
                ),
                line_off,
            ));
        }
        let close = line.iter().position(|&b| b == b']').ok_or_else(|| {
            DictError::new(
                format!("INI format violation (line {}): no ']'", ln + 1),
                line_off,
            )
        })?;
        let key = line[1..close].to_vec();
        let rest = trim_ascii(&line[close + 1..]);
        let value = rest
            .iter()
            .position(|&b| b == b'=')
            .map_or_else(Vec::new, |eq| {
                let v = trim_ascii(&rest[eq + 1..]);
                if v.first() == Some(&b'[') && v.last() == Some(&b']') {
                    v[1..v.len() - 1].to_vec()
                } else {
                    v.to_vec()
                }
            });
        entries.push((key, value));
    }
    Ok(Wdic { entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wdic_path() -> std::path::PathBuf {
        std::path::PathBuf::from(
            std::env::var("KTTSDB_DIR")
                .expect("KTTSDB_DIR must point to the dictionary data (kttsdb)"),
        )
        .join("InfoDic.wdic")
    }

    #[test]
    fn infodic_parse() {
        let p = wdic_path();
        let data = std::fs::read(&p).unwrap_or_else(|e| panic!("read failed {}: {e}", p.display()));
        assert_eq!(data.len(), 776);
        let w = parse(&data).expect("InfoDic.wdic parse");
        assert_eq!(w.len(), 35);
        assert_eq!(w.get_str("KOREAPCM").as_deref(), Some("URAW"));
        assert_eq!(w.get_str("KOREASEX").as_deref(), Some("Woman"));
        assert_eq!(w.get_str("LANG").as_deref(), Some("Korea"));
        assert_eq!(w.get_str("VOLUME").as_deref(), Some("150"));
        assert_eq!(w.get_str("SPEED").as_deref(), Some("100"));
        assert_eq!(w.get_str("PITCH").as_deref(), Some("150"));
        assert_eq!(w.get_str("Reading Mode").as_deref(), Some("Sentence"));
        assert_eq!(w.get_str("AUTO").as_deref(), Some("TRUE"));
        assert_eq!(w.get_str("Enter Read").as_deref(), Some("TRUE"));
        assert_eq!(
            w.get_str("TXT, WAV, MP3 File Save").as_deref(),
            Some("FALSE")
        );
        assert_eq!(w.entries[0].0, b"TXT, WAV, MP3 File Save".to_vec());
        assert_eq!(w.entries.last().unwrap().0, b"AUTO".to_vec());
        assert_eq!(w.get("no-such-key"), None);
    }

    #[test]
    fn infodic_synthetic() {
        let data = b"[A]=[1]\r\n\r\n  [B]=[2]  \n[C]=[3]\n";
        let w = parse(data).unwrap();
        assert_eq!(w.len(), 3);
        assert_eq!(w.get_str("A").as_deref(), Some("1"));
        assert_eq!(w.get_str("B").as_deref(), Some("2"));
        assert_eq!(w.get_str("C").as_deref(), Some("3"));
        assert!(parse(b"no-bracket-line\n").is_err());
    }
}
