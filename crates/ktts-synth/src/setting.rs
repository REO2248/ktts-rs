use crate::consts::{CLAMP_HIGH_PITCH_SPEED, CLAMP_HIGH_VOICE, HALF, PITCH_BASE};

/// Engine-unit voice parameters (`m_pitch`/`m_speed`/`m_voice`/`m_volume` in
/// the C engine) as loaded from `InfoDic.wdic`.
///
/// `SynthContext` keeps these as the source of truth: the single-field
/// setters change one field and leave the others intact, so setting several
/// parameters composes instead of resetting the others to their defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IniParams {
    pub pitch: i32,
    pub speed: i32,
    pub voice: i32,
    pub volume: i32,
}

impl IniParams {
    #[must_use]
    pub const fn defaults() -> Self {
        Self {
            pitch: 150,
            speed: 100,
            voice: 100,
            volume: 150,
        }
    }

    /// Maps the frontend synthesis params (speed multiplier, pitch offset,
    /// volume multiplier; API defaults 1.0 / 0.0 / 1.0) to engine units,
    /// keeping the `base` value for params left at their default.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "C port: param scaling to engine units"
    )]
    pub const fn from_api(base: Self, speed: f32, pitch: f32, volume: f32) -> Self {
        Self {
            pitch: if pitch.abs() > 1e-6 {
                (PITCH_BASE * (1.0 + pitch)) as i32
            } else {
                base.pitch
            },
            speed: if (speed - 1.0).abs() > 1e-6 {
                (100.0 * speed) as i32
            } else {
                base.speed
            },
            voice: base.voice,
            volume: if (volume - 1.0).abs() > 1e-6 {
                (150.0 * volume) as i32
            } else {
                base.volume
            },
        }
    }

    #[must_use]
    pub fn params(self) -> SynthParams {
        SynthParams::from_ini(self.pitch, self.speed, self.voice, self.volume)
    }
}

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
    fn ini_params_defaults_match_synth_params_defaults() {
        assert_eq!(IniParams::defaults().params(), SynthParams::defaults());
    }

    #[test]
    fn from_api_keeps_base_for_default_params() {
        let base = IniParams::defaults();
        assert_eq!(IniParams::from_api(base, 1.0, 0.0, 1.0), base);
        let custom = IniParams {
            pitch: 160,
            speed: 90,
            voice: 110,
            volume: 130,
        };
        assert_eq!(IniParams::from_api(custom, 1.0, 0.0, 1.0), custom);
    }

    #[test]
    fn from_api_scales_all_params_together() {
        let p = IniParams::from_api(IniParams::defaults(), 1.5, 0.5, 1.5);
        assert_eq!(p.pitch, 225);
        assert_eq!(p.speed, 150);
        assert_eq!(p.voice, 100);
        assert_eq!(p.volume, 225);
        let p = IniParams::from_api(IniParams::defaults(), 0.5, -0.2, 0.5);
        assert_eq!(p.pitch, 120);
        assert_eq!(p.speed, 50);
        assert_eq!(p.volume, 75);
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
