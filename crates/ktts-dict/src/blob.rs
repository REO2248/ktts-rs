//! Length-prefixed blob format for embedding dictionary data into binaries.
//!
//! Layout (little-endian): `u32` entry count, then per entry
//! `u32` path length | path bytes (UTF-8) | `u64` data length | data bytes.
//! `encode` sorts entries by key, so the output is deterministic.

use crate::common::{DataMap, DictError, DictResult};

use std::path::Path;

/// Encodes a data map into the blob format, sorted by key.
///
/// # Panics
///
/// Panics if an entry count or length does not fit the fixed-size fields
/// (impossible for any realistic data map).
#[must_use]
pub fn encode(files: &DataMap) -> Vec<u8> {
    let mut entries: Vec<(&String, &Vec<u8>)> = files.iter().collect();
    entries.sort_unstable_by_key(|e| e.0);
    let mut out = Vec::new();
    out.extend_from_slice(
        &u32::try_from(entries.len())
            .expect("entry count fits u32")
            .to_le_bytes(),
    );
    for (path, data) in entries {
        out.extend_from_slice(
            &u32::try_from(path.len())
                .expect("path length fits u32")
                .to_le_bytes(),
        );
        out.extend_from_slice(path.as_bytes());
        out.extend_from_slice(
            &u64::try_from(data.len())
                .expect("data length fits u64")
                .to_le_bytes(),
        );
        out.extend_from_slice(data);
    }
    out
}

/// Recursively packs every file below `root` into a deterministic blob.
///
/// Keys are paths relative to `root` and always use `/` separators.
///
/// # Errors
///
/// Returns an error if a directory entry or file cannot be read.
pub fn encode_dir(root: &Path) -> DictResult<Vec<u8>> {
    let mut files = DataMap::new();
    collect_files(root, root, &mut files)?;
    Ok(encode(&files))
}

fn collect_files(root: &Path, dir: &Path, files: &mut DataMap) -> DictResult<()> {
    let entries = std::fs::read_dir(dir).map_err(|error| io_err("read directory", dir, &error))?;
    for entry in entries {
        let path = entry
            .map_err(|error| io_err("read directory entry", dir, &error))?
            .path();
        if path.is_dir() {
            collect_files(root, &path, files)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .expect("collected paths stay below the root")
                .to_string_lossy()
                .replace('\\', "/");
            let data = std::fs::read(&path).map_err(|error| io_err("read file", &path, &error))?;
            files.insert(relative, data);
        }
    }
    Ok(())
}

fn io_err(operation: &str, path: &Path, error: &std::io::Error) -> DictError {
    DictError::new(format!("{operation} {}: {error}", path.display()), 0)
}

/// Decodes a blob into a data map.
///
/// # Errors
///
/// Returns an error if the blob is truncated, has an invalid length, or
/// contains a non-UTF-8 key.
pub fn decode(blob: &[u8]) -> DictResult<DataMap> {
    let mut files = DataMap::new();
    let mut off = 0usize;
    let count = usize::try_from(read_u32(blob, &mut off)?)
        .map_err(|_| err("entry count does not fit usize", off))?;
    for _ in 0..count {
        let path_len = usize::try_from(read_u32(blob, &mut off)?)
            .map_err(|_| err("path length does not fit usize", off))?;
        let path = read_bytes(blob, &mut off, path_len)?;
        let path = std::str::from_utf8(path).map_err(|e| err(format!("invalid key: {e}"), off))?;
        let data_len = usize::try_from(read_u64(blob, &mut off)?)
            .map_err(|_| err("data length does not fit usize", off))?;
        let data = read_bytes(blob, &mut off, data_len)?;
        files.insert(path.to_string(), data.to_vec());
    }
    Ok(files)
}

fn err(msg: impl Into<String>, offset: usize) -> DictError {
    DictError::new(msg, offset)
}

fn read_u32(blob: &[u8], off: &mut usize) -> DictResult<u32> {
    let bytes: [u8; 4] = read_bytes(blob, off, 4)?
        .try_into()
        .expect("read_bytes returned exactly 4 bytes");
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64(blob: &[u8], off: &mut usize) -> DictResult<u64> {
    let bytes: [u8; 8] = read_bytes(blob, off, 8)?
        .try_into()
        .expect("read_bytes returned exactly 8 bytes");
    Ok(u64::from_le_bytes(bytes))
}

fn read_bytes<'a>(blob: &'a [u8], off: &mut usize, len: usize) -> DictResult<&'a [u8]> {
    let end = off
        .checked_add(len)
        .ok_or_else(|| err("blob offset overflow", *off))?;
    let slice = blob
        .get(*off..end)
        .ok_or_else(|| err("blob truncated", *off))?;
    *off = end;
    Ok(slice)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_map(entries: &[(&str, &[u8])]) -> DataMap {
        entries
            .iter()
            .map(|(k, v)| ((*k).to_string(), v.to_vec()))
            .collect()
    }

    #[test]
    fn roundtrip() {
        let files = build_map(&[
            ("InfoDic.wdic", b"KOREAPCM=0"),
            ("KLangDic/user.bin", b"\x01\x02\x03"),
            ("KSpeechDic/woman/synth.pcm", &[0u8; 4]),
        ]);
        let decoded = decode(&encode(&files)).expect("decode");
        assert_eq!(decoded, files);
    }

    #[test]
    fn encode_sorts_keys() {
        let files = build_map(&[("b", b"2"), ("a", b"1")]);
        let blob = encode(&files);
        assert_eq!(&blob[8..9], b"a", "first entry must be the smallest key");
    }

    #[test]
    fn empty_map_roundtrips() {
        let files = DataMap::new();
        let blob = encode(&files);
        assert_eq!(blob, 0u32.to_le_bytes());
        assert!(decode(&blob).expect("decode").is_empty());
    }

    #[test]
    fn truncated_blob_is_error() {
        let blob = encode(&build_map(&[("a", b"12345")]));
        assert!(decode(&blob[..blob.len() - 3]).is_err());
        assert!(decode(b"").is_err());
    }

    #[test]
    fn invalid_utf8_key_is_error() {
        let mut blob = encode(&build_map(&[("ok", b"data")]));
        // Corrupt the first path byte (offset: 4 count + 4 path length).
        blob[8] = 0xff;
        assert!(decode(&blob).is_err());
    }

    #[test]
    fn encode_dir_collects_nested_files_with_portable_keys() {
        let root = std::env::temp_dir().join(format!(
            "ktts-dict-encode-dir-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time after epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("KSpeechDic/woman")).expect("create test dirs");
        std::fs::write(root.join("InfoDic.wdic"), b"info").expect("write root file");
        std::fs::write(root.join("KSpeechDic/woman/synth.pcm"), b"pcm").expect("write nested file");

        let files = decode(&encode_dir(&root).expect("encode directory")).expect("decode blob");

        assert_eq!(files.get("InfoDic.wdic"), Some(&b"info".to_vec()));
        assert_eq!(
            files.get("KSpeechDic/woman/synth.pcm"),
            Some(&b"pcm".to_vec())
        );
        std::fs::remove_dir_all(root).expect("remove test directory");
    }
}
