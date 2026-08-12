use crate::tables::{
    GSCH_BREAK_O_JONG_TBL, GSCH_DEFAULT_JONG_TBL, GSSW_G_PRON_TBL, SSCH_GENERAL_TRAN_TBL1,
};

#[derive(Debug, Clone, Default)]
pub struct WordCvc {
    pub cvc: Vec<u8>,
    pub ty: Vec<u8>,
    pub tag: Vec<u8>,
    pub mpos: Vec<u8>,
    pub irr_flag: Vec<u8>,
    pub native_flag: Vec<u8>,
    pub native_words: Vec<Vec<u8>>,
}

impl WordCvc {
    #[must_use]
    pub const fn syllable_count(&self) -> usize {
        self.cvc.len()
    }

    #[must_use]
    pub fn is_h(&self, pos: usize) -> bool {
        self.ty.get(pos).copied() == Some(b'H')
    }

    #[must_use]
    pub fn is_x(&self, pos: usize) -> bool {
        self.ty.get(pos).copied() == Some(b'X')
    }

    #[must_use]
    pub fn syllable_starts(&self) -> Vec<usize> {
        let mut v = Vec::new();
        let mut i = 0;
        while i < self.cvc.len() {
            v.push(i);
            i += if self.is_h(i) { 3 } else { 1 };
        }
        v
    }

    #[must_use]
    pub fn tag_at(&self, pos: usize) -> u8 {
        self.tag.get(pos).copied().unwrap_or(0)
    }

    #[must_use]
    pub fn mpos_at(&self, pos: usize) -> u8 {
        self.mpos.get(pos).copied().unwrap_or(0)
    }
}

#[must_use]
pub const fn is_k_root_pumsa(tag: u8) -> bool {
    tag.wrapping_sub(0x30) < 0x1C
}

#[must_use]
pub const fn is_to(tag: u8) -> bool {
    tag.wrapping_add(0xAC) < 0x15
}

#[must_use]
pub const fn is_k_voice_yong_yon(tag: u8) -> bool {
    matches!(tag, b'B' | b'@' | b'D' | b'C' | b'f')
}

#[must_use]
pub const fn is_k_voice_yong_yon_to(tag: u8) -> bool {
    tag.wrapping_add(0x9C) < 6 || tag.wrapping_add(0xA2) < 5
}

#[must_use]
pub const fn is_symbol_pumsa(tag: u8) -> bool {
    tag.wrapping_add(0xB4) < 7
}

const fn is_eomi(tag: u8) -> bool {
    if tag.wrapping_add(0xAC) > 0x10
        && !matches!(tag, b'g' | b'i' | b'h' | b'@' | b'm' | b'D' | b'C')
    {
        tag == b'B'
    } else {
        true
    }
}

#[must_use]
pub const fn is_sun_cho(cho: u8) -> bool {
    matches!(cho, 5 | 2 | 11 | 9 | 14)
}

#[must_use]
pub const fn is_mack_him_jong(jong: u8) -> bool {
    if jong.wrapping_sub(2) > 2 && jong != 8 && jong.wrapping_sub(0x13) > 3 {
        jong.wrapping_sub(0x18) < 5
    } else {
        true
    }
}

const fn is_break_jong(jong: u8) -> bool {
    matches!(jong, 10 | 4 | 21 | 20 | 26 | 25 | 28 | 27)
}

const fn is_break_jung(jung: u8) -> bool {
    matches!(jung, 7 | 3 | 20 | 13 | 18 | 4 | 29)
}

const fn is_nasal(cho: u8) -> bool {
    if cho != 7 && cho != 4 {
        cho == 13 || cho == 8
    } else {
        true
    }
}

pub fn apply_pron_trans_rule(cvc: &mut [u8]) {
    let n = cvc.len();
    if n < 3 {
        return;
    }
    let mut i = 3;
    while i < n {
        let jong = cvc[i - 1] as usize;
        let cho = cvc[i] as usize;
        if jong < 32 && cho < 22 {
            let w = SSCH_GENERAL_TRAN_TBL1[jong][cho];
            if w != 0xFFFF {
                cvc[i - 1] = (w >> 8) as u8;
                cvc[i] = (w & 0xFF) as u8;
            }
        }
        i += 3;
    }
    let mut i = 3;
    while i <= n {
        let jong = cvc[i - 1] as usize;
        if jong < 32 {
            cvc[i - 1] = GSCH_DEFAULT_JONG_TBL[jong];
        }
        i += 3;
    }
}

pub fn pronun_intra_word(p: &mut WordCvc) {
    let n = p.cvc.len();
    let mut i = 0;
    while i < n {
        let jp = i + 2;
        if p.is_h(i) && jp + 2 < n && p.is_h(i + 3) {
            let applied = is_irr_word(p, jp)
                || pron_break_o(p, jp)
                || pronun_except_inter_motph(p, jp)
                || p.cvc[jp] == 1;
            if !applied {
                pronun_revise_jong(p, jp);
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    let mut i = 0;
    while i < n {
        if !p.is_x(i) && i + 2 < n {
            let cho = p.cvc[i];
            if p.cvc[i + 1] == 12 && cho != 13 {
                p.cvc[i + 1] = 10;
            }
            if p.cvc[i + 1] == 28 {
                pron_ui(p, i + 1, n);
            }
            let j = p.cvc[i + 2] as usize;
            if j < 32 {
                p.cvc[i + 2] = GSCH_DEFAULT_JONG_TBL[j];
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    pronun_yu_nyu(p);
}

fn is_irr_word(p: &WordCvc, jong_pos: usize) -> bool {
    let mn = p.mpos_at(jong_pos + 1) as usize;
    p.irr_flag.get(mn).copied().unwrap_or(0) != 0 && p.mpos_at(jong_pos + 1) == p.mpos_at(jong_pos)
}

fn pron_break_o(p: &mut WordCvc, jong_pos: usize) -> bool {
    let jong = p.cvc[jong_pos];
    let cho_next = p.cvc.get(jong_pos + 1).copied().unwrap_or(0);
    let jung_next = p.cvc.get(jong_pos + 2).copied().unwrap_or(0);
    if is_break_jong(jong)
        && cho_next == 13
        && is_break_jung(jung_next)
        && i32::from(p.mpos_at(jong_pos)) == i32::from(p.mpos_at(jong_pos + 2)) - 1
        && (is_k_root_pumsa(p.tag_at(jong_pos + 2))
            || p.tag_at(jong_pos + 2) == b'@'
            || p.tag_at(jong_pos + 2) == b'e')
        && is_k_root_pumsa(p.tag_at(jong_pos))
    {
        if let Some(c) = p.cvc.get_mut(jong_pos + 1) {
            *c = GSCH_BREAK_O_JONG_TBL[jong as usize];
        }
        p.cvc[jong_pos] = 1;
        return true;
    }
    false
}

#[allow(unused_assignments)]
#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
#[expect(
    clippy::useless_let_if_seq,
    reason = "C port: imperative assignment structure kept"
)]
fn pronun_except_inter_motph(p: &mut WordCvc, jong_pos: usize) -> bool {
    let cho_pos = jong_pos + 1;
    let jung_pos = jong_pos + 2;
    let jn_pos = jong_pos + 3;
    let jong = p.cvc[jong_pos];
    let cho = p.cvc[cho_pos];
    let jung = p.cvc[jung_pos];
    let jong_next = p.cvc.get(jn_pos).copied().unwrap_or(0);
    let tag_prev = p.tag_at(jong_pos);
    let tag_next = p.tag_at(cho_pos);
    let mpos_prev = p.mpos_at(jong_pos);
    let mpos_next = p.mpos_at(cho_pos);

    if !is_sun_cho(cho) {
        return pronun_vowel_inflec_rule(p, jong_pos);
    }

    let mpos_diff = mpos_prev != mpos_next;
    let native = p.native_flag.get(mpos_next as usize).copied().unwrap_or(0) != 0;
    if native && mpos_diff {
        if tag_prev.wrapping_sub(0x3A) < 3 {
            return pronun_vowel_inflec_rule(p, jong_pos);
        }
        if !is_k_voice_yong_yon(tag_next) {
            let nw = p
                .native_words
                .get(mpos_next as usize)
                .cloned()
                .unwrap_or_default();
            if jong != 1 && nw.contains(&jong) {
                p.cvc[cho_pos] = p.cvc[cho_pos].wrapping_add(1);
                return true;
            }
        }
    }

    if search_rule_dic(p, jong_pos) {
        p.cvc[cho_pos] = p.cvc[cho_pos].wrapping_add(1);
        return true;
    }
    if tag_next == b'Y' && cho == 11 {
        p.cvc[cho_pos] = 12;
        return true;
    }
    if tag_next == b'6' && jong == 9 && tag_prev != b'6' {
        let ok = if cho == 9 {
            (jung == 7 || jung == 20) && jong_next == 5
        } else if cho == 2 {
            match jung {
                4 | 29 => jong_next == 1,
                20 => jong_next == 5,
                27 => jong_next == 19 || jong_next == 1,
                _ => false,
            }
        } else {
            false
        };
        if !ok {
            p.cvc[cho_pos] = cho.wrapping_add(1);
            return true;
        }
    }
    let mut c_cho = cho;
    let mut c_jong = jong;
    let mut chain: Option<Chain> = None;
    if is_k_root_pumsa(tag_next) && cho == 9 && jung == 29 && jong_next == 25 && tag_prev != b'_' {
        if jong != 5 {
            p.cvc[cho_pos] = 10;
            return true;
        }
        c_cho = 9;
        c_jong = 5;
        chain = Some(Chain::E398);
    } else {
        if jong == 1 {
            return false;
        }
        c_cho = cho;
        c_jong = jong;
        if c_cho == 2 {
            if jung == 21 {
                if tag_next == b'4' && jong_next == 5 {
                    p.cvc[cho_pos] = c_cho.wrapping_add(1);
                    return true;
                }
                chain = Some(Chain::E398);
            } else if jung == 7 && tag_next == b'7' && jong_next == 21 {
                if tag_prev == b'_' {
                    chain = Some(Chain::E3ac);
                } else {
                    if jong != 5 {
                        p.cvc[cho_pos] = c_cho.wrapping_add(1);
                        return true;
                    }
                    chain = Some(Chain::E398);
                }
            } else {
                chain = Some(Chain::E398);
            }
        } else if c_cho == 14 {
            if (tag_next == b'4' && jung == 10) || (jung == 7 && tag_next == b'>' && jong_next == 2)
            {
                p.cvc[cho_pos] = c_cho.wrapping_add(1);
                return true;
            }
            chain = Some(Chain::E398);
        } else if c_cho != 5 {
            if c_cho == 11 && jung == 20 {
                match e5e4_entry(
                    p, jong_pos, c_cho, jong, tag_prev, tag_next, jung, jong_next,
                ) {
                    E5Result::Applied => return true,
                    E5Result::E398 => chain = Some(Chain::E398),
                    E5Result::E3d2 => chain = Some(Chain::E3d2),
                }
            } else {
                chain = Some(Chain::E398);
            }
        } else {
            if jung == 16 {
                match e5e4_entry(
                    p, jong_pos, c_cho, jong, tag_prev, tag_next, jung, jong_next,
                ) {
                    E5Result::Applied => return true,
                    E5Result::E398 => chain = Some(Chain::E398),
                    E5Result::E3d2 => chain = Some(Chain::E3d2),
                }
            } else {
                chain = Some(Chain::E398);
            }
        }
    }

    let mut local_3d = false;
    let mut step = match chain {
        Some(Chain::E398) => {
            if tag_prev == b'0' {
                if tag_next.wrapping_sub(0x30) > 1 {
                    Chain::E3c8
                } else {
                    Chain::E550
                }
            } else if tag_prev == b'>' {
                if tag_next.wrapping_sub(0x30) > 2 {
                    Chain::E3c8
                } else {
                    Chain::E550
                }
            } else {
                Chain::E3ac
            }
        }
        Some(Chain::E3ac) => Chain::E3ac,
        Some(Chain::E3d2) => Chain::E3d2,
        None => Chain::Common,
        _ => unreachable!("branch only produces E398/E3ac/E3d2"),
    };
    loop {
        match step {
            Chain::Common => break,
            Chain::E3ac => {
                step = if (tag_prev != b'2' && tag_prev != b'9') || tag_next != b'0' {
                    Chain::E3c8
                } else {
                    Chain::E550
                };
            }
            Chain::E3c8 => {
                local_3d = tag_next == b'0';
                step = if tag_next != b'e' || tag_prev != b'7' {
                    Chain::E3d2
                } else {
                    Chain::E550
                };
            }
            Chain::E3d2 => {
                if tag_prev.wrapping_add(0xB8) < 2 {
                    if local_3d
                        || tag_next == b'6'
                        || tag_next == b'7'
                        || tag_next == b'H'
                        || tag_next == b'1'
                    {
                        step = Chain::E550;
                    } else {
                        step = Chain::Common;
                    }
                } else if tag_prev == b'1' && tag_next.wrapping_sub(0x30) < 2 {
                    step = Chain::E550;
                } else {
                    step = Chain::Common;
                }
            }
            Chain::E550 => {
                if mpos_diff {
                    return pronun_vowel_inflec_rule(p, jong_pos);
                }
                step = Chain::Common;
            }
            Chain::E398 => unreachable!(),
        }
    }

    if (c_cho == 5 || c_cho == 2 || c_cho == 11 || c_cho == 14)
        && (c_jong == 5
            || c_jong == 6
            || c_jong == 10
            || c_jong == 11
            || c_jong == 12
            || c_jong == 14)
    {
        if is_k_voice_yong_yon(tag_prev) && is_to(tag_next) {
            match c_jong {
                6 => p.cvc[jong_pos] = 5,
                10 | 14 => p.cvc[jong_pos] = 9,
                11 => p.cvc[jong_pos] = 17,
                12 => {
                    p.cvc[jong_pos] = 9;
                    if p.cvc[jong_pos.saturating_sub(2)] == 9
                        && p.cvc[jong_pos.saturating_sub(1)] == 3
                    {
                        p.cvc[jong_pos] = 19;
                    }
                }
                _ => {}
            }
            p.cvc[cho_pos] = p.cvc[cho_pos].wrapping_add(1);
            return true;
        }
        if c_jong == 10 {
            p.cvc[jong_pos] = 2;
            p.cvc[cho_pos] = p.cvc[cho_pos].wrapping_add(1);
            return true;
        }
    }
    if is_sun_cho(c_cho) && p.cvc[jong_pos] == 9 && (c_cho != 5 || jung != 13 || jong_next != 1) {
        if tag_prev == b'I' {
            p.cvc[cho_pos] = c_cho.wrapping_add(1);
            return true;
        }
        if is_k_voice_yong_yon_to(tag_prev) {
            p.cvc[cho_pos] = p.cvc[cho_pos].wrapping_add(1);
            return true;
        }
        if tag_next == b'<' && tag_prev == b'<' && (c_cho == 11 || c_cho == 5 || c_cho == 14) {
            p.cvc[cho_pos] = c_cho.wrapping_add(1);
            return true;
        }
    }
    pronun_vowel_inflec_rule(p, jong_pos)
}

#[derive(Clone, Copy, PartialEq)]
enum E5Result {
    Applied,
    E398,
    E3d2,
}

#[allow(clippy::too_many_arguments)]
#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
fn e5e4_entry(
    p: &mut WordCvc,
    jong_pos: usize,
    c_cho: u8,
    jong: u8,
    tag_prev: u8,
    tag_next: u8,
    jung: u8,
    jong_next: u8,
) -> E5Result {
    let cho_pos = jong_pos + 1;
    if tag_next != b'7' || jong_next != 1 {
        return E5Result::E398;
    }
    if tag_prev != b'7' {
        if jong == 9 {
            p.cvc[cho_pos] = c_cho.wrapping_add(1);
            return E5Result::Applied;
        }
        return E5Result::E398;
    }
    let _ = jung;
    E5Result::E3d2
}

#[derive(Clone, Copy, PartialEq)]
enum Chain {
    E398,
    E3ac,
    E3c8,
    E3d2,
    E550,
    Common,
}

#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
fn pronun_vowel_inflec_rule(p: &mut WordCvc, jong_pos: usize) -> bool {
    let jong = p.cvc[jong_pos];
    let cho = p.cvc[jong_pos + 1];
    let jung = p.cvc[jong_pos + 2];
    if p.tag_at(jong_pos + 1) == b'0'
        && p.tag_at(jong_pos) == b'0'
        && p.mpos_at(jong_pos) != p.mpos_at(jong_pos + 1)
    {
        return false;
    }
    if cho == 20 || cho == 13 {
        if (jung == 29 || jung == 11) && jong == 27 {
            p.cvc[jong_pos] = 1;
            p.cvc[jong_pos + 1] = 16;
            return true;
        }
        if cho != 20 {
            if cho == 13 && jung == 29 {
                if jong == 14 {
                    p.cvc[jong_pos] = 9;
                    p.cvc[jong_pos + 1] = 16;
                    return true;
                }
                if jong == 8 {
                    p.cvc[jong_pos] = 1;
                    p.cvc[jong_pos + 1] = 14;
                    return true;
                }
            }
        } else if (jung == 29 || jung == 11) && jong == 8 {
            p.cvc[jong_pos] = 1;
            p.cvc[jong_pos + 1] = 16;
            return true;
        }
    }
    false
}

#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
#[expect(clippy::too_many_lines, reason = "faithful C port of a large function")]
fn pronun_revise_jong(p: &mut WordCvc, jong_pos: usize) {
    let cho_pos = jong_pos + 1;
    let jung_pos = jong_pos + 2;
    let jn_pos = jong_pos + 3;
    let jong = p.cvc[jong_pos];
    let cho = p.cvc[cho_pos];
    let jung = p.cvc[jung_pos];
    let jong_next = p.cvc.get(jn_pos).copied().unwrap_or(0);
    let tag_prev = p.tag_at(jong_pos);
    let tag_next = p.tag_at(cho_pos);

    let w = if jong < 32 && cho < 22 {
        GSSW_G_PRON_TBL[jong as usize][cho as usize]
    } else {
        0xFFFF
    };

    if ((cho == 13 && jung == 18) || (cho == 20 && jung == 20))
        && tag_next == b'7'
        && jong_next == 1
    {
        return;
    }

    let tag_hi = tag_prev.wrapping_add(0xB8) < 2;

    let mut step = if tag_hi {
        if jong == 12 && cho == 11 {
            if jung == 3 && jong_next == 9 {
                RevStep::De58
            } else {
                RevStep::Dd38
            }
        } else if cho == 20 {
            if jung == 13 && jong_next == 1 {
                RevStep::De58
            } else {
                RevStep::Dd18
            }
        } else if cho == 8 {
            if jung == 11 && jong_next == 23 {
                RevStep::De58
            } else {
                RevStep::Dd18
            }
        } else if cho == 13 {
            if (jung == 29 || jung == 21) && jong_next == 9 {
                RevStep::De58
            } else {
                RevStep::Dd18
            }
        } else if cho == 4 && jung == 11 && jong_next == 5 {
            RevStep::De58
        } else {
            RevStep::Dd18
        }
    } else if tag_prev == b'0' {
        if tag_next.wrapping_sub(0x30) > 1 {
            RevStep::Dc43
        } else {
            RevStep::Joined
        }
    } else if tag_prev == b'>' {
        if tag_next.wrapping_sub(0x30) > 2 {
            RevStep::Dc43
        } else {
            RevStep::Joined
        }
    } else {
        RevStep::Dd38
    };

    loop {
        match step {
            RevStep::De58 => {
                if tag_next == b'6' && w != 0xFFFF {
                    apply_g_tbl(p, jong_pos, w);
                    return;
                }
                step = RevStep::Dd18;
            }
            RevStep::Dd18 => {
                if jong == 19 {
                    if cho == 7 {
                        if w != 0xFFFF {
                            apply_g_tbl(p, jong_pos, w);
                            return;
                        }
                    } else if cho == 13 && jung == 26 && jong_next == 2 {
                        p.cvc[jong_pos] = 17;
                        p.cvc[cho_pos] = 4;
                        return;
                    }
                }
                step = RevStep::Dd38;
            }
            RevStep::Dd38 => {
                let hit = if tag_prev == b'_' || tag_prev == b'9' {
                    Some(if tag_next == b'0' {
                        RevStep::Joined
                    } else {
                        RevStep::Dc43
                    })
                } else if tag_next == b'0' {
                    if tag_prev == b'2' {
                        Some(RevStep::Joined)
                    } else {
                        None
                    }
                } else if tag_next == b'e' && tag_prev == b'7' {
                    Some(RevStep::Joined)
                } else {
                    None
                };
                step = hit.unwrap_or_else(|| {
                    if tag_hi {
                        if tag_next == b'H'
                            || tag_next == b'6'
                            || tag_next == b'1'
                            || tag_next == b'7'
                        {
                            RevStep::Joined
                        } else {
                            RevStep::Deb0
                        }
                    } else if tag_prev == b'1' {
                        if tag_next.wrapping_sub(0x30) < 2 || tag_next == b'C' {
                            RevStep::Joined
                        } else {
                            RevStep::Final
                        }
                    } else {
                        RevStep::Dd92
                    }
                });
            }
            RevStep::Dc43 => {
                let hit = if tag_next == b'e' && tag_prev == b'7' {
                    Some(RevStep::Joined)
                } else {
                    None
                };
                step = hit.unwrap_or_else(|| {
                    if tag_hi {
                        if tag_next == b'H'
                            || tag_next == b'6'
                            || tag_next == b'1'
                            || tag_next == b'7'
                        {
                            RevStep::Joined
                        } else {
                            RevStep::Deb0
                        }
                    } else if tag_prev == b'1' {
                        if tag_next.wrapping_sub(0x30) < 2 || tag_next == b'C' {
                            RevStep::Joined
                        } else {
                            RevStep::Final
                        }
                    } else {
                        RevStep::Dd92
                    }
                });
            }
            RevStep::Joined => {
                if p.mpos_at(jong_pos) != p.mpos_at(cho_pos) {
                    return;
                }
                step = if tag_hi { RevStep::Deb0 } else { RevStep::Dd92 };
            }
            RevStep::Deb0 => {
                if tag_next.wrapping_add(0xB8) < 2 && jong != 19 {
                    return;
                }
                step = RevStep::Final;
            }
            RevStep::Dd92 => {
                if tag_prev == b'4' && !is_to(cho) && jong != 9 {
                    return;
                }
                step = RevStep::Final;
            }
            RevStep::Final => {
                if is_pronun_revise_jong(p, jong_pos) {
                    return;
                }
                if jong >= 0x80 || cho >= 0x80 {
                    return;
                }
                if w == 0xFFFF {
                    return;
                }
                apply_g_tbl(p, jong_pos, w);
                return;
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum RevStep {
    De58,
    Dd18,
    Dd38,
    Dc43,
    Joined,
    Deb0,
    Dd92,
    Final,
}

fn is_pronun_revise_jong(p: &WordCvc, jong_pos: usize) -> bool {
    let cho_pos = jong_pos + 1;
    if p.mpos_at(cho_pos) == p.mpos_at(jong_pos) {
        return false;
    }
    let tag_next = p.tag_at(cho_pos);
    let tag_prev = p.tag_at(jong_pos);
    if is_eomi(tag_next) {
        return false;
    }
    if tag_prev == b'F' {
        return false;
    }
    if tag_next != b'H' && tag_prev == b'H' {
        return false;
    }
    if tag_next == b'7' && tag_prev == b'0' {
        return false;
    }
    if is_eomi(tag_prev) && !is_eomi(tag_next) {
        return false;
    }
    if is_k_root_pumsa(tag_prev) && tag_next == b'e' {
        return false;
    }
    let cho = p.cvc[cho_pos];
    let jong = p.cvc[jong_pos];
    if is_nasal(cho) && is_mack_him_jong(jong) {
        return false;
    }
    if (tag_next == b'>' || tag_next == b'7') && cho == 20 {
        return false;
    }
    if tag_next == b']' && cho == 9 {
        return false;
    }
    true
}

fn apply_g_tbl(p: &mut WordCvc, jong_pos: usize, w: u16) {
    p.cvc[jong_pos] = (w >> 8) as u8;
    p.cvc[jong_pos + 1] = (w & 0xFF) as u8;
}

fn pron_ui(p: &mut WordCvc, jung_pos: usize, _n: usize) {
    if p.cvc[jung_pos + 1] == 1 && p.tag_at(jung_pos) == b'V' {
        p.cvc[jung_pos] = 10;
        return;
    }
    if p.cvc[jung_pos - 1] == 13 {
        let m = p.mpos_at(jung_pos);
        let mut i = 0usize;
        loop {
            i += 1;
            if i >= jung_pos {
                break;
            }
            if p.mpos_at(jung_pos - i) != m {
                break;
            }
        }
        if i < 3 {
            return;
        }
    }
    p.cvc[jung_pos] = 29;
}

#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
fn pronun_yu_nyu(p: &mut WordCvc) {
    let n = p.cvc.len();
    let mut i = 0;
    while i < n {
        if !p.is_h(i) {
            i += 1;
            continue;
        }
        let jong_pos = i + 2;
        let cho_next = i + 3;
        if n <= jong_pos + 3 || !p.is_h(cho_next) {
            i += 3;
            continue;
        }
        let jong = p.cvc[jong_pos];
        let cho = p.cvc[cho_next];
        let jung_next = p.cvc[cho_next + 1];
        let jong_next = p.cvc[cho_next + 2];
        let tag_prev = p.tag_at(jong_pos);
        let tag_next = p.tag_at(cho_next);
        if cho == 7 && (jung_next == 26 || jung_next == 11) && jong_next == 9 && jong == 5 {
            p.cvc[cho_next] = 13;
        }
        if (jong == 17 || jong == 23)
            && p.cvc[cho_next] == 7
            && is_k_voice_yong_yon(tag_prev)
            && p.mpos_at(jong_pos) == p.mpos_at(cho_next)
            && tag_prev == tag_next
        {
            p.cvc[cho_next] = 4;
        }
        i = cho_next;
        if n <= i {
            return;
        }
    }
}

pub fn pronun_between_word(front: &mut WordCvc, back: &mut WordCvc) {
    if back.cvc.is_empty() || !back.is_h(0) || back.cvc.len() < 3 {
        return;
    }
    let fn_ = front.cvc.len();
    if fn_ == 0 || !front.is_h(fn_ - 1) {
        return;
    }
    if back.cvc.len() != 3 {
        return;
    }
    if !is_sun_cho(back.cvc[0]) {
        return;
    }
    let bcode = front.cvc[fn_ - 1];
    if (bcode == 9 || bcode == 2) && back.cvc == [5, 4, 1] {
        back.cvc[0] = 6;
    }
    if is_mack_him_jong(bcode) && back.cvc[0] == 5 && back.cvc[1] == 27 && back.cvc[2] == 23 {
        back.cvc[0] = 6;
    }
}

pub fn merge_solo_jong(p: &mut WordCvc) {
    let mut i = 0;
    let mut prev_h_start: Option<usize> = None;
    while i < p.cvc.len() {
        if p.is_h(i) {
            if p.cvc[i] == 1 && p.cvc.get(i + 1) == Some(&1) {
                if let Some(ps) = prev_h_start {
                    let jong_pos = ps + 2;
                    if p.cvc[jong_pos] == 1 {
                        p.cvc[jong_pos] = p.cvc[i + 2];
                    }
                    p.cvc.drain(i..i + 3);
                    p.ty.drain(i..i + 3);
                    p.tag.drain(i..i + 3);
                    p.mpos.drain(i..i + 3);
                    continue;
                }
            } else {
                prev_h_start = Some(i);
            }
            i += 3;
        } else {
            prev_h_start = None;
            i += 1;
        }
    }
}

/// Searches the pronunciation rule dictionary.
///
/// # Panics
///
/// Panics if the dictionary data is inconsistent.
#[expect(
    clippy::significant_drop_tightening,
    reason = "lock guard scope is intentional"
)]
pub fn search_rule_dic(p: &WordCvc, jong_pos: usize) -> bool {
    let rules = crate::RULE_DICT.lock().expect("RULE_DICT lock poisoned");
    let Some(rules) = rules.as_ref() else {
        return false;
    };
    for rule in &rules.rules {
        if compare_rule(rule, p, jong_pos) {
            return true;
        }
    }
    false
}

fn compare_rule(rule: &ktts_dict::pronrule::PronRule, p: &WordCvc, jong_pos: usize) -> bool {
    let raw = rule_raw(rule);
    let sch_cho_cond = &raw[..12];
    let ch_cho_tag = raw[12];
    let ch_jong_flag = raw[13];
    let ch_jong_key = raw[14];
    let ch_jong_tag_flag = raw[15];
    let ch_jong_tag_key = raw[16];
    let ch_jong_count_flag = raw[17];
    let ch_jong_count_key = raw[18];
    let ch_tag_with_jong_flag = raw[19];

    if p.mpos_at(jong_pos) == p.mpos_at(jong_pos + 1) {
        return false;
    }
    let n = p.cvc.len();
    let m = p.mpos_at(jong_pos + 1);
    let mut sch_cho: Vec<u8> = Vec::with_capacity(12);
    let mut pos = jong_pos + 1;
    while sch_cho.len() < 12 && pos < n && p.mpos_at(pos) == m {
        sch_cho.push(p.cvc[pos]);
        pos += 1;
    }
    if sch_cho_cond[0] != b'*' && !cstr_eq(sch_cho_cond, &sch_cho) {
        return false;
    }
    let tag_next = p.tag_at(jong_pos + 1);
    let tag_here = p.tag_at(jong_pos);
    if ch_cho_tag != tag_next {
        return false;
    }
    let jong = p.cvc[jong_pos];
    match ch_jong_flag {
        1 => {
            if ch_jong_key != jong {
                return false;
            }
        }
        3 => {}
        _ => {
            if ch_jong_key == jong {
                return false;
            }
        }
    }
    match ch_jong_tag_flag {
        1 => {
            if ch_jong_tag_key != tag_here {
                return false;
            }
        }
        3 => {}
        _ => {
            if ch_jong_tag_key == tag_here {
                return false;
            }
        }
    }
    match ch_tag_with_jong_flag {
        1 => {
            if ch_cho_tag != tag_here {
                return false;
            }
        }
        3 => {}
        _ => {
            if ch_cho_tag == tag_here {
                return false;
            }
        }
    }
    if ch_jong_count_flag != 3 {
        let cnt = jong_syllable_count(p, jong_pos);
        if ch_jong_count_flag == 2 {
            if cnt == ch_jong_count_key {
                return false;
            }
        } else if cnt != ch_jong_count_key {
            return false;
        }
    }
    true
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn jong_syllable_count(p: &WordCvc, jong_pos: usize) -> u8 {
    let m = p.mpos_at(jong_pos);
    let mut k: usize = 0;
    loop {
        let idx = jong_pos as isize - (k as isize + 1);
        if idx < 0 || p.mpos_at(idx as usize) != m {
            k += 1;
            break;
        }
        k += 1;
    }
    (k / 3) as u8 + b'0'
}

fn cstr_eq(a: &[u8], b: &[u8]) -> bool {
    let n = a.len().min(b.len());
    for i in 0..n {
        if a[i] != b[i] {
            return false;
        }
        if a[i] == 0 {
            return true;
        }
    }
    a.get(n).copied().unwrap_or(0) == b.get(n).copied().unwrap_or(0)
}

fn rule_raw(rule: &ktts_dict::pronrule::PronRule) -> [u8; 20] {
    let mut raw = [0u8; 20];
    raw[..12].copy_from_slice(&rule.cond);
    raw[12..].copy_from_slice(&rule.apply);
    raw
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wc(cvc: &[u8], tags: &[u8]) -> WordCvc {
        let ty: Vec<u8> = vec![b'H'; cvc.len()];
        let mpos: Vec<u8> = vec![0; cvc.len()];
        WordCvc {
            cvc: cvc.to_vec(),
            ty,
            tag: tags.to_vec(),
            mpos,
            ..Default::default()
        }
    }

    fn wc2(cvc: &[u8], tags: &[u8]) -> WordCvc {
        let mut w = wc(cvc, tags);
        w.mpos = (0..cvc.len())
            .map(|i| u8::try_from(i / 3).expect("morph index fits u8"))
            .collect();
        w
    }

    #[test]
    fn apply_pron_trans_rule_hand_computed() {
        let mut c = vec![2, 20, 2, 8, 20, 9];
        apply_pron_trans_rule(&mut c);
        assert_eq!(c, vec![2, 20, 23, 8, 20, 9], "국물→궁물 (ㄱㅁ→ㅇㅁ)");

        let mut c = vec![11, 3, 5, 7, 3, 1];
        apply_pron_trans_rule(&mut c);
        assert_eq!(c, vec![11, 3, 9, 7, 3, 1], "신라→실라 (ㄴㄹ→ㄹㄹ)");

        let mut c = vec![13, 3, 5, 13, 3, 1];
        apply_pron_trans_rule(&mut c);
        assert_eq!(c, vec![13, 3, 1, 4, 3, 1], "안아→아나 (ㄴ 연음)");

        let mut c = vec![9, 3, 27];
        apply_pron_trans_rule(&mut c);
        assert_eq!(c, vec![9, 3, 8], "밭→받 (ㅌ→ㄷ 대표음)");

        let mut c = vec![5, 13, 29, 2, 13, 1];
        apply_pron_trans_rule(&mut c);
        assert_eq!(c, vec![5, 13, 2, 17, 13, 1], "놓고→놕코 (ㅎ+ㄱ→ㄱㅋ)");
    }

    #[test]
    fn default_jong_representative() {
        let mut c = vec![5, 3, 10];
        apply_pron_trans_rule(&mut c);
        assert_eq!(c, vec![5, 3, 2], "닭→닥 (ㄺ→ㄱ)");
        let mut c = vec![2, 3, 20];
        apply_pron_trans_rule(&mut c);
        assert_eq!(c, vec![2, 3, 19], "값→갑 (ㅄ→ㅂ)");
        let mut c = vec![11, 3, 11];
        apply_pron_trans_rule(&mut c);
        assert_eq!(c, vec![11, 3, 17], "삶→삼 (ㄻ→ㅁ)");
    }

    #[test]
    fn is_eomi_decomp_table() {
        let true_tags: &[u8] = b"TUVWXYZ[\\]^_`abcdghim@CDB";
        let false_tags: &[u8] = b"0123456789:;<=>?AEFGHIJKLMNOPQRSefjklno";
        for &t in true_tags {
            assert!(is_eomi(t), "{:?} should be Eomi", t as char);
        }
        for &t in false_tags {
            assert!(!is_eomi(t), "{:?} should not be Eomi", t as char);
        }
        for &t in b"CD@ghim" {
            assert!(
                is_eomi(t),
                "{:?} should be Eomi (excluded group)",
                t as char
            );
        }
        assert!(!is_eomi(0x80));
        assert!(!is_eomi(0xFF));
    }

    #[test]
    fn is_mack_him_jong_decomp_set() {
        for j in 1..=29u8 {
            let expect = matches!(
                j,
                2 | 3 | 4 | 8 | 19 | 20 | 21 | 22 | 24 | 25 | 26 | 27 | 28
            );
            assert_eq!(is_mack_him_jong(j), expect, "jong {j}");
        }
    }

    #[test]
    fn is_nasal_decomp_set() {
        for c in 1..=21u8 {
            let expect = matches!(c, 7 | 4 | 13 | 8);
            assert_eq!(is_nasal(c), expect, "cho {c}");
        }
    }

    #[test]
    fn is_symbol_pumsa_decomp() {
        for t in 0..=255u8 {
            let expect = t.wrapping_add(0xB4) < 7;
            assert_eq!(is_symbol_pumsa(t), expect, "tag {t:#04x}");
        }
    }

    #[test]
    fn tensing_school() {
        let mut w = wc(&[20, 3, 2, 2, 13, 1], &[b'0'; 6]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[20, 3, 2, 3, 13, 1], "학교→학꾜 (ㄱㄱ→ㄱㄲ)");
    }

    #[test]
    fn aspiration_h_drop() {
        let mut w = wc(&[5, 13, 29, 2, 13, 1], &[b'0'; 6]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[5, 13, 1, 17, 13, 1], "놓고→노코");
    }

    #[test]
    fn liaison_an_a() {
        let mut w = wc(&[13, 3, 5, 13, 3, 1], &[b'0'; 6]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[13, 3, 1, 4, 3, 1], "안아→아나");
    }

    #[test]
    fn liquid_sinla_decomp_faithful() {
        let mut w = wc(&[11, 3, 5, 7, 3, 1], &[b'0'; 6]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[11, 3, 5, 7, 3, 1],);
        let mut c = vec![11, 3, 5, 7, 3, 1];
        apply_pron_trans_rule(&mut c);
        assert_eq!(c, vec![11, 3, 9, 7, 3, 1]);
    }

    #[test]
    fn liquid_after_nasal_yong_eon() {
        let mut w = wc(&[13, 27, 17, 7, 19, 1, 11, 20, 1], &[b'B'; 9]);
        pronun_intra_word(&mut w);
        assert_eq!(
            &w.cvc,
            &[13, 27, 17, 4, 19, 1, 11, 20, 1],
            "음료수→음뇨수 (유성용언)"
        );
        let mut w = wc(&[13, 27, 17, 7, 19, 1, 11, 20, 1], &[b'0'; 9]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[13, 27, 17, 7, 19, 1, 11, 20, 1],);
    }

    #[test]
    fn vowel_inflec_gachi() {
        let mut w = wc(&[2, 3, 27, 13, 29, 1], &[b'0'; 6]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[2, 3, 1, 16, 29, 1], "같이→가치 (ㅌ+ㅣ→ㅊ)");

        let mut w = wc2(&[2, 3, 27, 13, 29, 1], &[b'0'; 6]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[2, 3, 1, 5, 29, 1],);
    }

    #[test]
    fn ui_rules() {
        let mut w = wc(&[13, 28, 1], &[b'V'; 3]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[13, 10, 1], "의(조사)→에");

        let mut w = wc(&[20, 28, 1, 8, 3, 5], &[b'0'; 6]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc[0..3], &[20, 29, 1], "희→히 (ㅢ→ㅣ)");

        let mut w = wc(&[13, 28, 1, 8, 3, 5], &[b'0'; 6]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc[0..3], &[13, 28, 1]);

        let mut w = wc(&[13, 7, 1, 13, 28, 1], &[b'0'; 6]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc[3..6], &[13, 29, 1]);
    }

    #[test]
    fn revise_jong_special_cases_h_i() {
        let mut w = wc2(&[9, 3, 12, 11, 3, 9], b"HHH666");
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[9, 3, 9, 11, 3, 9],);

        let mut w = wc2(&[9, 3, 12, 11, 3, 9], b"HHH000");
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[9, 3, 9, 11, 3, 9],);

        let mut w = wc(&[9, 3, 19, 7, 3, 1], b"HHH000");
        pronun_intra_word(&mut w);
        assert_eq!(
            &w.cvc,
            &[9, 3, 17, 4, 3, 1],
            "밥라 (H/0) → 밤나 (ㅂㄹ→ㅁㄴ)"
        );

        let mut w = wc(&[9, 3, 19, 13, 26, 2], b"III000");
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[9, 3, 17, 4, 26, 2], "밥육 (I/0) → 밤눅");
    }

    #[test]
    fn revise_jong_top_special() {
        let mut w = wc2(&[2, 3, 2, 13, 18, 1], b"000777");
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[2, 3, 2, 13, 18, 1]);
        let mut w = wc(&[2, 3, 2, 13, 18, 1], &[b'0'; 6]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[2, 3, 1, 2, 18, 1], "각외 (0/0) → 가괴 (연음)");
    }

    #[test]
    fn revise_jong_morph_gate() {
        let mut w = wc2(&[20, 3, 2, 2, 13, 1], &[b'0'; 6]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[20, 3, 2, 2, 13, 1],);

        let mut w = wc(&[20, 3, 2, 2, 13, 1], &[b'0'; 6]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[20, 3, 2, 3, 13, 1],);

        let mut w = wc2(&[20, 3, 2, 2, 13, 1], b"000222");
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[20, 3, 2, 2, 13, 1],);

        let mut w = wc2(&[20, 3, 2, 2, 13, 1], b"000TTT");
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[20, 3, 2, 3, 13, 1],);
    }

    #[test]
    fn except_inter_motph_y_ss() {
        let mut w = wc2(&[9, 3, 27, 11, 29, 1], b"000YYY");
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[9, 3, 8, 12, 29, 1],);
    }

    #[test]
    fn except_inter_motph_6_block() {
        let mut w = wc2(&[9, 3, 9, 9, 4, 1], b"000666");
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[9, 3, 9, 10, 4, 1], "랄+래 (0/6) → ㄹ→ㄸ");
        let mut w = wc2(&[9, 3, 9, 2, 4, 1], b"000666");
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[9, 3, 9, 2, 4, 1],);
    }

    #[test]
    fn except_inter_motph_common_chain() {
        let mut w = wc2(&[9, 3, 10, 5, 3, 1], b"BBBTTT");
        pronun_intra_word(&mut w);
        assert_eq!(
            &w.cvc,
            &[9, 3, 9, 6, 3, 1],
            "닭+다 (B/T) → 달따 (ㄺ→ㄹ, ㄴ→ㄸ)"
        );

        let mut w = wc(&[9, 3, 10, 5, 3, 1], &[b'0'; 6]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[9, 3, 2, 6, 3, 1]);
    }

    #[test]
    fn between_word_rules() {
        let mut f = wc(&[11, 7, 9, 20, 27, 9], &[b'0'; 6]);
        let mut b = wc(&[5, 4, 1], &[b'0'; 3]);
        pronun_between_word(&mut f, &mut b);
        assert_eq!(&b.cvc, &[6, 4, 1], "ㄹ+내 → ㄸ내");

        let mut f = wc(&[2, 3, 26], &[b'0'; 3]);
        let mut b = wc(&[5, 27, 23], &[b'0'; 3]);
        pronun_between_word(&mut f, &mut b);
        assert_eq!(&b.cvc, &[6, 27, 23], "ㅋ+능 → ㄸ능");
    }

    #[test]
    fn yu_nyu_n_to_ng() {
        let mut w = wc(&[5, 3, 5, 7, 26, 9], &[b'B'; 6]);
        pronun_intra_word(&mut w);
        assert_eq!(&w.cvc, &[5, 3, 5, 13, 26, 9], "난률 → 나율 (ㄴ→ㅇ)");
    }

    #[allow(clippy::field_reassign_with_default)]
    #[test]
    fn pronun_yu_nyu_loop_continuation() {
        let mut w = WordCvc::default();
        w.cvc = vec![13, 27, 17, b'X', 7, 19, 1];
        w.ty = vec![b'H', b'H', b'H', b'X', b'H', b'H', b'H'];
        w.tag = vec![b'B'; 7];
        w.mpos = vec![0; 7];
        pronun_intra_word(&mut w);
        assert_eq!(w.cvc[4], 7);
    }

    #[test]
    fn compare_rule_real_data() {
        let mut raw = [0u8; 20];
        raw[..6].copy_from_slice(&[14, 20, 1, 8, 7, 1]);
        raw[6] = 0;
        raw[12] = b'0';
        raw[13] = 3;
        raw[15] = 1;
        raw[16] = b'0';
        raw[17] = 1;
        raw[18] = b'1';
        raw[19] = 1;
        let rule = ktts_dict::pronrule::PronRule {
            cond: raw[..12].try_into().unwrap(),
            apply: raw[12..].try_into().unwrap(),
        };
        let mut w = wc(&[2, 3, 2, 14, 20, 1, 8, 7, 1], &[b'0'; 9]);
        w.mpos = vec![0, 0, 0, 1, 1, 1, 1, 1, 1];
        assert!(compare_rule(&rule, &w, 2));
        let mut w = wc(&[2, 3, 2, 14, 20, 1, 8, 7, 1, 4, 29, 1], &[b'0'; 12]);
        w.mpos = vec![0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1];
        assert!(!compare_rule(&rule, &w, 2));
        let mut w = wc(&[2, 3, 2, 4, 13, 1, 14, 20, 1, 8, 7, 1], &[b'0'; 12]);
        w.mpos = vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1];
        assert!(!compare_rule(&rule, &w, 5),);
        let mut w = wc(&[2, 3, 2, 14, 20, 1, 8, 7, 1], &[b'0'; 9]);
        w.mpos = vec![0, 0, 0, 1, 1, 1, 1, 1, 1];
        w.tag[3] = b'1';
        assert!(!compare_rule(&rule, &w, 2));
    }

    #[test]
    fn search_rule_dic_requires_boundary() {
        let rule = ktts_dict::pronrule::PronRule {
            cond: [0; 12],
            apply: [0; 8],
        };
        let w = wc(&[14, 20, 1, 8, 7, 1], &[b'0'; 6]);
        assert!(!compare_rule(&rule, &w, 2));
    }

    #[allow(clippy::field_reassign_with_default)]
    #[test]
    fn merge_solo_jong_merges_into_prev_jong() {
        let mut w = wc(&[2, 3, 1, 5, 27, 1, 17, 3, 1, 1, 1, 5], &[b'0'; 12]);
        w.mpos = vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 2, 2, 2];
        merge_solo_jong(&mut w);
        assert_eq!(&w.cvc, &[2, 3, 1, 5, 27, 1, 17, 3, 5], "가드카+ㄴ → 가드칸");
        assert_eq!(w.syllable_starts(), vec![0, 3, 6]);
        assert_eq!(w.ty.len(), 9);
        assert_eq!(w.tag.len(), 9);
        assert_eq!(w.mpos.len(), 9);

        let mut w = wc(&[13, 29, 19, 1, 1, 17, 4, 29, 1], &[b'0'; 9]);
        merge_solo_jong(&mut w);
        assert_eq!(&w.cvc, &[13, 29, 19, 4, 29, 1],);

        let mut w = wc(&[1, 1, 17, 4, 29, 1, 5, 3, 1], &[b'0'; 9]);
        merge_solo_jong(&mut w);
        assert_eq!(&w.cvc, &[1, 1, 17, 4, 29, 1, 5, 3, 1]);

        let mut w = WordCvc::default();
        w.cvc = vec![2, 3, 1, b'X', 1, 1, 5];
        w.ty = vec![b'H', b'H', b'H', b'X', b'H', b'H', b'H'];
        w.tag = vec![b'0'; 7];
        w.mpos = vec![0; 7];
        merge_solo_jong(&mut w);
        assert_eq!(&w.cvc, &[2, 3, 1, b'X', 1, 1, 5],);
    }
}
