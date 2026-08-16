#![expect(
    unused_crate_dependencies,
    reason = "deps used by other targets of this package"
)]
use std::io::{IsTerminal, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;

use ktts_cli::pipeline;
use ktts_cli::types::{PipelineError, VoiceParams};
use ktts_cli::wav;

#[derive(Debug, Parser)]
#[command(
    name = "ktts",
    version,
    about = "청봉 4.0 TTS text-to-WAV synthesis CLI"
)]
struct Cli {
    /// dictionary data directory (kttsdb; fallback: $KTTSDB_DIR, then /usr/share/apps/kttsdb)
    #[expect(
        clippy::doc_markdown,
        reason = "env var name rendered literally in --help"
    )]
    #[arg(short, long)]
    data_dir: Option<PathBuf>,

    /// output WAV path (default: write WAV bytes to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// synthesis Voice identifier (default: woman)
    #[arg(long, default_value = ktts_engine::DEFAULT_VOICE)]
    voice: String,

    /// speed multiplier (default: 1.0)
    #[arg(short, long, default_value_t = 1.0)]
    speed: f32,

    /// pitch offset, 0.0 = normal, 1.0 = double (default: 0.0)
    #[arg(short, long, default_value_t = 0.0)]
    pitch: f32,

    /// volume multiplier (default: 1.0)
    #[arg(short, long, default_value_t = 1.0)]
    volume: f32,

    /// input text (multiple arguments are joined with spaces; if absent, read from stdin)
    text: Vec<String>,
}

#[cfg(not(feature = "embed"))]
const DEFAULT_DATA_DIR: &str = "/usr/share/apps/kttsdb";

/// Resolves the dictionary data directory: `--data-dir`, then `$KTTSDB_DIR`,
/// then the default install path.
#[cfg(not(feature = "embed"))]
fn resolve_data_dir(cli_dir: Option<&Path>, env_dir: Option<&str>) -> PathBuf {
    cli_dir
        .map(Path::to_path_buf)
        .or_else(|| env_dir.filter(|s| !s.is_empty()).map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DATA_DIR))
}

/// Resolves the dictionary data directory: `--data-dir`, then `$KTTSDB_DIR`;
/// `None` means the dictionaries embedded into this binary are used.
#[cfg(feature = "embed")]
fn resolve_data_dir(cli_dir: Option<&Path>, env_dir: Option<&str>) -> Option<PathBuf> {
    cli_dir
        .map(Path::to_path_buf)
        .or_else(|| env_dir.filter(|s| !s.is_empty()).map(PathBuf::from))
}

/// Reads the input text: positional args are joined with spaces; if none are
/// given, reads all of stdin (fails when stdin is a terminal or empty).
fn read_input(args: &[String]) -> Result<String, String> {
    if !args.is_empty() {
        return Ok(args.join(" "));
    }
    if std::io::stdin().is_terminal() {
        return Err("no input text (pass TEXT args or pipe text via stdin)".to_string());
    }
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| format!("stdin read failed: {e}"))?;
    if buf.trim().is_empty() {
        return Err("no input text (stdin is empty)".to_string());
    }
    Ok(buf)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let params = VoiceParams {
        speed: cli.speed,
        pitch: cli.pitch,
        volume: cli.volume,
    };
    #[cfg(not(feature = "embed"))]
    let data_dir = Some(resolve_data_dir(
        cli.data_dir.as_deref(),
        std::env::var("KTTSDB_DIR").ok().as_deref(),
    ));
    #[cfg(feature = "embed")]
    let data_dir = resolve_data_dir(
        cli.data_dir.as_deref(),
        std::env::var("KTTSDB_DIR").ok().as_deref(),
    );
    let text = match read_input(&cli.text) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    match run(
        &text,
        data_dir.as_deref(),
        cli.output.as_deref(),
        &cli.voice,
        &params,
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

fn run(
    text: &str,
    data_dir: Option<&Path>,
    output: Option<&Path>,
    voice: &str,
    params: &VoiceParams,
) -> Result<(), PipelineError> {
    if params.speed <= 0.0 {
        return Err(PipelineError::BadParam(
            "--speed must be greater than 0".to_string(),
        ));
    }
    if params.volume < 0.0 {
        return Err(PipelineError::BadParam(
            "--volume must be non-negative".to_string(),
        ));
    }

    let samples = synthesize_samples(text, data_dir, voice, params)?;
    let wav_bytes = wav::build_wav(&samples);
    if let Some(path) = output {
        std::fs::write(path, &wav_bytes).map_err(|e| {
            PipelineError::Engine("wav-write", format!("{}: {}", path.display(), e))
        })?;
    } else {
        let mut stdout = std::io::stdout().lock();
        stdout
            .write_all(&wav_bytes)
            .and_then(|()| stdout.flush())
            .map_err(|e| PipelineError::Engine("stdout-write", format!("{e}")))?;
    }
    Ok(())
}

/// Runs the pipeline against a data directory, or the dictionaries embedded
/// into this binary when `data_dir` is `None` (embed feature only).
fn synthesize_samples(
    text: &str,
    data_dir: Option<&Path>,
    voice: &str,
    params: &VoiceParams,
) -> Result<Vec<i16>, PipelineError> {
    if let Some(dir) = data_dir {
        let speech_dir = dir.join("KSpeechDic").join(voice);
        if !speech_dir.is_dir() {
            return Err(PipelineError::BadParam(format!(
                "voice data directory not found: {} (set --data-dir or $KTTSDB_DIR)",
                speech_dir.display()
            )));
        }
        pipeline::run_pipeline(text, dir, voice, params)
    } else {
        #[cfg(feature = "embed")]
        {
            ktts_cli::embedded::synthesize(text, voice, params)
        }
        #[cfg(not(feature = "embed"))]
        {
            unreachable!("resolve_data_dir always yields a directory without the embed feature")
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::float_cmp, reason = "parameter defaults are exact")]
    use super::*;

    #[test]
    fn parses_minimal_args() {
        let cli = Cli::try_parse_from(["ktts", "-d", "/d", "안녕하세요"]).unwrap();
        assert_eq!(cli.text, ["안녕하세요"]);
        assert_eq!(cli.output, None);
        assert_eq!(cli.data_dir, Some(PathBuf::from("/d")));
    }

    #[test]
    fn data_dir_is_optional_at_parse_time() {
        let cli = Cli::try_parse_from(["ktts", "안녕하세요"]).unwrap();
        assert_eq!(cli.data_dir, None);
    }

    #[cfg(not(feature = "embed"))]
    #[test]
    fn data_dir_resolution_prefers_cli_over_env_over_default() {
        assert_eq!(
            resolve_data_dir(Some(Path::new("/cli")), Some("/env")),
            PathBuf::from("/cli")
        );
        assert_eq!(resolve_data_dir(None, Some("/env")), PathBuf::from("/env"));
        assert_eq!(
            resolve_data_dir(None, Some("")),
            PathBuf::from(DEFAULT_DATA_DIR)
        );
        assert_eq!(
            resolve_data_dir(None, None),
            PathBuf::from(DEFAULT_DATA_DIR)
        );
    }

    #[cfg(feature = "embed")]
    #[test]
    fn data_dir_resolution_prefers_cli_over_env_over_embedded() {
        assert_eq!(
            resolve_data_dir(Some(Path::new("/cli")), Some("/env")),
            Some(PathBuf::from("/cli"))
        );
        assert_eq!(
            resolve_data_dir(None, Some("/env")),
            Some(PathBuf::from("/env"))
        );
        assert_eq!(resolve_data_dir(None, Some("")), None);
        assert_eq!(resolve_data_dir(None, None), None);
    }

    #[test]
    fn joins_multiple_text_args() {
        let cli = Cli::try_parse_from(["ktts", "-d", "/d", "반갑습니다", "감사합니다"]).unwrap();
        assert_eq!(cli.text.join(" "), "반갑습니다 감사합니다");
    }

    #[test]
    fn text_is_optional_at_parse_time() {
        let cli = Cli::try_parse_from(["ktts", "-d", "/d"]).unwrap();
        assert!(cli.text.is_empty());
    }

    #[test]
    fn read_input_joins_args_when_present() {
        assert_eq!(
            read_input(&["반갑습니다".to_string(), "감사합니다".to_string()]),
            Ok("반갑습니다 감사합니다".to_string())
        );
    }

    #[test]
    fn voice_params_default_when_absent() {
        let cli = Cli::try_parse_from(["ktts", "-d", "/d", "안녕하세요"]).unwrap();
        assert_eq!(cli.voice, "woman");
        assert_eq!(cli.speed, 1.0);
        assert_eq!(cli.pitch, 0.0);
        assert_eq!(cli.volume, 1.0);
    }

    #[test]
    fn parses_voice_params() {
        let cli = Cli::try_parse_from([
            "ktts",
            "-d",
            "/d",
            "--voice",
            "future-voice",
            "--speed",
            "1.5",
            "--pitch",
            "0.25",
            "--volume",
            "0.8",
            "hi",
        ])
        .unwrap();
        assert_eq!(cli.voice, "future-voice");
        assert_eq!(cli.speed, 1.5);
        assert_eq!(cli.pitch, 0.25);
        assert_eq!(cli.volume, 0.8);
    }

    #[test]
    fn speed_zero_is_rejected() {
        let params = VoiceParams {
            speed: 0.0,
            ..VoiceParams::default()
        };
        assert!(matches!(
            run(
                "hi",
                Some(Path::new("/d")),
                None,
                ktts_engine::DEFAULT_VOICE,
                &params,
            ),
            Err(PipelineError::BadParam(_))
        ));
    }

    #[test]
    fn negative_volume_is_rejected() {
        let params = VoiceParams {
            volume: -0.5,
            ..VoiceParams::default()
        };
        assert!(matches!(
            run(
                "hi",
                Some(Path::new("/d")),
                None,
                ktts_engine::DEFAULT_VOICE,
                &params,
            ),
            Err(PipelineError::BadParam(_))
        ));
    }

    #[test]
    fn unknown_option_is_error() {
        assert!(Cli::try_parse_from(["ktts", "-d", "/d", "--nope", "hi"]).is_err());
    }

    #[test]
    fn supports_equals_and_long_forms() {
        let cli = Cli::try_parse_from(["ktts", "--data-dir=/d", "--output=x.wav", "hi"]).unwrap();
        assert_eq!(cli.data_dir, Some(PathBuf::from("/d")));
        assert_eq!(cli.output, Some(PathBuf::from("x.wav")));
    }
}
