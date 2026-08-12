use crate::consts::{CLAMP_HIGH_PITCH_SPEED, CLAMP_HIGH_VOICE, HALF, PITCH_BASE};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SynthParams {
    pub pitch: f32,
    pub speed: f32,
    pub voice: f32,
    pub volume: f32,
}

impl SynthParams {
    #[must_use]
    pub const fn defaults() -> Self {
        Self {
            pitch: 1.0,
            speed: 1.0,
            voice: 1.0,
            volume: 1.5,
        }
    }

    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "C port: index/math casts with wrap semantics"
    )]
    pub fn from_ini(m_pitch: i32, m_speed: i32, m_voice: i32, m_volume: i32) -> Self {
        let mut p = Self {
            pitch: m_pitch as f32 / PITCH_BASE,
            speed: 1.0 / (m_speed as f32 / 100.0),
            voice: m_voice as f32 / 100.0,
            volume: m_volume as f32 / 100.0,
        };
        if p.pitch >= CLAMP_HIGH_PITCH_SPEED || p.pitch <= HALF {
            p.pitch = if p.pitch >= CLAMP_HIGH_PITCH_SPEED {
                CLAMP_HIGH_PITCH_SPEED
            } else {
                HALF
            };
        }
        if p.speed >= CLAMP_HIGH_PITCH_SPEED || p.speed <= HALF {
            p.speed = if p.speed >= CLAMP_HIGH_PITCH_SPEED {
                CLAMP_HIGH_PITCH_SPEED
            } else {
                HALF
            };
        }
        if p.voice >= CLAMP_HIGH_VOICE || p.voice <= HALF {
            p.voice = if p.voice >= CLAMP_HIGH_VOICE {
                CLAMP_HIGH_VOICE
            } else {
                HALF
            };
        }
        p
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::float_cmp,
        reason = "test fixtures: oracle values converted with intentional casts"
    )]
    use super::*;

    #[test]
    fn defaults_match_measured() {
        let p = SynthParams::from_ini(150, 100, 100, 150);
        assert_eq!(p.pitch, 1.0);
        assert_eq!(p.speed, 1.0);
        assert_eq!(p.voice, 1.0);
        assert_eq!(p.volume, 1.5);
        assert_eq!(SynthParams::defaults().pitch, 1.0);
        assert_eq!(SynthParams::defaults().speed, 1.0);
    }

    #[test]
    fn clamps() {
        let p = SynthParams::from_ini(400, 300, 300, 100);
        assert_eq!(p.pitch, 1.7);
        assert_eq!(p.speed, 0.5);
        assert_eq!(p.voice, 1.5);
        assert_eq!(p.volume, 1.0);
        let p = SynthParams::from_ini(50, 50, 40, 100);
        assert_eq!(p.pitch, 0.5);
        assert_eq!(p.speed, 1.7);
        assert_eq!(p.voice, 0.5);
    }
}
