//! Standalone mode for the `embed` feature.
//!
//! The kttsdb dictionary data is baked into the wasm binary at build time
//! (`wasm-pack build --features embed`) via `build.rs`, so the JS side does
//! not need to fetch and pass the dictionary files. The blob format is
//! defined in `ktts_dict::blob`.

use ktts_dict::common::DataMap;

/// The dictionary blob written by build.rs when the `embed` feature is on.
static KTT_SDB_BLOB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/kttsdb.blob"));

/// Parses the embedded blob into a data map.
///
/// # Errors
///
/// Returns an error if the embedded blob is malformed.
pub fn datamap() -> Result<DataMap, String> {
    ktts_dict::blob::decode(KTT_SDB_BLOB).map_err(|e| format!("embedded data: {e}"))
}
