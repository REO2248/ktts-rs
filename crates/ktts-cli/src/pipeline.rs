use std::path::Path;

use ktts_engine::{DataMap, Engine};

use crate::types::{PipelineError, VoiceParams};

pub const VOICE: &str = "woman";

/// Builds a data map from a kttsdb directory.
///
/// # Errors
///
/// Returns an error if the directory cannot be read.
pub fn load_datamap(data_dir: &Path) -> Result<DataMap, PipelineError> {
    let mut files = DataMap::new();
    let mut stack = vec![data_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|e| {
            PipelineError::Engine("ktts-cli", format!("read_dir {}: {e}", dir.display()))
        })?;
        for entry in entries {
            let entry = entry
                .map_err(|e| PipelineError::Engine("ktts-cli", format!("read_dir entry: {e}")))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(data) = std::fs::read(&path) {
                let rel = path.strip_prefix(data_dir).map_err(|e| {
                    PipelineError::Engine("ktts-cli", format!("relative dictionary path: {e}"))
                })?;
                files.insert(rel.to_string_lossy().replace('\\', "/"), data);
            }
        }
    }
    Ok(files)
}

/// Loads the dictionaries and synthesizes text.
///
/// # Errors
///
/// Returns an error if loading or synthesis fails.
pub fn run_pipeline(
    text: &str,
    data_dir: &Path,
    params: &VoiceParams,
) -> Result<Vec<i16>, PipelineError> {
    run_pipeline_files(text, load_datamap(data_dir)?, params)
}

/// Synthesizes text from a pre-loaded dictionary map.
///
/// # Errors
///
/// Returns an error if loading or synthesis fails.
pub fn run_pipeline_files(
    text: &str,
    files: DataMap,
    params: &VoiceParams,
) -> Result<Vec<i16>, PipelineError> {
    Engine::load(files, VOICE)?.synthesize(text, params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_data_directory_reports_cli_adapter() {
        let err = run_pipeline(
            "안녕하세요",
            Path::new("/nonexistent"),
            &VoiceParams::default(),
        )
        .unwrap_err();
        assert!(matches!(err, PipelineError::Engine("ktts-cli", _)));
    }
}
