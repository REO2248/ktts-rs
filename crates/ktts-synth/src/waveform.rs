use crate::SynthResult;
use crate::consts::{
    PSOLA_CLIP, REST_SENT_END, REST_SPACE, REST_WORD, REST_WORD_STRONG, TWO_PI, VOLUME_CLIP,
};
use crate::context::Phrase;
use crate::setting::SynthParams;
use crate::unitselect::Selection;
use ktts_dict::synthdb::{UpmMark, decode_marks};

#[derive(Debug, Clone)]
pub struct UnitEntry {
    pub pcm: Vec<i16>,
    pub n_pcm_start: u32,
    pub n_upm_start: u32,
    pub w_pcm_size: u16,
    pub w_upm_size: u16,
    pub ch_dict_file_id: i8,
    pub ch_rest_flag: u8,
    pub upm: Vec<u8>,
}

impl UnitEntry {
    const fn rest(w_pcm_size: u16) -> Self {
        Self {
            pcm: Vec::new(),
            n_pcm_start: 0,
            n_upm_start: 0,
            w_pcm_size,
            w_upm_size: 0,
            ch_dict_file_id: 0,
            ch_rest_flag: 1,
            upm: Vec::new(),
        }
    }
}

struct ResultInfo {
    buf: Vec<i16>,
    w_prev_pitch: u16,
    w_next_pitch: u16,
    params: SynthParams,
    tail: Vec<i16>,
}

impl ResultInfo {
    fn new(params: SynthParams) -> Self {
        Self {
            buf: Vec::new(),
            w_prev_pitch: 0,
            w_next_pitch: 0,
            params,
            tail: vec![0i16; 1076],
        }
    }
    #[expect(
        clippy::cast_sign_loss,
        reason = "C port: index/math casts with wrap semantics"
    )]
    fn tail_at(&self, idx: i32) -> i16 {
        if idx < 0 {
            return 0;
        }
        self.tail.get(idx as usize).copied().unwrap_or(0)
    }
}

fn end_rest_length_set(phrase: &Phrase) -> u16 {
    match phrase.words[phrase.word_num() - 1].rest_flag {
        0x60 => REST_SENT_END,
        0x61 => REST_SPACE,
        _ => 0,
    }
}

fn rest_length_set(
    ctx: &crate::SynthDb,
    phrase: &Phrase,
    selection: &Selection,
    ave_length: &mut [u16],
) -> SynthResult<()> {
    for a in ave_length.iter_mut() {
        *a = 0;
    }
    let word_num = phrase.word_num();
    if word_num < 2 {
        return Ok(());
    }
    for wi in 1..word_num {
        let prev = &phrase.words[wi - 1];
        let cur = &phrase.words[wi];
        let cur_first = cur.letters.start;
        if prev.rest_flag == 0x60 {
            ave_length[cur_first] = REST_SENT_END;
            continue;
        }
        if prev.rest_flag == 0x61 {
            ave_length[cur_first] = REST_SPACE;
            continue;
        }
        let prev_last = prev.letters.end - 1;
        let cur_first = cur.letters.start;
        let prev_phone = selection.best[prev_last].iter().flatten().last();
        let cur_phone = selection.best[cur_first].iter().flatten().last();
        let prev_ok = match prev_phone {
            Some(bp) => {
                let rec = ctx.rec(bp)?;
                rec.phones[3].wrapping_add(0xa0) < 2
            }
            None => true,
        };
        let cur_ok = match cur_phone {
            Some(bp) => {
                let rec = ctx.rec(bp)?;
                rec.phones[1].wrapping_add(0xa0) < 2
            }
            None => true,
        };
        if prev_ok && cur_ok {
            ave_length[cur_first] = REST_WORD;
            continue;
        }
        let prev_letter = &phrase.letters[prev_last];
        let mut len = 0u16;
        if prev_letter.f_jong == -2 {
            if prev_letter.dict_type == 3 {
                len = REST_WORD_STRONG;
            } else {
                let cho = phrase.letters[cur_first].cvc[0];
                let raw_cho = if cho == 0 { 1 } else { cho };
                if raw_cho < 0x14 {
                    let u = 1u32 << (raw_cho & 0x1f);
                    if (u & 0xf9448) != 0 || (u & 0x2192) != 0 {
                        len = 0;
                    } else {
                        len = REST_WORD;
                    }
                } else {
                    len = REST_WORD;
                }
            }
        }
        ave_length[cur_first] = len;
    }
    Ok(())
}

fn all_phone_info(
    ctx: &crate::SynthDb,
    phrase: &Phrase,
    selection: &Selection,
    ave_length: &[u16],
) -> SynthResult<Vec<UnitEntry>> {
    let mut units: Vec<UnitEntry> = Vec::new();
    for (li, _letter) in phrase.letters.iter().enumerate() {
        if ave_length[li] != 0 {
            units.push(UnitEntry::rest(ave_length[li]));
        }
        for bp in selection.best[li].iter().flatten() {
            let rec = ctx.rec(bp)?;
            if rec.w_pcm_size != 0 {
                let pcm = ctx.pcm_segment(rec)?;
                units.push(UnitEntry {
                    pcm,
                    n_pcm_start: rec.n_pcm_start,
                    n_upm_start: rec.n_upm_start,
                    w_pcm_size: rec.w_pcm_size,
                    w_upm_size: rec.w_upm_size,
                    ch_dict_file_id: rec.ch_dict_file_id,
                    ch_rest_flag: if bp.f_type_index == 2 { 3 } else { 2 },
                    upm: ctx.upm_segment(rec)?,
                });
            }
        }
    }
    let end = end_rest_length_set(phrase);
    for _ in 0..10 {
        units.push(UnitEntry::rest(end / 10));
    }
    Ok(units)
}

fn upm_marks(unit: &UnitEntry) -> Vec<UpmMark> {
    let mode = if unit.ch_rest_flag == 2 { 2 } else { 0 };
    decode_marks(&unit.upm, mode)
}

#[inline]
#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
const fn clip_psola(v: i32) -> i16 {
    if v > PSOLA_CLIP {
        PSOLA_CLIP as i16
    } else if v < -PSOLA_CLIP {
        -PSOLA_CLIP as i16
    } else {
        v as i16
    }
}

fn convert_max_upm_info(n_pos: i32, marks: &[UpmMark]) -> Option<UpmMark> {
    let last = marks.last()?;
    if n_pos >= i32::from(last.addr) + i32::from(last.cur_pitch) + i32::from(last.next_pitch) {
        return None;
    }
    let mut best_dist = i32::MAX;
    let mut best = 0usize;
    for (i, m) in marks.iter().enumerate() {
        let d = (i32::from(m.addr) + i32::from(m.cur_pitch) - n_pos).abs();
        if d < best_dist {
            best_dist = d;
            best = i;
        }
    }
    Some(marks[best])
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn fast_path(res: &mut ResultInfo, units: &[UnitEntry], marks: &[UpmMark], n_index: usize) {
    let unit = &units[n_index];
    let n = marks.last().map_or(0, |m| i32::from(m.next_pitch));
    let p = i32::from(res.w_prev_pitch);
    let prev_tail: Option<&[i16]> = if n_index > 0 && p > 0 {
        let prev = &units[n_index - 1];
        let len = prev.pcm.len();
        if (p as usize) <= len {
            Some(&prev.pcm[len - p as usize..])
        } else {
            None
        }
    } else {
        None
    };
    let cur = &unit.pcm;
    let next_is_rest = units.get(n_index + 1).is_none_or(|u| u.ch_rest_flag == 1);
    let out_len = if next_is_rest {
        i32::from(unit.w_pcm_size)
    } else {
        i32::from(unit.w_pcm_size) - n
    };
    let d5 = TWO_PI / f64::from(p * 2 + 1);
    if out_len > 0 {
        let mut i = 1i32;
        while i <= out_len {
            if i - 1 < p {
                let w1 = (1.0 - (f64::from(i + p) * d5).cos()) * 0.5;
                let w2 = (1.0 - (f64::from(i) * d5).cos()) * 0.5;
                let prev_v = f64::from(
                    prev_tail
                        .and_then(|t| t.get((i - 1) as usize))
                        .copied()
                        .unwrap_or(0),
                );
                let cur_v = f64::from(cur.get((i - 1) as usize).copied().unwrap_or(0));
                let v = prev_v * w1 + cur_v * w2;
                res.buf.push(v as i16);
            } else {
                res.buf
                    .push(cur.get((i - 1) as usize).copied().unwrap_or(0));
            }
            i += 1;
        }
    }
    res.w_prev_pitch = n as u16;
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
#[expect(
    clippy::many_single_char_names,
    reason = "C port: original single-letter variable names"
)]
fn slow_path(
    res: &mut ResultInfo,
    units: &[UnitEntry],
    marks: &[UpmMark],
    n_index: usize,
) -> (Vec<i16>, Option<(usize, usize)>) {
    let unit = &units[n_index];
    let params = res.params;
    let speed = params.speed;
    let pitch = params.pitch;
    let mut n_upm_pitch_length = marks.first().map_or(0, |m| i32::from(m.cur_pitch));
    let mut n_process = 0i32;
    let mut s_pitch_tmp: Option<UpmMark> = None;
    let mut out: Vec<i16> = Vec::new();
    let mut window = [0i16; 1024];
    let mut last_frame_len = i32::from(unit.w_pcm_size);
    loop {
        let pos = (n_upm_pitch_length as f32 / speed) as i32;
        let found = convert_max_upm_info(pos, marks);
        let Some(tmp) = found else {
            let t = s_pitch_tmp.unwrap_or(UpmMark {
                addr: 0,
                cur_pitch: 0,
                next_pitch: 0,
            });
            let next_is_rest = units.get(n_index + 1).is_none_or(|u| u.ch_rest_flag == 1);
            if next_is_rest {
                res.w_prev_pitch = u16::from(t.next_pitch);
                return (out, None);
            }
            let frame_len = last_frame_len.max(0) as usize;
            let start = (n_upm_pitch_length - last_frame_len).max(0) as usize;
            let src: Vec<i16> = out.iter().skip(start).take(frame_len).copied().collect();
            let copy_len = src.len().min(res.tail.len());
            res.tail[..copy_len].copy_from_slice(&src[..copy_len]);
            res.w_prev_pitch = u16::from(t.cur_pitch);
            res.w_next_pitch = u16::from(t.next_pitch);
            return (out, None);
        };
        s_pitch_tmp = Some(tmp);
        let p = i32::from(tmp.cur_pitch);
        let n = i32::from(tmp.next_pitch);
        let addr = tmp.addr as usize;
        let mut p_prime = (p as f32 / pitch) as i32;
        if p_prime < p / 16 {
            p_prime = p / 16;
        }
        if p_prime > p * 2 {
            p_prime = p * 2;
        }
        if n_upm_pitch_length < (p_prime - p) + n_process {
            p_prime = n_upm_pitch_length + p - n_process;
        }
        let frame_len = p + n;
        last_frame_len = frame_len;
        let d5 = TWO_PI / f64::from(p * 2 + 1);
        let d6 = TWO_PI / f64::from(n * 2 + 1);
        let cur = &unit.pcm;
        if frame_len > 0 {
            let mut i = 1i32;
            while i <= frame_len {
                let i11 = i - 1;
                let v = if i11 < p {
                    let w = (1.0 - (f64::from(i) * d5).cos()) * 0.5;
                    (f64::from(cur.get(addr + i11 as usize).copied().unwrap_or(0)) * w) as i32
                } else {
                    let arg = f64::from((n + 1 - p) + i11) * d6;
                    let w = (1.0 - arg.cos()) * 0.5;
                    (f64::from(cur.get(addr + i11 as usize).copied().unwrap_or(0)) * w) as i32
                };
                window[i as usize] = v as i16;
                i += 1;
            }
        }
        let gap = p_prime - p;
        let pos = gap + n_process;
        if gap < 0 {
            if n_process != 0 {
                for i in 0..frame_len {
                    let idx = pos + i;
                    if idx < 0 {
                        continue;
                    }
                    let v = if idx < n_upm_pitch_length && (idx as usize) < out.len() {
                        clip_psola(i32::from(out[idx as usize]) + i32::from(window[i as usize]))
                    } else {
                        window[i as usize]
                    };
                    if idx as usize >= out.len() {
                        out.resize(idx as usize + 1, 0);
                    }
                    out[idx as usize] = v;
                }
                n_upm_pitch_length = n + pos + p;
                n_process += p_prime;
            } else if frame_len > 0 {
                for i in 0..frame_len {
                    let idx = pos + i;
                    if idx < 0 {
                        continue;
                    }
                    let v = if i < i32::from(res.w_next_pitch) - gap {
                        let tv = res.tail_at(i32::from(res.w_prev_pitch) + i + gap);
                        clip_psola(i32::from(tv) + i32::from(window[i as usize]))
                    } else {
                        window[i as usize]
                    };
                    if idx as usize >= out.len() {
                        out.resize(idx as usize + 1, 0);
                    }
                    out[idx as usize] = v;
                }
                n_upm_pitch_length = n + pos + p;
                n_process = p_prime;
            }
        } else {
            if n_process == 0 {
                let prev_p = i32::from(res.w_prev_pitch);
                if prev_p == 0 {
                    if gap > 0 {
                        out.resize(gap as usize, 0);
                    }
                    for i in 0..frame_len {
                        let idx = pos + i;
                        if idx as usize >= out.len() {
                            out.resize(idx as usize + 1, 0);
                        }
                        out[idx as usize] = window[i as usize];
                    }
                } else {
                    if gap > 0 {
                        for i in 0..gap {
                            let idx = i as usize;
                            if idx >= out.len() {
                                out.resize(idx + 1, 0);
                            }
                            out[idx] = res.tail_at(prev_p + i);
                        }
                    }
                    for i in 0..frame_len {
                        let idx = pos + i;
                        let v = if i < i32::from(res.w_next_pitch) - gap {
                            let tv = res.tail_at(prev_p + gap + i);
                            clip_psola(i32::from(tv) + i32::from(window[i as usize]))
                        } else {
                            window[i as usize]
                        };
                        if idx as usize >= out.len() {
                            out.resize(idx as usize + 1, 0);
                        }
                        out[idx as usize] = v;
                    }
                }
            } else if frame_len > 0 {
                for i in 0..frame_len {
                    let idx = pos + i;
                    if idx < 0 {
                        continue;
                    }
                    let v = if idx < n_upm_pitch_length && (idx as usize) < out.len() {
                        clip_psola(i32::from(out[idx as usize]) + i32::from(window[i as usize]))
                    } else {
                        window[i as usize]
                    };
                    if idx as usize >= out.len() {
                        out.resize(idx as usize + 1, 0);
                    }
                    out[idx as usize] = v;
                }
            }
            n_process += p_prime;
            n_upm_pitch_length = n + pos + p;
        }
    }
}

#[expect(clippy::float_cmp, reason = "C port: exact default-parameter check")]
fn arrange_process(res: &mut ResultInfo, units: &[UnitEntry], marks: &[UpmMark], n_index: usize) {
    if res.params.pitch == 1.0
        && !res.params.pitch.is_nan()
        && res.params.speed == 1.0
        && !res.params.speed.is_nan()
    {
        fast_path(res, units, marks, n_index);
        return;
    }
    let (out, _tail_save) = slow_path(res, units, marks, n_index);
    res.buf.extend_from_slice(&out);
}

#[allow(clippy::manual_clamp)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
fn volume_synth_sample(res: &mut ResultInfo) {
    let vol = res.params.volume;
    for s in &mut res.buf {
        let mut v = (f64::from(*s) * f64::from(vol)) as i32;
        if v < -VOLUME_CLIP {
            v = -VOLUME_CLIP;
        }
        if v > VOLUME_CLIP {
            v = VOLUME_CLIP;
        }
        *s = v as i16;
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
/// Synthesizes the waveform for a phrase.
///
/// # Errors
///
/// Returns an error if the input is invalid.
pub fn synthesize_wave(
    ctx: &crate::SynthDb,
    phrase: &Phrase,
    selection: &Selection,
    params: SynthParams,
) -> SynthResult<Vec<i16>> {
    let mut ave_length = vec![0u16; phrase.letter_num()];
    rest_length_set(ctx, phrase, selection, &mut ave_length)?;
    let units = all_phone_info(ctx, phrase, selection, &ave_length)?;
    let mut res = ResultInfo::new(params);
    for (i, unit) in units.iter().enumerate() {
        if unit.ch_rest_flag == 1 {
            let zlen = (f32::from(unit.w_pcm_size) * params.speed) as i32;
            let zlen = zlen.max(0) as usize;
            res.buf.resize(res.buf.len() + zlen, 0);
            res.w_prev_pitch = 0;
            continue;
        }
        let marks = upm_marks(unit);
        arrange_process(&mut res, &units, &marks, i);
    }
    volume_synth_sample(&mut res);
    Ok(res.buf)
}
