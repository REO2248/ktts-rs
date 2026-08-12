pub const SAMPLE_RATE: u32 = 16000;
pub const CHANNELS: u16 = 1;
pub const BITS_PER_SAMPLE: u16 = 16;

/// Builds a WAV file from PCM samples.
///
/// # Panics
///
/// Panics if the sample count does not fit in `u32`.
#[must_use]
pub fn build_wav(samples: &[i16]) -> Vec<u8> {
    let data_len = u32::try_from(samples.len() * 2).expect("WAV data length fits u32");
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&CHANNELS.to_le_bytes());
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(
        &(SAMPLE_RATE * u32::from(CHANNELS) * (u32::from(BITS_PER_SAMPLE) / 8)).to_le_bytes(),
    );
    out.extend_from_slice(&(CHANNELS * BITS_PER_SAMPLE / 8).to_le_bytes());
    out.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for &s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WavInfo {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub data_len: u32,
    pub data_offset: usize,
}

#[must_use]
pub fn parse_wav_header(bytes: &[u8]) -> Option<WavInfo> {
    if bytes.len() < 44 {
        return None;
    }
    let riff = &bytes[0..4];
    let wave = &bytes[8..12];
    let fmt_magic = &bytes[12..16];
    let pcm = u16::from_le_bytes([bytes[20], bytes[21]]);
    let channels = u16::from_le_bytes([bytes[22], bytes[23]]);
    let sample_rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
    let bits = u16::from_le_bytes([bytes[34], bytes[35]]);
    let data_magic = &bytes[36..40];
    let data_len = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]);
    if riff != b"RIFF"
        || wave != b"WAVE"
        || fmt_magic != b"fmt "
        || data_magic != b"data"
        || pcm != 1
    {
        return None;
    }
    Some(WavInfo {
        sample_rate,
        channels,
        bits_per_sample: bits,
        data_len,
        data_offset: 44,
    })
}

#[must_use]
pub fn rms(samples: &[i16]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f64 = samples.iter().map(|&s| f64::from(s) * f64::from(s)).sum();
    #[expect(
        clippy::cast_precision_loss,
        reason = "sample count to f64 is exact for real audio"
    )]
    (sum / samples.len() as f64).sqrt()
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::float_cmp,
        reason = "oracle assertions use exact float equality"
    )]
    use super::*;

    #[test]
    fn header_layout_is_44_bytes_and_correct() {
        let wav = build_wav(&[]);
        assert_eq!(wav.len(), 44);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(u32::from_le_bytes([wav[4], wav[5], wav[6], wav[7]]), 36);
        assert_eq!(u16::from_le_bytes([wav[20], wav[21]]), 1);
        assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 1);
        assert_eq!(
            u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]),
            16000
        );
        assert_eq!(
            u32::from_le_bytes([wav[28], wav[29], wav[30], wav[31]]),
            32000
        );
        assert_eq!(u16::from_le_bytes([wav[32], wav[33]]), 2);
        assert_eq!(u16::from_le_bytes([wav[34], wav[35]]), 16);
    }

    #[test]
    fn samples_are_little_endian_pcm16() {
        let samples = [1i16, -2, 32767, -32768];
        let wav = build_wav(&samples);
        assert_eq!(wav.len(), 44 + 8);
        assert_eq!(&wav[44..46], &1i16.to_le_bytes());
        assert_eq!(&wav[46..48], &(-2i16).to_le_bytes());
        assert_eq!(&wav[50..52], &(-32768i16).to_le_bytes());
        let info = parse_wav_header(&wav).expect("valid header");
        assert_eq!(info.data_len, 8);
        assert_eq!(info.sample_rate, 16000);
        assert_eq!(info.channels, 1);
        assert_eq!(info.bits_per_sample, 16);
    }

    #[test]
    fn parse_rejects_garbage() {
        assert!(parse_wav_header(b"not a wav at all........").is_none());
        assert!(parse_wav_header(&[0u8; 44]).is_none());
    }

    #[test]
    fn rms_zero_for_silence_positive_for_tone() {
        assert_eq!(rms(&[]), 0.0);
        assert_eq!(rms(&[0, 0, 0]), 0.0);
        let tone = [1000i16, -1000, 1000, -1000];
        assert!(rms(&tone) > 0.0);
    }
}
