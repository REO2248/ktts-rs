//! Embeds the kttsdb dictionary data into the wasm binary when the `embed`
//! feature is enabled (`wasm-pack build --features embed`).
//!
//! The data is packed into a length-prefixed blob written to
//! `$OUT_DIR/kttsdb.blob` and pulled in with `include_bytes!` by
//! `src/embedded.rs`. Without the feature this script does nothing, so
//! regular builds are unaffected.

use std::env;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use ktts_dict::blob;
use ktts_dict::common::DataMap;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_FEATURE_EMBED").is_err() {
        return;
    }

    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"))
        .join("../../kttsdb");
    assert!(
        root.is_dir(),
        "embed feature requires the dictionary data at {} (relative to the crate root); \
         copy kttsdb/ into the workspace root first",
        root.display()
    );
    println!("cargo:rerun-if-changed={}", root.display());

    let mut files: DataMap = DataMap::new();
    collect_files(&root, &root, &mut files);

    let blob = blob::encode(&files);
    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("kttsdb.blob");
    fs::File::create(&out)
        .expect("create kttsdb.blob")
        .write_all(&blob)
        .expect("write kttsdb.blob");
    eprintln!(
        "ktts-wasm(embed): packed {} files ({} bytes) from {}",
        files.len(),
        blob.len(),
        root.display()
    );
}

fn collect_files(root: &Path, dir: &Path, out: &mut DataMap) {
    for entry in fs::read_dir(dir).expect("read_dir") {
        let path = entry.expect("read_dir entry").path();
        if path.is_dir() {
            collect_files(root, &path, out);
        } else {
            let rel = path
                .strip_prefix(root)
                .expect("paths are walked under the data root")
                .to_string_lossy()
                .replace('\\', "/");
            out.insert(rel, fs::read(&path).expect("read dictionary file"));
        }
    }
}
