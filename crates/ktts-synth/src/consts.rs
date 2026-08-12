pub const CTX_INIT: f32 = 3.0;
pub const PHONE_MISMATCH: f32 = 2.0;
pub const PHONE_EDGE_MISMATCH: f32 = 1.0;
pub const PENALTY_10: f32 = 10.0;
pub const PENALTY_5: f32 = 5.0;
pub const LEN_LOW_RATIO: f32 = 0.4;
pub const LEN_SCALE: f32 = 4.0;
pub const LEN_HIGH_RATIO: f64 = 2.1;
pub const HALF: f32 = 0.5;
pub const SCORE_LIMIT: f32 = 1000.0;
pub const PITCH_W_END: f32 = 0.7;
pub const PITCH_W_START: f32 = 0.3;
pub const ENG_SAT: f32 = 0.0035;
pub const ENG_W_LOW: f32 = 50.0;
pub const PITCH_W_LOW: f32 = 20.0;
pub const ENG_W_HIGH: f32 = 300.0;
pub const PITCH_W_HIGH: f32 = 100.0;
pub const CEP_DIV: f32 = 30.0;
pub const TWO_PI: f64 = 6.283_185_482_025_146;
pub const CLAMP_HIGH_PITCH_SPEED: f32 = 1.7;
pub const CLAMP_HIGH_VOICE: f32 = 1.5;
pub const PITCH_BASE: f32 = 150.0;
pub const SAMPLE_RATE: f32 = 16000.0;

pub const PANALTY: [f32; 8] = [10.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];

pub const MIN_PITCH: i16 = 30;

pub const PSOLA_CLIP: i32 = 0x6c76;
pub const VOLUME_CLIP: i32 = 32000;
pub const SAMPLE_CHUNK: usize = 160_000;
pub const CAND_MAX: usize = 10;
pub const TOKEN_MAX: usize = 30;
pub const GROUP_TYPE_LIMIT: usize = 30_000;
pub const REST_WORD: u16 = 1000;
pub const REST_WORD_STRONG: u16 = 1600;
pub const REST_SENT_END: u16 = 11000;
pub const REST_SPACE: u16 = 4200;
