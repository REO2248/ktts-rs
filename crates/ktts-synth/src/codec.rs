pub const CODEC_RAW: u8 = 0;
pub const CODEC_URAW: u8 = 1;
pub const CODEC_G721: u8 = 2;
pub const CODEC_G723_24: u8 = 3;
pub const CODEC_G723_40: u8 = 4;
pub const CODEC_G729: u8 = 5;

#[must_use]
pub fn codec_from_name(name: &str) -> Option<u8> {
    match name.trim() {
        "" => Some(CODEC_RAW),
        "URAW" => Some(CODEC_URAW),
        "G721" => Some(CODEC_G721),
        "G723_24" => Some(CODEC_G723_24),
        "G723_40" => Some(CODEC_G723_40),
        "G729" => Some(CODEC_G729),
        _ => None,
    }
}

#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
pub fn get_codec_byte_number(w_pcm_size: u16, n_bps: i32) -> i16 {
    let w = i32::from(w_pcm_size);
    let rem = i32::from(w != (w / 0xa0) * 0xa0);
    let bps_off = i32::from(n_bps != 8000);
    let n50 = n_bps / 0x32;
    let f = (((n50 >> 0x1f) as u32 >> 0x1d) as i32 + n50) >> 3;
    ((rem + w / 0x140 + 2 + bps_off) * f) as i16
}

const SEG_END: [i32; 8] = [255, 511, 1023, 2047, 4095, 8191, 16383, 32767];

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
const fn linear2alaw(pcm_val: i32) -> u8 {
    let mut bvar3 = 0xd5u8;
    let mut pcm_val = pcm_val;
    if pcm_val < 0 {
        bvar3 = 0x55;
        pcm_val = -8 - pcm_val;
    }
    let mut ivar2 = 0i32;
    while ivar2 != 8 {
        if pcm_val <= SEG_END[ivar2 as usize] {
            let bvar1 = if ivar2 < 2 {
                (pcm_val >> 4) as u8
            } else {
                (pcm_val >> ((ivar2 + 3) & 0x1f)) as u8
            };
            return bvar3 ^ (bvar1 & 0xf | ((ivar2 << 4) as u8));
        }
        ivar2 += 1;
    }
    bvar3 ^ 0x7f
}

#[expect(
    clippy::cast_possible_wrap,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
fn alaw2linear(a_val: u8) -> i32 {
    let bvar3 = a_val ^ 0x55;
    let uvar2 = (bvar3 & 0x70) >> 4;
    let mut ivar1 = i32::from(bvar3 & 0xf) * 0x10;
    if uvar2 == 0 {
        ivar1 += 8;
    } else if uvar2 == 1 {
        ivar1 += 0x108;
    } else {
        ivar1 = (ivar1 + 0x108) << (uvar2 - 1);
    }
    if (bvar3 as i8) >= 0 {
        ivar1 = -ivar1;
    }
    ivar1
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
const fn linear2ulaw(pcm_val: i32) -> u8 {
    let mut bvar2 = 0xffu8;
    let mut pcm_val = pcm_val;
    if pcm_val < 0 {
        bvar2 = 0x7f;
        pcm_val = -pcm_val;
    }
    let mut ivar1 = 0i32;
    while ivar1 != 8 {
        if pcm_val + 0x84 <= SEG_END[ivar1 as usize] {
            let b = ((pcm_val + 0x84) >> ((ivar1 + 3) & 0x1f)) as u8;
            return (b & 0xf | ((ivar1 << 4) as u8)) ^ bvar2;
        }
        ivar1 += 1;
    }
    bvar2 ^ 0x7f
}

#[expect(
    clippy::cast_possible_wrap,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
fn ulaw2linear(u_val: u8) -> i32 {
    let ivar1 = (i32::from(!u_val & 0xf) * 8 + 0x84) << i32::from((!u_val & 0x70) >> 4);
    if (u_val as i8) < 0 {
        ivar1 - 0x84
    } else {
        0x84 - ivar1
    }
}

const POWER2: [i16; 16] = [
    1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 0,
];

#[derive(Debug, Clone, Copy)]
pub struct G72xState {
    pub yl: i32,
    pub yu: i16,
    pub dms: i16,
    pub dml: i16,
    pub ap: i16,
    pub a: [i16; 2],
    pub b: [i16; 6],
    pub pk: [i16; 2],
    pub dq: [i16; 6],
    pub sr: [i16; 2],
    pub td: u8,
}

#[must_use]
pub const fn g72x_init_state() -> G72xState {
    G72xState {
        yl: 0x8800,
        yu: 0x220,
        dms: 0,
        dml: 0,
        ap: 0,
        a: [0, 0],
        pk: [0, 0],
        sr: [0x20, 0x20],
        b: [0; 6],
        dq: [0x20; 6],
        td: 0,
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
fn fmult(an: i32, srn: i32) -> i32 {
    let uvar3: u32 = if an < 1 {
        (-(an as i16) as u32) & 0x1fff
    } else {
        an as u32
    };
    let mut ivar6 = 0i32;
    let mut svar2: i16;
    loop {
        svar2 = uvar3 as i16;
        if svar2 < POWER2[ivar6 as usize] {
            break;
        }
        ivar6 += 1;
        if ivar6 == 0xf {
            break;
        }
    }
    let mut ivar5 = 0x20i32;
    if svar2 != 0 {
        let bvar1 = ivar6 + -6;
        if (ivar6 + -6) < 0 {
            ivar5 = i32::from((i32::from(svar2) << ((-bvar1) & 0x1f)) as i16);
        } else {
            ivar5 = i32::from(svar2) >> (bvar1 & 0x1f);
        }
    }
    ivar6 = ivar6 + -0x13 + ((srn >> 6) & 0xf);
    svar2 = (((ivar5 * (srn & 0x3f) + 0x30) >> 4) & 0xffff) as i16;
    let uvar4: i16 = if ivar6 < 0 {
        (i32::from(svar2) >> ((-ivar6) & 0x1f)) as u16
    } else {
        ((i32::from(svar2) << (ivar6 & 0x1f)) & 0x7fff) as u16
    } as i16;
    if (srn ^ an) >= 0 {
        i32::from(uvar4)
    } else {
        -i32::from(uvar4)
    }
}

fn predictor_zero(state: &G72xState) -> i32 {
    let mut sezi = fmult(i32::from(state.b[0] >> 2), i32::from(state.dq[0]));
    for i in 1..6 {
        sezi += fmult(i32::from(state.b[i] >> 2), i32::from(state.dq[i]));
    }
    sezi
}

fn predictor_pole(state: &G72xState) -> i32 {
    let ivar1 = fmult(i32::from(state.a[1] >> 2), i32::from(state.sr[1]));
    let ivar2 = fmult(i32::from(state.a[0] >> 2), i32::from(state.sr[0]));
    ivar2 + ivar1
}

fn step_size(state: &G72xState) -> i32 {
    if 0xff < state.ap {
        return i32::from(state.yu);
    }
    let ivar2 = i32::from(state.ap >> 2);
    let mut ivar1 = state.yl >> 6;
    let ivar3 = i32::from(state.yu) - ivar1;
    if ivar3 < 1 {
        if ivar3 != 0 {
            return ivar1 + ((ivar3 * ivar2 + 0x3f) >> 6);
        }
    } else {
        ivar1 += (ivar3 * ivar2) >> 6;
    }
    ivar1
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
fn reconstruct(sign: i32, dqln: i32, y: i32) -> i32 {
    let uvar2 = (dqln as i16).wrapping_add((y >> 2) as i16);
    if uvar2 >= 0 {
        let dql = i32::from(uvar2);
        let mut ivar1 = ((dql & 0x7f) + 0x80) << 7;
        let dex = (dql >> 7) & 0xf;
        ivar1 = i32::from(ivar1 as i16) >> ((0xe - dex) & 0x1f);
        if sign != 0 {
            ivar1 += -0x8000;
        }
        return ivar1;
    }
    if sign == 0 { 0 } else { -0x8000 }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
fn quantize(d: i32, y: i32, table: &[i16], size: i32) -> i32 {
    let uvar1 = (d >> 0x1f) as u16;
    let mut svar2 = (i32::from((d as u16) ^ uvar1) - i32::from(uvar1)) as i16;
    let mut ivar3 = 0i32;
    loop {
        if (svar2 >> 1) < POWER2[ivar3 as usize] {
            break;
        }
        ivar3 += 1;
        if ivar3 == 0xf {
            break;
        }
    }
    if 0 < size {
        let exp = ivar3;
        svar2 = (((ivar3 << 7) as i16).wrapping_sub((y >> 2) as i16))
            .wrapping_add((((i32::from(svar2) << 7) >> (exp & 0x1f)) as u16 & 0x7f) as i16);
        ivar3 = 0;
        if table[0] <= svar2 {
            loop {
                ivar3 += 1;
                if size <= ivar3 {
                    break;
                }
                if table[ivar3 as usize] > svar2 {
                    break;
                }
            }
        }
    } else {
        ivar3 = 0;
    }
    if d < 0 {
        return (size * 2 + 1) - ivar3;
    }
    if ivar3 == 0 {
        ivar3 = size * 2 + 1;
    }
    ivar3
}

#[allow(clippy::if_same_then_else)]
#[allow(clippy::too_many_arguments)]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
#[expect(
    clippy::branches_sharing_code,
    reason = "C port: shared code in branches kept as-is (extraction would change control flow)"
)]
fn update(
    code_size: i32,
    y: i32,
    wi: i32,
    fi: i32,
    dq: i32,
    sr: i32,
    dqsez: i32,
    state: &mut G72xState,
) {
    let mut svar8: i32 = 0x5d00;
    let uvar4 = state.yl;
    let uvar5 = ((dqsez as u32) >> 0x1f) as u16;
    let uvar6 = (dq as u16) & 0x7fff;
    if ((uvar4 >> 0xf) as i16) < 10 {
        svar8 = i32::from(((((uvar4 >> 10) & 0x1f) + 0x20) << ((uvar4 >> 0xf) & 0x1f)) as i16);
        svar8 = i32::from((((svar8 >> 1) + svar8) >> 1) as i16);
    }
    let bvar10 = state.td != 0;
    let tr = bvar10 && svar8 < i32::from(uvar6);
    let svar1 = (((wi - y) >> 5) as i16).wrapping_add(y as i16);
    state.yu = svar1;
    let ivar9: i32 = if svar1 < 0x220 {
        state.yu = 0x220;
        0x220
    } else if 0x1400 < svar1 {
        state.yu = 0x1400;
        0x1400
    } else {
        i32::from(svar1)
    };
    state.yl = ((-uvar4) >> 6) + uvar4 + ivar9;

    let local_2a = state.pk[0];
    let mut local_2c: i16 = 0;
    if tr {
        state.a = [0, 0];
        state.b = [0; 6];
    } else {
        local_2c = state.a[1].wrapping_sub(state.a[1] >> 7);
        let mut svar2: i32;
        let local_14: i32;
        let mut svar1: i32;
        if dqsez == 0 {
            svar1 = i32::from(state.a[0]);
            state.a[1] = local_2c;
            svar1 = svar1 - (svar1 >> 8);
            svar2 = 0x3c00 - i32::from(local_2c);
            state.a[0] = svar1 as i16;
            local_14 = -svar2;
            if local_14 > svar1 {
                svar2 = -svar2;
                state.a[0] = svar2 as i16;
            } else if svar2 < svar1 {
                state.a[0] = svar2 as i16;
            }
        } else {
            let same_sign = (uvar5 ^ local_2a as u16) == 0;
            let svar1_outer: i32;
            let mut svar2_outer: i32;
            if same_sign {
                svar1_outer = i32::from(state.a[0]);
                svar2_outer = -svar1_outer;
                if -0x2000 < svar2_outer {
                    svar2_outer = if svar2_outer < 0x2000 {
                        (svar2_outer >> 5) + i32::from(local_2c)
                    } else {
                        i32::from(local_2c) + 0xff
                    };
                } else {
                    svar2_outer = i32::from(local_2c) + -0x100;
                }
            } else {
                svar2_outer = i32::from(state.a[0]);
                svar1_outer = svar2_outer;
                if svar2_outer < -0x1fff {
                    svar2_outer = i32::from(local_2c) + -0x100;
                } else {
                    svar2_outer = if svar2_outer < 0x2000 {
                        (svar2_outer >> 5) + i32::from(local_2c)
                    } else {
                        i32::from(local_2c) + 0xff
                    };
                }
            }
            let (local_2c_new, svar2_new, local_14_new): (i16, i32, i32) =
                if state.pk[1] as u16 == uvar5 {
                    if svar2_outer < -0x307f {
                        (-0x3000, 0x6c00, -0x6c00)
                    } else if svar2_outer < 0x2f80 {
                        let l2c = svar2_outer + 0x80;
                        let sv2 = 0x3c00 - l2c;
                        (l2c as i16, sv2, -sv2)
                    } else {
                        (0x3000, 0xc00, -0xc00)
                    }
                } else if svar2_outer < -0x2f7f {
                    (-0x3000, 0x6c00, -0x6c00)
                } else if 0x307f < svar2_outer {
                    (0x3000, 0xc00, -0xc00)
                } else {
                    let l2c = svar2_outer - 0x80;
                    let sv2 = 0x3c00 - l2c;
                    (l2c as i16, sv2, -sv2)
                };
            local_2c = local_2c_new;
            svar2 = svar2_new;
            let local_14 = local_14_new;
            state.a[1] = local_2c;
            let mut svar1 = if same_sign { svar1_outer } else { svar1_outer };
            svar1 = svar1 - (svar1 >> 8);
            state.a[0] = svar1 as i16;
            if same_sign {
                svar1 += 0xc0;
                state.a[0] = svar1 as i16;
                if svar1 < local_14 {
                    state.a[0] = (-svar2) as i16;
                } else if svar2 < svar1 {
                    state.a[0] = svar2 as i16;
                }
            } else {
                svar1 += -0xc0;
                state.a[0] = svar1 as i16;
                if local_14 > svar1 {
                    state.a[0] = (-svar2) as i16;
                } else if svar2 < svar1 {
                    state.a[0] = svar2 as i16;
                }
            }
            let _ = svar1_outer;
        }

        for i in 0..6usize {
            let b = state.b[i];
            state.b[i] = if code_size == 5 {
                b.wrapping_sub(b >> 9)
            } else {
                b.wrapping_sub(b >> 8)
            };
            if (dq & 0x7fff) != 0 {
                if (state.dq[i] ^ (dq as i16)) < 0 {
                    state.b[i] = state.b[i].wrapping_sub(0x80);
                } else {
                    state.b[i] = state.b[i].wrapping_add(0x80);
                }
            }
        }
    }
    state.dq[5] = state.dq[4];
    state.dq[4] = state.dq[3];
    state.dq[3] = state.dq[2];
    state.dq[2] = state.dq[1];
    state.dq[1] = state.dq[0];
    let uvar6 = i32::from(uvar6);
    if uvar6 == 0 {
        state.dq[0] = ((((dq >> 0x1f) as u16) & 0xfc00) + 0x20) as i16;
        state.sr[1] = state.sr[0];
    } else {
        let mut ivar9 = 0i32;
        loop {
            if (uvar6 as i16) < POWER2[ivar9 as usize] {
                break;
            }
            ivar9 += 1;
            if ivar9 == 0xf {
                break;
            }
        }
        let svar1 = if dq < 0 {
            (ivar9 << 6)
                + -0x400
                + i32::from(((i32::from(uvar6 as i16) << 6) >> (ivar9 & 0x1f)) as i16)
        } else {
            ivar9 * 0x40 + i32::from(((i32::from(uvar6 as i16) << 6) >> (ivar9 & 0x1f)) as i16)
        };
        state.dq[0] = svar1 as i16;
        state.sr[1] = state.sr[0];
    }
    if sr == 0 {
        state.sr[0] = 0x20;
    } else if sr < 1 {
        if sr < -0x7fff {
            state.sr[0] = -0x3e0;
        } else {
            let mut ivar9 = 0i32;
            loop {
                if i32::from(-(sr as i16)) < i32::from(POWER2[ivar9 as usize]) {
                    break;
                }
                ivar9 += 1;
                if ivar9 == 0xf {
                    break;
                }
            }
            state.sr[0] = ((ivar9 << 6)
                + -0x400
                + i32::from(((i32::from(-(sr as i16)) << 6) >> (ivar9 & 0x1f)) as i16))
                as i16;
        }
    } else {
        let mut ivar9 = 0i32;
        loop {
            if sr < i32::from(POWER2[ivar9 as usize]) {
                break;
            }
            ivar9 += 1;
            if ivar9 == 0xf {
                break;
            }
        }
        state.sr[0] = ((ivar9 << 6) + i32::from(((sr << 6) >> (ivar9 & 0x1f)) as i16)) as i16;
    }
    state.pk[1] = local_2a;
    state.pk[0] = uvar5 as i16;
    if tr {
        state.td = 0;
    } else {
        state.td = u8::from(local_2c < -0x2e00);
    }
    let svar2 = state
        .dms
        .wrapping_add(((fi - i32::from(state.dms)) >> 5) as i16);
    state.dms = svar2;
    let svar1 = state
        .dml
        .wrapping_add(((fi * 4 - i32::from(state.dml)) >> 7) as i16);
    state.dml = svar1;
    if bvar10 && svar8 < uvar6 {
        state.ap = 0x100;
        return;
    }
    if (0x5ff < y)
        && state.td != 1
        && ((i32::from(svar2) * 4 - i32::from(svar1)).unsigned_abs() as i32)
            < (i32::from(svar1) >> 3)
    {
        state.ap = state.ap.wrapping_add((-i32::from(state.ap) >> 4) as i16);
        return;
    }
    state.ap = state
        .ap
        .wrapping_add(((0x200 - i32::from(state.ap)) >> 4) as i16);
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
fn tandem_adjust_alaw(sr: i32, se: i32, y: i32, i: i32, sign: i32, qtab: &[i16]) -> i32 {
    let ivar1 = if -0x8000 < sr { (sr >> 1) * 8 } else { -8 };
    let a_val = linear2alaw(ivar1);
    let ivar1 = alaw2linear(a_val);
    let ivar1 = quantize(
        i32::from(((ivar1 >> 2) as i16).wrapping_sub(se as i16)),
        y,
        qtab,
        sign - 1,
    );
    if i32::from(ivar1 as i8) == i {
        return i32::from(a_val);
    }
    if (sign ^ i) < (i32::from(ivar1 as i8) ^ sign) {
        if (a_val as i8) < 0 {
            if a_val == 0xd5 {
                return 0x55;
            }
            return (i32::from(a_val ^ 0x55) - 1) ^ 0x55;
        }
        if a_val == 0x2a {
            return 0x2a;
        }
    } else {
        if (a_val as i8) >= 0 {
            if a_val == 0x55 {
                return 0xd5;
            }
            return (i32::from(a_val ^ 0x55) - 1) ^ 0x55;
        }
        if a_val == 0xaa {
            return 0xaa;
        }
    }
    (i32::from(a_val ^ 0x55) + 1) ^ 0x55
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
fn tandem_adjust_ulaw(sr: i32, se: i32, y: i32, i: i32, sign: i32, qtab: &[i16]) -> i32 {
    let u_val = linear2ulaw((sr * 4) & (i32::from(sr < -0x7fff) - 1));
    let uvar1 = u32::from(u_val);
    let ivar2 = ulaw2linear(u_val);
    let ivar2 = quantize(
        i32::from(((ivar2 >> 2) as i16).wrapping_sub(se as i16)),
        y,
        qtab,
        sign - 1,
    );
    if i32::from(ivar2 as i8) == i {
        return uvar1 as i32;
    }
    if (sign ^ i) < (i32::from(ivar2 as i8) ^ sign) {
        if (u_val as i8) >= 0 {
            return (uvar1 as i32 - 1) & i32::from(u_val != 0);
        }
        if u_val == 0xff {
            return 0x7e;
        }
    } else {
        if (u_val as i8) < 0 {
            if u_val != 0x80 {
                return uvar1 as i32 - 1;
            }
            return 0x80;
        }
        if u_val == 0x7f {
            return 0xfe;
        }
    }
    uvar1 as i32 + 1
}

const DQLNTAB_721: [i16; 16] = [
    -2048, 4, 135, 213, 273, 323, 373, 425, 425, 373, 323, 273, 213, 135, 4, -2048,
];
const FITAB_721: [i16; 16] = [
    0, 0, 0, 512, 512, 512, 1536, 3584, 3584, 1536, 512, 512, 512, 0, 0, 0,
];
const WITAB_721: [i16; 16] = [
    -12, 18, 41, 64, 112, 198, 355, 1122, 1122, 355, 198, 112, 64, 41, 18, -12,
];
const QTAB_721: [i16; 7] = [-124, 80, 178, 246, 300, 349, 400];

const DQLNTAB_723_24: [i16; 8] = [-2048, 135, 273, 373, 373, 273, 135, -2048];
const FITAB_723_24: [i16; 8] = [0, 512, 1024, 3584, 3584, 1024, 512, 0];
const WITAB_723_24: [i16; 8] = [-128, 960, 4384, 18624, 18624, 4384, 960, -128];
const QTAB_723_24: [i16; 3] = [8, 218, 331];

const DQLNTAB_723_40: [i16; 32] = [
    -2048, -66, 28, 104, 169, 224, 274, 318, 358, 395, 429, 459, 488, 514, 539, 566, 566, 539, 514,
    488, 459, 429, 395, 358, 318, 274, 224, 169, 104, 28, -66, -2048,
];
const FITAB_723_40: [i16; 32] = [
    0, 0, 0, 0, 0, 512, 512, 512, 512, 512, 1024, 1536, 2048, 2560, 3072, 3072, 3072, 3072, 2560,
    2048, 1536, 1024, 512, 512, 512, 512, 512, 0, 0, 0, 0, 0,
];
const WITAB_723_40: [i16; 32] = [
    448, 448, 768, 1248, 1280, 1312, 1856, 3200, 4512, 5728, 7008, 8960, 11456, 14080, 16928,
    22272, 22272, 16928, 14080, 11456, 8960, 7008, 5728, 4512, 3200, 1856, 1312, 1280, 1248, 768,
    448, 448,
];
const QTAB_723_40: [i16; 15] = [
    -122, -16, 68, 139, 198, 250, 298, 339, 378, 413, 445, 475, 502, 528, 553,
];

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
fn g721_decoder(i: i32, out_coding: i32, state: &mut G72xState) -> i32 {
    let i_00 = i & 0xf;
    let ivar3 = predictor_zero(state);
    let ivar4 = predictor_pole(state);
    let ivar5 = step_size(state);
    let svar6 = i32::from((ivar3 as i16).wrapping_add(ivar4 as i16)) >> 1;
    let ivar4 = i32::from(ivar5 as i16);
    let ivar5 = reconstruct(i & 8, i32::from(DQLNTAB_721[i_00 as usize]), ivar4);
    let uvar2 = ivar5 as u16;
    let uvar1 = if (uvar2 as i16) < 0 {
        -i32::from(uvar2 & 0x3fff)
    } else {
        i32::from(uvar2)
    };
    let ivar5 = i32::from((svar6 + uvar1) as i16);
    update(
        4,
        ivar4,
        i32::from(WITAB_721[i_00 as usize]) << 5,
        i32::from(FITAB_721[i_00 as usize]),
        i32::from(uvar2),
        ivar5,
        i32::from(((ivar5 - svar6) + i32::from(ivar3 as i16 >> 1)) as i16),
        state,
    );
    if out_coding == 2 {
        return tandem_adjust_alaw(ivar5, svar6, ivar4, i_00, 8, &QTAB_721);
    }
    if out_coding == 3 {
        return ivar5 * 4;
    }
    if out_coding != 1 {
        return -1;
    }
    tandem_adjust_ulaw(ivar5, svar6, ivar4, i_00, 8, &QTAB_721)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
fn g723_24_decoder(i: i32, out_coding: i32, state: &mut G72xState) -> i32 {
    let i_00 = i & 7;
    let ivar3 = predictor_zero(state);
    let ivar4 = predictor_pole(state);
    let ivar5 = step_size(state);
    let svar6 = i32::from((ivar3 as i16).wrapping_add(ivar4 as i16)) >> 1;
    let ivar4 = i32::from(ivar5 as i16);
    let ivar5 = reconstruct(i & 4, i32::from(DQLNTAB_723_24[i_00 as usize]), ivar4);
    let uvar2 = ivar5 as u16;
    let uvar1 = if (uvar2 as i16) < 0 {
        -i32::from(uvar2 & 0x3fff)
    } else {
        i32::from(uvar2)
    };
    let ivar5 = i32::from((svar6 + uvar1) as i16);
    update(
        3,
        ivar4,
        i32::from(WITAB_723_24[i_00 as usize]),
        i32::from(FITAB_723_24[i_00 as usize]),
        i32::from(uvar2),
        ivar5,
        i32::from(((ivar5 - svar6) + i32::from(ivar3 as i16 >> 1)) as i16),
        state,
    );
    if out_coding == 2 {
        return tandem_adjust_alaw(ivar5, svar6, ivar4, i_00, 4, &QTAB_723_24);
    }
    if out_coding == 3 {
        return ivar5 * 4;
    }
    if out_coding != 1 {
        return -1;
    }
    tandem_adjust_ulaw(ivar5, svar6, ivar4, i_00, 4, &QTAB_723_24)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
fn g723_40_decoder(i: i32, out_coding: i32, state: &mut G72xState) -> i32 {
    let i_00 = i & 0x1f;
    let ivar3 = predictor_zero(state);
    let ivar4 = predictor_pole(state);
    let ivar5 = step_size(state);
    let svar6 = i32::from((ivar3 as i16).wrapping_add(ivar4 as i16)) >> 1;
    let ivar4 = i32::from(ivar5 as i16);
    let ivar5 = reconstruct(i & 0x10, i32::from(DQLNTAB_723_40[i_00 as usize]), ivar4);
    let uvar2 = ivar5 as u16;
    let uvar1 = if (uvar2 as i16) < 0 {
        -i32::from(uvar2 & 0x7fff)
    } else {
        i32::from(uvar2)
    };
    let ivar5 = i32::from((svar6 + uvar1) as i16);
    update(
        5,
        ivar4,
        i32::from(WITAB_723_40[i_00 as usize]),
        i32::from(FITAB_723_40[i_00 as usize]),
        i32::from(uvar2),
        ivar5,
        i32::from(((ivar5 - svar6) + i32::from(ivar3 as i16 >> 1)) as i16),
        state,
    );
    if out_coding == 2 {
        return tandem_adjust_alaw(ivar5, svar6, ivar4, i_00, 0x10, &QTAB_723_40);
    }
    if out_coding == 3 {
        return ivar5 * 4;
    }
    if out_coding != 1 {
        return -1;
    }
    tandem_adjust_ulaw(ivar5, svar6, ivar4, i_00, 0x10, &QTAB_723_40)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
pub fn g721_encoder(sl: i32, in_coding: i32, state: &mut G72xState) -> i32 {
    let svar6 = match in_coding {
        2 => (alaw2linear(sl as u8) >> 2) as i16,
        3 => (sl >> 2) as i16,
        1 => (ulaw2linear(sl as u8) >> 2) as i16,
        _ => return -1,
    };
    let ivar4 = predictor_zero(state);
    let ivar5 = predictor_pole(state);
    let svar2 = ((i32::from(ivar4 as i16) + ivar5) as u32 >> 1) as i16;
    let ivar5 = i32::from(step_size(state) as i16);
    let y = ivar5;
    let ivar5 = quantize(i32::from(svar6.wrapping_sub(svar2)), y, &QTAB_721, 7);
    let uvar7 = i32::from(ivar5 as i16 as u16);
    let ivar5 = reconstruct(uvar7 & 8, i32::from(DQLNTAB_721[uvar7 as usize]), y);
    let uvar3 = ivar5 as u16;
    let uvar1 = if (uvar3 as i16) < 0 {
        -i32::from(uvar3 & 0x3fff)
    } else {
        i32::from(uvar3)
    };
    let local_30 = i32::from(svar2) + uvar1;
    update(
        4,
        y,
        i32::from(WITAB_721[uvar7 as usize]) << 5,
        i32::from(FITAB_721[uvar7 as usize]),
        i32::from(uvar3),
        local_30,
        i32::from((i32::from(ivar4 as i16 >> 1) - i32::from(svar2) + local_30) as i16),
        state,
    );
    uvar7
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
pub fn g723_24_encoder(sl: i32, in_coding: i32, state: &mut G72xState) -> i32 {
    let svar5 = match in_coding {
        2 => (alaw2linear(sl as u8) >> 2) as i16,
        3 => (sl >> 2) as i16,
        1 => (ulaw2linear(sl as u8) >> 2) as i16,
        _ => return -1,
    };
    let ivar3 = predictor_zero(state);
    let ivar4 = predictor_pole(state);
    let svar7 = i32::from((ivar3 as i16).wrapping_add(ivar4 as i16)) >> 1;
    let ivar4 = step_size(state);
    let y = i32::from(ivar4 as i16);
    let ivar4 = quantize(
        i32::from(svar5.wrapping_sub(svar7 as i16)),
        y,
        &QTAB_723_24,
        3,
    );
    let uvar6 = i32::from(ivar4 as i16 as u16);
    let ivar4 = reconstruct(uvar6 & 4, i32::from(DQLNTAB_723_24[uvar6 as usize]), y);
    let uvar2 = ivar4 as u16;
    let uvar1 = if (uvar2 as i16) < 0 {
        -i32::from(uvar2 & 0x3fff)
    } else {
        i32::from(uvar2)
    };
    let local_20 = svar7 + uvar1;
    update(
        3,
        y,
        i32::from(WITAB_723_24[uvar6 as usize]),
        i32::from(FITAB_723_24[uvar6 as usize]),
        i32::from(uvar2),
        local_20,
        i32::from((ivar3 as i16 >> 1) - svar7 as i16 + local_20 as i16),
        state,
    );
    uvar6
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
pub fn g723_40_encoder(sl: i32, in_coding: i32, state: &mut G72xState) -> i32 {
    let svar5 = match in_coding {
        2 => (alaw2linear(sl as u8) >> 2) as i16,
        3 => (sl >> 2) as i16,
        1 => (ulaw2linear(sl as u8) >> 2) as i16,
        _ => return -1,
    };
    let ivar3 = predictor_zero(state);
    let ivar4 = predictor_pole(state);
    let svar7 = i32::from((ivar3 as i16).wrapping_add(ivar4 as i16)) >> 1;
    let ivar4 = step_size(state);
    let y = i32::from(ivar4 as i16);
    let ivar4 = quantize(
        i32::from(svar5.wrapping_sub(svar7 as i16)),
        y,
        &QTAB_723_40,
        0xf,
    );
    let uvar6 = i32::from(ivar4 as i16 as u16);
    let ivar4 = reconstruct(uvar6 & 0x10, i32::from(DQLNTAB_723_40[uvar6 as usize]), y);
    let uvar2 = ivar4 as u16;
    let uvar1 = if (uvar2 as i16) < 0 {
        -i32::from(uvar2 & 0x7fff)
    } else {
        i32::from(uvar2)
    };
    let local_20 = svar7 + uvar1;
    update(
        5,
        y,
        i32::from(WITAB_723_40[uvar6 as usize]),
        i32::from(FITAB_723_40[uvar6 as usize]),
        i32::from(uvar2),
        local_20,
        i32::from((ivar3 as i16 >> 1) - svar7 as i16 + local_20 as i16),
        state,
    );
    uvar6
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
fn unpack_input_buf(
    pin_buffer: &mut u32,
    pin_bits: &mut i32,
    bits: i32,
    pb_buffer: &[u8],
    pn_number: &mut usize,
    n_byte_number: usize,
) -> (i32, i32) {
    let mut uvar4 = *pin_buffer;
    let mut ivar3 = *pin_bits;
    if ivar3 < bits {
        if n_byte_number <= *pn_number {
            return (0, -1);
        }
        let bvar1 = pb_buffer[*pn_number];
        *pn_number += 1;
        uvar4 |= u32::from(bvar1) << (ivar3 & 0x1f);
        ivar3 += 8;
    }
    let code = (uvar4 as u8) & (((1u32 << (bits & 0x1f)) - 1) as u8);
    *pin_buffer = uvar4 >> (bits & 0x1f);
    *pin_bits = ivar3 - bits;
    (i32::from(code), i32::from(0 < ivar3 - bits))
}

type DecoderRoutine = fn(i32, i32, &mut G72xState) -> i32;

#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
pub fn g72x_decode(bytes: &[u8], n_sample_number: i32, n_codec_mode: i32) -> Vec<i16> {
    let (dec_bits, routine): (i32, DecoderRoutine) = match n_codec_mode {
        2 => (4, g721_decoder),
        3 => (3, g723_24_decoder),
        4 => (5, g723_40_decoder),
        _ => return Vec::new(),
    };
    let out_coding = 3;
    let out_size = 2;
    let n_byte_number_00 = ((n_sample_number * dec_bits) >> 3) + 4;
    let n_byte_number_00 = n_byte_number_00.min(bytes.len() as i32);
    let cap = ((out_size * 8 * n_byte_number_00) / dec_bits) as usize;
    let mut out: Vec<i16> = Vec::with_capacity(cap);
    let mut state = g72x_init_state();
    let mut in_buffer: u32 = 0;
    let mut in_bits: i32 = 0;
    let mut nb: usize = 0;
    loop {
        let (code, status) = unpack_input_buf(
            &mut in_buffer,
            &mut in_bits,
            dec_bits,
            bytes,
            &mut nb,
            n_byte_number_00 as usize,
        );
        if status < 0 {
            break;
        }
        let uvar1 = routine(code, out_coding, &mut state);
        if out_size == 2 {
            out.push(uvar1 as i16);
        } else {
            out.push((uvar1 & 0xff) as i16);
        }
    }
    out
}

#[derive(Debug, Clone, Copy)]
pub struct G729Unsupported;

impl std::fmt::Display for G729Unsupported {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "G729 (G.729EV) decoder core not ported (frame splitting via GetCodecByteNumber and \
             skipping the first 0xa0 samples are ported. Not used because the bundled data is KOREAPCM=URAW)"
        )
    }
}

/// Decodes G.729 audio; unsupported in this port.
///
/// # Errors
///
/// Always returns [`G729Unsupported`].
pub const fn g729_decode(
    _bytes: &[u8],
    _n_byte_number: i32,
    _n_used_rate: i32,
) -> Result<Vec<i16>, G729Unsupported> {
    Err(G729Unsupported)
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "G72x/G729 codec: intentional truncation per C reference"
)]
/// Decodes a PCM file with the given codec mode.
///
/// # Errors
///
/// Returns an error if the input is invalid.
pub fn decode_pcm(
    codec_mode: u8,
    file_bytes: &[u8],
    w_pcm_size: u16,
    sample_rate: i32,
) -> Result<Vec<i16>, String> {
    let n = w_pcm_size as usize;
    match codec_mode {
        0 => {
            if file_bytes.len() < n * 2 {
                return Err(format!(
                    "codec=RAW: not enough data ({}B < {}B)",
                    file_bytes.len(),
                    n * 2
                ));
            }
            let mut out = Vec::with_capacity(n);
            for i in 0..n {
                out.push(i16::from_le_bytes([
                    file_bytes[i * 2],
                    file_bytes[i * 2 + 1],
                ]));
            }
            Ok(out)
        }
        1 => {
            if file_bytes.len() < n {
                return Err(format!(
                    "codec=URAW: not enough data ({}B < {}B)",
                    file_bytes.len(),
                    n
                ));
            }
            Ok(file_bytes[..n]
                .iter()
                .map(|&b| ktts_dict::synthdb::uraw_to_pcm(b as i8))
                .collect())
        }
        2..=4 => {
            if file_bytes.len() < n {
                return Err(format!(
                    "codec=G72x: not enough data ({}B < {}B)",
                    file_bytes.len(),
                    n
                ));
            }
            let decoded = g72x_decode(
                &file_bytes[..n],
                i32::from(w_pcm_size),
                i32::from(codec_mode),
            );
            let mut out = decoded;
            out.truncate(n);
            out.resize(n, 0);
            Ok(out)
        }
        5 => {
            let byte_count = get_codec_byte_number(w_pcm_size, sample_rate) as usize;
            if file_bytes.len() < byte_count {
                return Err(format!(
                    "codec=G729: not enough data ({}B < {}B)",
                    file_bytes.len(),
                    byte_count
                ));
            }
            let decoded = g729_decode(&file_bytes[..byte_count], byte_count as i32, sample_rate)
                .map_err(|e| e.to_string())?;
            let mut out: Vec<i16> = decoded.iter().skip(0xa0).take(n).copied().collect();
            out.resize(n, 0);
            Ok(out)
        }
        other => Err(format!("unknown codec mode: {other}")),
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap,
        reason = "test fixtures: oracle values converted with intentional casts"
    )]
    use super::*;

    #[test]
    fn codec_byte_number_formula() {
        assert_eq!(get_codec_byte_number(160, 16000), 3 * 40);
        assert_eq!(get_codec_byte_number(320, 16000), 4 * 40);
        assert_eq!(get_codec_byte_number(161, 16000), 4 * 40);
        assert_eq!(get_codec_byte_number(160, 8000), 2 * 20);
    }

    #[test]
    fn codec_from_name_mapping() {
        assert_eq!(codec_from_name(""), Some(0));
        assert_eq!(codec_from_name("URAW"), Some(1));
        assert_eq!(codec_from_name("G721"), Some(2));
        assert_eq!(codec_from_name("G723_24"), Some(3));
        assert_eq!(codec_from_name("G723_40"), Some(4));
        assert_eq!(codec_from_name("G729"), Some(5));
        assert_eq!(codec_from_name("BOGUS"), None);
    }

    #[test]
    fn raw_pcm_roundtrip() {
        let src = [1000i16, -2000, 0, 32767, -32768];
        let mut bytes = Vec::new();
        for s in src {
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        let out = decode_pcm(CODEC_RAW, &bytes, src.len() as u16, 16000).unwrap();
        assert_eq!(out, src);
        assert!(decode_pcm(CODEC_RAW, &bytes[..4], 5, 16000).is_err());
    }

    #[test]
    fn uraw_decode_matches_dict() {
        let bytes = [0xc1u8, 0x00, 0xff, 0x7f];
        let out = decode_pcm(CODEC_URAW, &bytes, 4, 16000).unwrap();
        assert_eq!(out.len(), 4);
        assert_eq!(out[0], ktts_dict::synthdb::uraw_to_pcm(0xc1u8 as i8));
        assert_eq!(out[2], ktts_dict::synthdb::uraw_to_pcm(-1));
    }

    #[test]
    fn g711_roundtrip() {
        for pcm in [-32768i32, -1000, -8, -1, 0, 1, 8, 1000, 32767] {
            let a = alaw2linear(linear2alaw(pcm));
            let u = ulaw2linear(linear2ulaw(pcm));
            let lim = if pcm.abs() >= 32000 {
                2048
            } else {
                (pcm.abs() >> 4) + 256
            };
            assert!((a - pcm).abs() <= lim, "alaw {pcm} → {a}");
            assert!((u - pcm).abs() <= lim, "ulaw {pcm} → {u}");
        }
        assert_eq!(linear2ulaw(0), 0xff);
        assert_eq!(linear2ulaw(1), 0xff);
        assert_eq!(linear2ulaw(8), 0xfe);
        assert_eq!(linear2ulaw(-1), 0x7f);
        assert_eq!(linear2alaw(0), 0xd5);
        assert_eq!(linear2alaw(-8), 0x55);
        assert_eq!(linear2alaw(-1), 0x5a);
        assert_eq!(alaw2linear(0x5a), -248);
    }

    #[test]
    fn g72x_init_values() {
        let s = g72x_init_state();
        assert_eq!(s.yl, 0x8800);
        assert_eq!(s.yu, 0x220);
        assert_eq!(s.sr, [0x20, 0x20]);
        assert_eq!(s.dq, [0x20; 6]);
        assert_eq!(s.td, 0);
    }

    #[test]
    fn g721_roundtrip_sine() {
        let mut enc = g72x_init_state();
        let mut dec = g72x_init_state();
        let n = 2000;
        let mut codes = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f32 * 2.0 * std::f32::consts::PI * 100.0 / 8000.0;
            let s = (t.sin() * 2000.0) as i32;
            codes.push(g721_encoder(s, 3, &mut enc));
        }
        let decoded: Vec<i32> = codes
            .iter()
            .map(|&c| g721_decoder(c, 3, &mut dec))
            .collect();
        let err: i64 = (0..n)
            .map(|i| {
                let t = i as f32 * 2.0 * std::f32::consts::PI * 100.0 / 8000.0;
                let s = (t.sin() * 2000.0) as i32;
                i64::from((decoded[i] - s).abs())
            })
            .sum();
        let mean = err as f64 / n as f64;
        assert!(mean < 400.0, "G.721 mean error {mean} too large");
        let mut dec2 = g72x_init_state();
        for _ in 0..100 {
            g721_decoder(8, 3, &mut dec2);
        }
    }

    #[test]
    fn g723_24_roundtrip_sine() {
        let mut enc = g72x_init_state();
        let mut dec = g72x_init_state();
        let n = 2000;
        let mut codes = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f32 * 2.0 * std::f32::consts::PI * 100.0 / 8000.0;
            let s = (t.sin() * 2000.0) as i32;
            codes.push(g723_24_encoder(s, 3, &mut enc));
        }
        let decoded: Vec<i32> = codes
            .iter()
            .map(|&c| g723_24_decoder(c, 3, &mut dec))
            .collect();
        let err: i64 = (0..n)
            .map(|i| {
                let t = i as f32 * 2.0 * std::f32::consts::PI * 100.0 / 8000.0;
                let s = (t.sin() * 2000.0) as i32;
                i64::from((decoded[i] - s).abs())
            })
            .sum();
        let mean = err as f64 / n as f64;
        assert!(mean < 600.0, "G.723_24 mean error {mean} too large");
    }

    #[test]
    fn g723_40_roundtrip_sine() {
        let mut enc = g72x_init_state();
        let mut dec = g72x_init_state();
        let n = 2000;
        let mut codes = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f32 * 2.0 * std::f32::consts::PI * 100.0 / 8000.0;
            let s = (t.sin() * 2000.0) as i32;
            codes.push(g723_40_encoder(s, 3, &mut enc));
        }
        let decoded: Vec<i32> = codes
            .iter()
            .map(|&c| g723_40_decoder(c, 3, &mut dec))
            .collect();
        let err: i64 = (0..n)
            .map(|i| {
                let t = i as f32 * 2.0 * std::f32::consts::PI * 100.0 / 8000.0;
                let s = (t.sin() * 2000.0) as i32;
                i64::from((decoded[i] - s).abs())
            })
            .sum();
        let mean = err as f64 / n as f64;
        assert!(mean < 200.0, "G.723_40 mean error {mean} too large");
    }

    #[test]
    fn g72x_decode_wrapper() {
        let mut enc = g72x_init_state();
        let n = 800;
        let mut bytes = Vec::new();
        let mut acc = 0u32;
        let mut bits = 0i32;
        for i in 0..n {
            let t = i as f32 * 2.0 * std::f32::consts::PI * 200.0 / 8000.0;
            let s = (t.sin() * 3000.0) as i32;
            let code = g721_encoder(s, 3, &mut enc);
            acc |= (code as u32 & 0xf) << (bits & 0x1f);
            bits += 4;
            while bits >= 8 {
                bytes.push((acc & 0xff) as u8);
                acc >>= 8;
                bits -= 8;
            }
        }
        if bits > 0 {
            bytes.push((acc & 0xff) as u8);
        }
        let out = g72x_decode(&bytes, n, 2);
        assert!(!out.is_empty());
        let peak = out.iter().map(|&s| s.abs()).max().unwrap_or(0);
        assert!(peak > 1000, "G.721 decode output peak {peak}");
    }
}
