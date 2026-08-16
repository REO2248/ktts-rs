//! Standalone mode: the kttsdb dictionary data is baked into the binary at
//! build time (`cargo build --features embed`) via `build.rs`, so `ktts` runs
//! without any data directory on disk.
//!
//! The blob format is defined in `ktts_dict::blob`.

use crate::pipeline;
use crate::types::{PipelineError, VoiceParams};
use ktts_dict::common::DataMap;

/// The dictionary blob written by build.rs when the `embed` feature is on.
static KTT_SDB_BLOB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/kttsdb.blob"));

/// Synthesizes speech using the dictionaries embedded into this binary.
///
/// # Errors
///
/// Returns an error if the embedded blob is malformed or an engine stage fails.
pub fn synthesize(
    text: &str,
    voice: &str,
    params: &VoiceParams,
) -> Result<Vec<i16>, PipelineError> {
    let files = datamap()?;
    pipeline::run_pipeline_files(text, files, voice, params)
}

/// Parses the embedded blob into a data map.
fn datamap() -> Result<DataMap, PipelineError> {
    ktts_dict::blob::decode(KTT_SDB_BLOB).map_err(|e| PipelineError::Engine("embed", e.to_string()))
}
