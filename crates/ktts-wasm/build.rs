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
use std::path::PathBuf;

use ktts_dict::blob;

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

    let blob = blob::encode_dir(&root).expect("pack dictionary directory");
    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("kttsdb.blob");
    fs::File::create(&out)
        .expect("create kttsdb.blob")
        .write_all(&blob)
        .expect("write kttsdb.blob");
    eprintln!(
        "ktts-wasm(embed): packed {} bytes from {}",
        blob.len(),
        root.display()
    );
}
