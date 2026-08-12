#[derive(Debug, Clone, Default)]
pub(crate) struct BiTagInfo {
    pub pw_johab: Vec<u16>,
    pub ch_tag1: u8,
    pub ch_tag2: u8,
    pub ch_tag3: u8,
    pub b_bi_flag: u8,
    pub b_union_flag: u8,
    pub b_sp_flag: u8,
    pub w_union_depth: u16,
}

#[derive(Debug, Clone)]
pub(crate) struct BiWord {
    pub surface: Vec<u16>,
    pub root: Vec<u16>,
    pub to_word: Vec<u16>,
    pub morph_pos: Vec<u8>,
    pub morph_surfaces: Vec<Vec<u16>>,
    pub word_of_sen: Vec<u16>,
    pub b_word_sen: u8,
    pub w_morph_cnt: u16,
    pub b_break_info: u8,
}

impl BiWord {
    pub(crate) const fn len(&self) -> usize {
        self.surface.len()
    }
}

fn wstr_cmp(a: &[u16], b: &[u16]) -> bool {
    let mut i = 0;
    loop {
        let ca = a.get(i).copied().unwrap_or(0);
        let cb = b.get(i).copied().unwrap_or(0);
        if ca == 0 || cb == 0 {
            return ca == cb;
        }
        if ca != cb {
            return false;
        }
        i += 1;
    }
}

pub(crate) fn get_bi_tag_str_copy(word: &BiWord, n_pos: usize) -> u8 {
    let pos = &word.morph_pos;
    let c_var1 = *pos.get(n_pos).unwrap_or(&0);
    if n_pos != 0 {
        let b_var8 = c_var1 == b'7';
        let c_var2 = *pos.get(n_pos - 1).unwrap_or(&0);
        let w_var3 = word
            .morph_surfaces
            .get(n_pos - 1)
            .and_then(|s| s.first())
            .copied()
            .unwrap_or(0);
        let w_var4 = word
            .morph_surfaces
            .get(n_pos)
            .and_then(|s| s.first())
            .copied()
            .unwrap_or(0);
        if w_var4 == 0xb370 {
            if b_var8 && (w_var3 == 0xc740 || w_var3 == 0x3134 || w_var3 == 0xb294) {
                return b'g';
            }
        } else if w_var4 == 0xac8c && b_var8 && w_var3 == 0x3134 {
            return b'`';
        }
        if c_var2 == b'_' && b_var8 {
            return b'_';
        }
        let b_var8 = c_var1 == b']';
        if c_var2 == b'`' {
            return if b_var8 { b'`' } else { c_var1 };
        }
        if c_var2 == b'X' {
            return if b_var8 { b'X' } else { c_var1 };
        }
        if c_var2 == b'g' {
            return if b_var8 { b'g' } else { c_var1 };
        }
        if c_var2 == b'Z' {
            return if b_var8 { b'Z' } else { c_var1 };
        }
        return c_var1;
    }
    if c_var1 == b'2' {
        if pos.get(1) == Some(&b'D') {
            return b'C';
        }
    } else if c_var1 == b'K' {
        return b'@';
    }
    let w_morph_cnt = word.w_morph_cnt as usize;
    let mut i_var6 = 0usize;
    if w_morph_cnt >= 1 && pos.first() != Some(&b'@') && pos.first() != Some(&b'B') {
        loop {
            i_var6 += 1;
            if w_morph_cnt <= i_var6 {
                break;
            }
            let c = pos[i_var6];
            if c == b'B' || c == b'@' {
                break;
            }
        }
    }
    if i_var6 == w_morph_cnt { c_var1 } else { b'@' }
}

pub(crate) fn get_morph_info(words: &[BiWord], tags: &mut [BiTagInfo]) {
    let n = words.len();
    for t in tags.iter_mut() {
        *t = BiTagInfo::default();
    }
    tags[n + 1].ch_tag1 = b'O';
    tags[n + 1].ch_tag2 = b'P';
    tags[n + 1].ch_tag3 = b'P';
    for (i, w) in words.iter().enumerate() {
        let t = &mut tags[i + 1];
        t.ch_tag1 = get_bi_tag_str_copy(w, 0);
        t.pw_johab.clone_from(&w.to_word);
        let last = w.w_morph_cnt.saturating_sub(1) as usize;
        t.ch_tag2 = get_bi_tag_str_copy(w, last);
        if t.ch_tag2 == b'M' {
            t.ch_tag3 = get_bi_tag_str_copy(w, last.saturating_sub(1));
        }
    }
}

pub(crate) fn tag_stream(tags: &[BiTagInfo], n: usize) -> [u8; 5] {
    let mut s = [0u8; 5];
    s[0] = tags[n].ch_tag2;
    s[1] = tags[n + 1].ch_tag2;
    s[2] = tags[n + 1].ch_tag1;
    s[3] = tags[n].ch_tag1;
    s[4] = 0;
    s
}

#[allow(clippy::too_many_arguments)]
#[expect(
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn is_attrib_compare(
    attrs: &[ktts_dict::birule::AttribRecord],
    b_not_flag: u8,
    ch_cmp_str_idx: i8,
    w_attrib_idx: i16,
    pw_to: &[u16],
    pw_root: &[u16],
    ch_tag2: u8,
    ch_tag1: u8,
    pw_word_of_sen: &[u16],
) -> bool {
    if w_attrib_idx - 1 < 0 {
        return false;
    }
    let sw_str: Vec<u16> = match ch_cmp_str_idx {
        1 => pw_to.to_vec(),
        2 => pw_root.to_vec(),
        3 => vec![u16::from(ch_tag2)],
        4 => vec![u16::from(ch_tag1)],
        _ => pw_word_of_sen.to_vec(),
    };
    let rec = &attrs[(w_attrib_idx - 1) as usize];
    for item in &rec.items {
        if wstr_cmp(item, &sw_str) {
            return b_not_flag != 0;
        }
    }
    b_not_flag == 0
}

fn strstr_pos(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

fn cw(s: &mut Vec<u16>) {
    if s.first() == Some(&u16::from(b',')) {
        *s = vec![u16::from(b';')];
    }
}

#[allow(clippy::too_many_arguments)]
#[expect(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
fn rule_morph_and_word_of_sen_cmp(
    n_cur_word_idx: usize,
    word: &BiWord,
    n_word_of_sen: usize,
    morph: &ktts_dict::birule::RuleMorph,
    attrs: &[ktts_dict::birule::AttribRecord],
    tags: &[BiTagInfo],
) -> i8 {
    let next = &tags[n_cur_word_idx + 1];
    let mut sw_to: Vec<u16> = if next.ch_tag2 == b'M' {
        next.pw_johab.clone()
    } else {
        word.to_word.clone()
    };
    cw(&mut sw_to);

    if morph.b_root_not_flag == 0 {
        if morph.ch_root_pos_tag as u8 != next.ch_tag1 && morph.ch_root_pos_tag as u8 != b'*' {
            return -1;
        }
    } else if morph.ch_root_pos_tag as u8 == next.ch_tag1 {
        return -1;
    }
    if morph.b_end_not_flag == 0 {
        if morph.ch_end_pos_tag as u8 != next.ch_tag2 && morph.ch_end_pos_tag as u8 != b'*' {
            return -1;
        }
    } else if morph.ch_end_pos_tag as u8 == next.ch_tag2 {
        return -1;
    }
    let sw_string = morph.string();
    if morph.b_string_not_flag == 0 {
        if !wstr_cmp(&sw_string, &sw_to) && sw_string.first() != Some(&u16::from(b'*')) {
            return -1;
        }
    } else if wstr_cmp(&sw_string, &sw_to) {
        return -1;
    }
    if morph.b_attrib_exist != 0 {
        if is_attrib_compare(
            attrs,
            morph.b_first_not_flag,
            morph.ch_first_cmp_str_idx,
            morph.w_first_attrib_idx,
            &sw_to,
            &word.root,
            next.ch_tag2,
            next.ch_tag1,
            &word.word_of_sen,
        ) {
            return -1;
        }
        if morph.w_second_attrib_idx != 0
            && is_attrib_compare(
                attrs,
                morph.b_second_not_flag,
                morph.ch_second_cmp_str_idx,
                morph.w_second_attrib_idx,
                &sw_to,
                &word.root,
                next.ch_tag2,
                next.ch_tag1,
                &word.word_of_sen,
            )
        {
            return -1;
        }
    }
    let b_str_pos = morph.b_str_pos as usize;
    if morph.b_start_flag == 0 {
        if b_str_pos != 0 && b_str_pos < n_word_of_sen - n_cur_word_idx {
            return -1;
        }
    } else if b_str_pos != 0 && b_str_pos <= n_cur_word_idx {
        return -1;
    }
    let s_len = word.surface.len();
    let b_str_len = morph.b_str_len as usize;
    if morph.b_over_flag == 0 {
        if b_str_len != 0 && b_str_len < s_len {
            return -1;
        }
    } else if b_str_len != 0 && s_len < b_str_len {
        return -1;
    }
    let pos_arr: &[u8] = &morph.ch_pos_array;
    let needle_end = pos_arr
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(pos_arr.len());
    let needle = &pos_arr[..needle_end];
    if morph.b_pos_not_flag == 0 {
        if strstr_pos(&word.morph_pos, needle).is_none() && pos_arr.first() != Some(&b'*') {
            return -1;
        }
    } else if strstr_pos(&word.morph_pos, needle).is_some() {
        return -1;
    }
    morph.b_bi_idx as i8
}

#[allow(clippy::needless_range_loop)]
#[allow(clippy::type_complexity)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
fn rule_search(
    n_word_idx: usize,
    n_word_of_sen: usize,
    words: &[BiWord],
    tags: &[BiTagInfo],
    rule_info: &ktts_dict::birule::BIRuleFile,
) -> (Vec<i8>, Vec<i8>, usize, u16, usize) {
    let mut sch_bi_array = [-1i8; 16];
    let mut sch_union_array = [-1i8; 16];
    let mut w_last_search_cnt: u16 = 0;
    let mut n_morph_num: usize = 0;
    let mut b_var2 = false;
    for (n_i, rule) in rule_info.rules.iter().enumerate() {
        let n_rule_morph_num = rule.morphs.len();
        if n_rule_morph_num > n_word_of_sen - n_word_idx {
            continue;
        }
        let mut sch_bi_array_buff = [-1i8; 16];
        let mut ok = true;
        let mut b_union = false;
        for (i, morph) in rule.morphs.iter().enumerate() {
            let c = rule_morph_and_word_of_sen_cmp(
                n_word_idx + i,
                &words[n_word_idx + i],
                n_word_of_sen,
                morph,
                &rule_info.attributes,
                tags,
            );
            if c < 0 {
                ok = false;
                break;
            }
            sch_bi_array_buff[i] = c;
            if c > 5 {
                b_union = true;
            }
        }
        if !ok {
            continue;
        }
        w_last_search_cnt = n_i as u16;
        if b_union {
            sch_union_array = sch_bi_array_buff;
            n_morph_num = n_rule_morph_num;
        } else if !b_var2 || n_morph_num < n_rule_morph_num {
            sch_bi_array = sch_bi_array_buff;
            b_var2 = true;
            n_morph_num = n_rule_morph_num;
        }
    }
    let mut count = 0usize;
    for i in 0..n_morph_num {
        if sch_bi_array[i] > 0 {
            count += 1;
        }
    }
    (
        sch_bi_array[..n_morph_num].to_vec(),
        sch_union_array[..n_morph_num].to_vec(),
        n_morph_num,
        w_last_search_cnt,
        count,
    )
}

fn set_break_info(
    words: &[BiWord],
    tags: &mut [BiTagInfo],
    rule_info: &ktts_dict::birule::BIRuleFile,
) {
    let n_morph_size = words.len();
    let mut break_phrase = vec![-1i8; n_morph_size + 1];
    let mut union_phrase = vec![-1i8; n_morph_size + 1];
    let mut b_last_sp_flag = false;
    let mut i_var5 = 0usize;
    while i_var5 < n_morph_size {
        let (sch_bi, sch_union, n_morph_num, w_last, count) =
            rule_search(i_var5, n_morph_size, words, tags, rule_info);
        for i in 0..n_morph_num {
            if i_var5 + i + 1 < break_phrase.len() {
                break_phrase[i_var5 + i + 1] = sch_bi[i];
                union_phrase[i_var5 + i + 1] = sch_union[i];
            }
        }
        if ({ rule_info.w_begin }..={ rule_info.w_last }).contains(&w_last) {
            b_last_sp_flag = true;
        }
        i_var5 += count.max(1);
    }
    if !b_last_sp_flag && i_var5 < break_phrase.len() {
        break_phrase[i_var5] = -1;
    }
    let mut i = 1usize;
    while i <= n_morph_size {
        let c = break_phrase[i];
        if c == 3 {
            tags[i].b_bi_flag = 1;
        }
        if c == 4 {
            tags[i].b_bi_flag = 2;
        }
        if union_phrase[i] > 5 {
            tags[i].b_union_flag = 1;
            if union_phrase[i] == 6 {
                tags[i].w_union_depth = 1;
            }
            if union_phrase[i] == 7 {
                tags[i].w_union_depth = 2;
            }
        }
        if break_phrase[i] != 1 {
            i += 1;
            continue;
        }
        tags[i].b_bi_flag = 0xff;
        i += 1;
    }
    tags[n_morph_size].b_bi_flag = 1;
}

#[allow(clippy::if_same_then_else)]
fn set_morph_union(n_morph_size: usize, tags: &mut [BiTagInfo]) {
    let mut n = 1usize;
    while n <= n_morph_size {
        let mut c = tags[n].ch_tag2;
        if c == b'M' {
            c = tags[n].ch_tag3;
        }
        if c == b']' || c == b'T' {
            tags[n].b_sp_flag = 2;
            n += 1;
            continue;
        }
        if (tags[n].ch_tag1 == b'C' || tags[n].ch_tag1 == b'@')
            && c != b'U'
            && c != b'`'
            && c != b'Y'
            && c != b'W'
            && c != b'X'
        {
            tags[n].b_sp_flag = 1;
        } else if c == b'h' || c == b'^' {
            tags[n].b_sp_flag = 1;
        }
        n += 1;
    }
}

fn set_hubo_break_info(words: &[BiWord], n_morph_size: usize, tags: &mut [BiTagInfo]) {
    if n_morph_size <= 1 {
        return;
    }
    for i in 1..n_morph_size {
        if tags[i].b_bi_flag != 0xff
            && tags[i].b_bi_flag != 1
            && tags[i].b_union_flag == 0
            && tags[i].b_sp_flag == 2
        {
            let mut n_len = words[i - 1].surface.len() + usize::from(words[i - 1].b_word_sen != 0);
            let mut p = i - 1;
            let mut w = i.saturating_sub(2);
            loop {
                if p == 0 {
                    break;
                }
                if tags[p].b_union_flag == 0 && tags[p].b_bi_flag != 0xff {
                    break;
                }
                n_len += words[w].surface.len() + usize::from(words[w].b_word_sen != 0);
                if w == 0 {
                    break;
                }
                p -= 1;
                w -= 1;
            }
            if n_len > 5 {
                tags[i].b_bi_flag = 5;
            }
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
#[expect(
    clippy::similar_names,
    clippy::too_many_lines,
    reason = "C port: disassembler/domain variable names kept as-is; large ported function"
)]
fn set_correction_ip(words: &[BiWord], n_morph_size: usize, tags: &mut [BiTagInfo]) {
    if n_morph_size < 1 {
        return;
    }
    let word_len =
        |i: usize| -> usize { words[i].surface.len() + usize::from(words[i].b_word_sen != 0) };

    loop {
        let mut changed = false;
        let mut last_b = 1usize;
        let mut acc = 0usize;
        let mut i = 1usize;
        loop {
            acc += word_len(i - 1);
            let i_next = i + 1;
            if tags[i].b_bi_flag.wrapping_sub(1) < 2 {
                if last_b != 0 && i_next != last_b && acc > 0x13 && last_b < i {
                    let mut p = last_b;
                    if tags[p].b_bi_flag != 5 {
                        p += 1;
                        while p < i && tags[p].b_bi_flag != 5 {
                            p += 1;
                        }
                    }
                    if p < i {
                        tags[p].b_bi_flag = 2;
                        changed = true;
                    }
                }
                acc = 0;
                last_b = i_next;
            }
            if n_morph_size <= i {
                break;
            }
            i = i_next;
        }
        if !changed {
            break;
        }
    }

    loop {
        let mut changed = false;
        let mut i_var21 = 0usize;
        let mut i_var25 = 1usize;
        let mut acc = 0usize;
        for pos in 1..=n_morph_size {
            acc += word_len(pos - 1);
            if tags[pos].b_bi_flag.wrapping_sub(1) < 2 {
                i_var21 = pos;
            }
            let mut i_var1 = i_var25;
            if i_var21 != 0 && i_var25 != 0 {
                i_var1 = i_var21 + 1;
                if i_var1 != i_var25 {
                    if acc > 0x1a {
                        let mid = ((i_var21 as i64 - i_var25 as i64) / 2) as usize + i_var25;
                        let mut fwd = mid;
                        while fwd < i_var21
                            && (tags[fwd].b_bi_flag != 0 || tags[fwd].b_union_flag != 0)
                        {
                            fwd += 1;
                        }
                        let mut bwd = mid;
                        while bwd >= i_var25
                            && (tags[bwd].b_bi_flag != 0 || tags[bwd].b_union_flag != 0)
                        {
                            bwd -= 1;
                        }
                        if (i_var25 <= bwd) || (fwd < i_var21) {
                            if fwd - mid < mid - bwd {
                                if fwd < i_var21 {
                                    tags[fwd].b_bi_flag = 2;
                                    changed = true;
                                }
                            } else if i_var25 <= bwd {
                                tags[bwd].b_bi_flag = 2;
                                changed = true;
                            }
                        }
                    }
                    acc = 0;
                }
            }
            i_var25 = i_var1;
        }
        if !changed {
            break;
        }
    }

    let mut end_flag = 1usize;
    let mut acc = 0usize;
    let mut i = 1usize;
    'pass3: loop {
        loop {
            acc += word_len(i - 1);
            let i_next = i + 1;
            if tags[i].b_bi_flag.wrapping_sub(1) >= 2 {
                break;
            }
            if end_flag != 0 && i_next != end_flag && acc > 0x1a && end_flag + 1 < i - 1 {
                let mut p = end_flag + 1;
                while p < i - 1 {
                    if tags[p].ch_tag2 == b'Z' && tags[p].b_bi_flag != 0xff {
                        tags[p].b_bi_flag = 2;
                        break;
                    }
                    p += 1;
                }
            }
            acc = 0;
            end_flag = i_next;
            i = i_next;
            if n_morph_size < i {
                break 'pass3;
            }
        }
        if n_morph_size <= i {
            break;
        }
        i += 1;
    }

    loop {
        let mut changed = false;
        let mut last_b = 1usize;
        let mut acc = 0usize;
        let mut i = 1usize;
        loop {
            acc += word_len(i - 1);
            let i_next = i + 1;
            if tags[i].b_bi_flag.wrapping_sub(1) < 2 {
                let new_last = i_next;
                if last_b != 0 && new_last != last_b && acc >= 0x1b && i > last_b {
                    let mut acc2 = 0usize;
                    let mut p = last_b;
                    while p < i {
                        acc2 += word_len(p - 1);
                        if acc2 > 4 && tags[p].w_union_depth == 2 && tags[p].b_bi_flag != 0xff {
                            tags[p].b_bi_flag = 2;
                            changed = true;
                            break;
                        }
                        p += 1;
                    }
                }
                acc = 0;
                last_b = new_last;
            }
            i = i_next;
            if n_morph_size < i {
                break;
            }
        }
        if !changed {
            break;
        }
    }

    loop {
        let mut changed = false;
        let mut last_b = 1usize;
        let mut acc = 0usize;
        let mut i = 1usize;
        loop {
            acc += word_len(i - 1);
            let i_next = i + 1;
            if tags[i].b_bi_flag.wrapping_sub(1) < 2 {
                if last_b != 0 && i_next != last_b && acc > 0x1a && last_b < i {
                    let mut acc2 = 0usize;
                    let mut u24 = last_b;
                    let mut i12;
                    let mut fired = false;
                    while u24 < i {
                        i12 = u24 - 1;
                        acc2 += word_len(u24 - 1);
                        if acc2 > 4 {
                            if tags[u24].b_bi_flag != 0xff {
                                let (abs11, abs15) = fc98_abs(acc2, word_len(u24 - 1));
                                if abs11 < abs15 && tags[i12].b_bi_flag != 0xff {
                                    tags[i12].b_bi_flag = 2;
                                } else {
                                    tags[u24].b_bi_flag = 2;
                                }
                                changed = true;
                                fired = true;
                                break;
                            }
                            let mut u11 = u24;
                            let mut local44 = u24 + 1;
                            let mut run_len = acc2;
                            let mut u15 = u24;
                            let mut p27 = u24 + 2;
                            if tags[local44].b_bi_flag != 0xff || i <= local44 {
                            } else {
                                u11 = local44;
                                run_len += word_len(u15);
                                if tags[p27].b_bi_flag == 0xff {
                                    loop {
                                        local44 = u11 + 1;
                                        p27 += 1;
                                        u15 = u11;
                                        if i <= local44 {
                                            break;
                                        }
                                        u11 = local44;
                                        run_len += word_len(u15);
                                        if tags[p27].b_bi_flag != 0xff {
                                            local44 = u11 + 1;
                                            break;
                                        }
                                    }
                                } else {
                                    local44 = u11 + 1;
                                }
                            }
                            let s6 = word_len(u11);
                            local44 = if local44 == i { 0 } else { local44 };
                            let mut u11b = u24;
                            let mut nfl = acc2 as i64;
                            let mut p_svar8 = u24.saturating_sub(1);
                            if last_b < u24 && tags[u24].b_bi_flag == 0xff && 1 < u24 {
                                loop {
                                    let u15b = u11b;
                                    u11b = u15b - 1;
                                    nfl -= word_len(u15b - 1) as i64;
                                    if u11b <= last_b || tags[u11b].b_bi_flag != 0xff {
                                        p_svar8 = u15b.saturating_sub(2);
                                        break;
                                    }
                                    if 1 >= u11b {
                                        p_svar8 = u15b.saturating_sub(2);
                                        break;
                                    }
                                }
                            }
                            let s9 = word_len(p_svar8) as i64;
                            let u10 = if u11b == last_b { 0 } else { u11b };
                            let u15f = if u11b == 1 { 1 } else { u10 };
                            let lhs = 0xf_i64 - (nfl - s9);
                            let rhs = run_len as i64 + s6 as i64 - 0xf;
                            if (u11b == 1 || u10 != 0) && lhs < rhs {
                                if u15f < tags.len() {
                                    tags[u15f].b_bi_flag = 2;
                                    changed = true;
                                }
                            } else if local44 == 0 {
                                let (abs11, abs15) = fc98_abs(acc2, word_len(u24 - 1));
                                if abs11 < abs15 && tags[i12].b_bi_flag != 0xff {
                                    tags[i12].b_bi_flag = 2;
                                } else {
                                    tags[u24].b_bi_flag = 2;
                                }
                                changed = true;
                            } else {
                                tags[local44].b_bi_flag = 2;
                                changed = true;
                            }
                            fired = true;
                            break;
                        }
                        u24 += 1;
                    }
                    let _ = fired;
                }
                acc = 0;
                last_b = i_next;
            }
            if n_morph_size <= i {
                break;
            }
            i = i_next;
        }
        if !changed {
            break;
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
const fn fc98_abs(i_var25: usize, s_var6: usize) -> (u32, u32) {
    let u11_32 = (i_var25 as u32)
        .wrapping_sub(s_var6 as u32)
        .wrapping_sub(0xf);
    let u11_i = u11_32 as i32;
    let uvar10 = u11_i >> 31;
    let abs11 = (u11_32 ^ uvar10 as u32).wrapping_sub(uvar10 as u32);
    let w15 = (i_var25 as u32).wrapping_sub(0xf);
    let uvar15 = (w15 >> 31) as i32;
    let abs15 = (w15 ^ uvar15 as u32).wrapping_sub(uvar15 as u32);
    (abs11, abs15)
}

#[allow(clippy::too_many_arguments)]
fn set_bi_info(
    words: &[BiWord],
    n_morph_size: usize,
    tags: &mut [BiTagInfo],
    rule_info: &ktts_dict::birule::BIRuleFile,
    use_rules: bool,
) {
    if use_rules {
        set_break_info(words, tags, rule_info);
    }
    set_morph_union(n_morph_size, tags);
    set_hubo_break_info(words, n_morph_size, tags);
    set_correction_ip(words, n_morph_size, tags);
}

#[allow(clippy::needless_range_loop)]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "C port: index/math casts with wrap semantics"
)]
#[expect(
    clippy::many_single_char_names,
    clippy::too_many_lines,
    reason = "C port variable names; large ported function"
)]
fn set_short_pause(
    n_morph_size: usize,
    tags: &[BiTagInfo],
    prob: &ktts_dict::birule::BIProbDic,
) -> Vec<u8> {
    const RT_INDEX_INIT: f32 = 10000.0;
    const TRANS_DEFAULT: f32 = 1e-6;
    const EMISSION_DEFAULT_BITS: u32 = 0x3586_37bd;
    let emission_default = f32::from_bits(EMISSION_DEFAULT_BITS);
    let n = n_morph_size;

    let mut forced = vec![-1i8; n + 1];
    for (i, t) in tags.iter().enumerate().take(n + 1) {
        if t.b_bi_flag.wrapping_sub(1) < 2 {
            forced[i] = 3;
        }
    }
    let mut p = vec![[(0.0f32, -1i32); 4]; n + 1];
    p[0][3] = (1.0, -1);
    let mut r_t_index = RT_INDEX_INIT;
    let mut before_tag = [0u8; 5];
    let level_char = |grp: usize| -> u8 { b"opqr"[grp] };

    for pos in 1..=n {
        let tag = tag_stream(tags, pos);
        let mut grp_stream = [[emission_default; 4]; 4];
        let mut max_len = 0usize;
        for grp in 0..4 {
            let mut key = [0u8; 8];
            key[0] = level_char(grp);
            let mut k = 0usize;
            while k < 4 {
                let c = tag[k];
                key[k + 1] = c;
                if c == 0 {
                    break;
                }
                key[k + 2] = 0;
                match prob.lookup(&key[..k + 2]) {
                    Some(v) if v >= 0.0 => {
                        grp_stream[grp][k] = v;
                        if max_len < k {
                            max_len = k;
                        }
                    }
                    _ => break,
                }
                k += 1;
            }
        }
        let sr_max_level: [f32; 4] = [
            grp_stream[0][max_len],
            grp_stream[1][max_len],
            grp_stream[2][max_len],
            grp_stream[3][max_len],
        ];
        let mut f_var12 = 0.0f32;
        for level in 0..4 {
            let (r_val, n_pos): (f32, i32) =
                if pos == n || forced[pos] == -1 || forced[pos] == level as i8 {
                    let mut best = 0.0f32;
                    let mut best_src = 0i32;
                    for src in 0..4 {
                        let t = if pos == 1 {
                            TRANS_DEFAULT * p[0][src].0
                        } else {
                            let mut key = [0u8; 8];
                            key[0] = level_char(src);
                            let mut pr = TRANS_DEFAULT;
                            let mut k = 0usize;
                            while k < 4 {
                                key[k + 1] = before_tag[k];
                                key[k + 2] = level_char(level);
                                key[k + 3] = 0;
                                pr = TRANS_DEFAULT;
                                match prob.lookup(&key[..k + 3]) {
                                    Some(v) if v >= 0.0 => {
                                        pr = v;
                                        k += 1;
                                    }
                                    _ => break,
                                }
                            }
                            pr * p[pos - 1][src].0
                        };
                        if best < t {
                            best = t;
                            best_src = src as i32;
                        }
                    }
                    (best, best_src)
                } else {
                    (0.0, 0)
                };
            let score = r_t_index * sr_max_level[level] * r_val;
            p[pos][level] = (score, n_pos);
            f_var12 = f_var12.max(score);
        }
        before_tag = tag;
        r_t_index = RT_INDEX_INIT / f_var12.max(1e-30);
    }

    let mut break_info = vec![0u8; n + 1];
    break_info[n] = 3;
    if n > 1 {
        let mut level = 3i32;
        for pos in (2..=n).rev() {
            level = p[pos][level as usize].1;
            break_info[pos - 1] = level as u8;
        }
    }
    for i in 1..n {
        if forced[i] == -1 && break_info[i] == 3 {
            break_info[i] = 2;
        }
    }
    for i in 1..n {
        break_info[i] = match break_info[i] {
            1 => 0x0a,
            2 => 0x0b,
            3 => 0x14,
            4 => 0x15,
            _ => 0x00,
        };
    }
    break_info[n] = 0x15;
    break_info
}

pub(crate) fn bi_proc(
    words: &mut [BiWord],
    rule_info: &ktts_dict::birule::BIRuleFile,
    prob: &ktts_dict::birule::BIProbDic,
    use_rules: bool,
) -> Vec<u8> {
    let n_morph_size = words.len();
    let mut tags = vec![BiTagInfo::default(); n_morph_size + 2];
    get_morph_info(words, &mut tags);
    set_bi_info(words, n_morph_size, &mut tags, rule_info, use_rules);
    let break_info = set_short_pause(n_morph_size, &tags, prob);
    let mut out = vec![0u8; n_morph_size];
    for i in 0..n_morph_size {
        words[i].b_break_info = break_info[i + 1];
        out[i] = break_info[i + 1];
    }
    out
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::cast_possible_truncation,
        reason = "test fixtures: oracle values converted with intentional casts"
    )]
    use super::*;
    use crate::cvc;
    use ktts_dict::birule;

    fn break_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(
            std::env::var("KTTSDB_DIR")
                .expect("set KTTSDB_DIR to the dictionary data (kttsdb) directory"),
        )
        .join("KSpeechDic")
        .join("woman")
        .join("Break")
    }

    fn load_rule() -> birule::BIRuleFile {
        let data = std::fs::read(break_dir().join("BIRule.bin")).unwrap();
        birule::parse_rule(&data).unwrap()
    }

    fn load_prob() -> birule::BIProbDic {
        let bin_data = std::fs::read(break_dir().join("BIProb_hash.bin")).unwrap();
        let dic_data = std::fs::read(break_dir().join("BIProb_hash.dic")).unwrap();
        let bin = birule::parse_prob_bin(&bin_data).unwrap();
        birule::parse_prob_dic(&dic_data, &bin.buckets).unwrap()
    }

    fn make_word(cvc_list: &[[u8; 3]], pos_list: &[u8]) -> BiWord {
        let surfaces: Vec<u16> = cvc_list.iter().map(|&c| cvc::cvc_to_char(c)).collect();
        let morph_surfaces: Vec<Vec<u16>> = surfaces.iter().map(|&c| vec![c]).collect();
        BiWord {
            surface: surfaces.clone(),
            root: morph_surfaces.first().cloned().unwrap_or_default(),
            to_word: morph_surfaces.last().cloned().unwrap_or_default(),
            morph_pos: pos_list.to_vec(),
            morph_surfaces,
            word_of_sen: surfaces,
            b_word_sen: 0,
            w_morph_cnt: pos_list.len() as u16,
            b_break_info: 0,
        }
    }

    #[test]
    fn get_bi_tag_str_copy_basic() {
        let w = make_word(&[[2, 3, 1], [4, 3, 1]], b"33");
        assert_eq!(get_bi_tag_str_copy(&w, 0), b'3');
        assert_eq!(get_bi_tag_str_copy(&w, 1), b'3');
        let w3 = make_word(&[[2, 3, 1], [2, 3, 1]], b"0@");
        assert_eq!(get_bi_tag_str_copy(&w3, 0), b'@');
        let w4 = make_word(&[[2, 3, 1], [2, 3, 1]], b"2D");
        assert_eq!(get_bi_tag_str_copy(&w4, 0), b'C');
    }

    #[test]
    fn rule_search_hand_computed() {
        let rule = load_rule();
        let words = vec![
            make_word(&[[2, 3, 1], [4, 3, 1]], b"33"),
            make_word(&[[5, 3, 1], [7, 3, 1]], b"30"),
            make_word(&[[8, 3, 1]], b"0"),
        ];
        let mut tags = vec![BiTagInfo::default(); 5];
        get_morph_info(&words, &mut tags);
        assert_eq!(tags[1].ch_tag1, b'3');
        assert_eq!(tags[1].ch_tag2, b'3');
        assert_eq!(tags[2].ch_tag1, b'3');
        assert_eq!(tags[2].ch_tag2, b'0');
        assert_eq!(tags[3].ch_tag1, b'0');
        assert_eq!(tags[3].ch_tag2, b'0');

        let (bi, un, n_morph, w_last, count) = rule_search(0, 3, &words, &tags, &rule);
        assert_eq!(bi, vec![1, 0], "schBIArray");
        assert_eq!(un, vec![-1, -1], "schUnionArray");
        assert_eq!(n_morph, 2, "nMorphNum");
        assert_eq!(w_last, 1, "wLastSearchCnt (index of R1)");
        assert_eq!(count, 1, "number of BI>0");
        assert!(!({ rule.w_begin }..={ rule.w_last }).contains(&w_last));

        let (bi, un, n_morph, w_last, count) = rule_search(1, 3, &words, &tags, &rule);
        assert_eq!(bi, vec![1, 0], "schBIArray (R10)");
        assert_eq!(un, vec![6, 0], "schUnionArray (R65)");
        assert_eq!(n_morph, 2, "nMorphNum");
        assert_eq!(w_last, 65, "wLastSearchCnt (R65)");
        assert_eq!(count, 1);
        assert!(
            ({ rule.w_begin }..={ rule.w_last }).contains(&w_last),
            "SP present"
        );

        let (bi, _, n_morph, _, count) = rule_search(2, 3, &words, &tags, &rule);
        assert_eq!(bi.len(), 0);
        assert_eq!(n_morph, 0);
        assert_eq!(count, 0);
    }

    #[test]
    fn set_break_info_mapping() {
        let rule = load_rule();
        let words = vec![
            make_word(&[[2, 3, 1], [4, 3, 1]], b"33"),
            make_word(&[[5, 3, 1], [7, 3, 1]], b"30"),
            make_word(&[[8, 3, 1]], b"0"),
        ];
        let mut tags = vec![BiTagInfo::default(); 5];
        get_morph_info(&words, &mut tags);
        set_break_info(&words, &mut tags, &rule);
        assert_eq!(tags[1].b_bi_flag, 0xff, "rule BI:1 → no boundary");
        assert_eq!(
            tags[2].b_bi_flag, 0xff,
            "rule BI:1 at position 1 → no boundary"
        );
        assert_eq!(tags[2].b_union_flag, 1, "union flag (R65)");
        assert_eq!(tags[2].w_union_depth, 1, "union depth (BI 6 → 1)");
        assert_eq!(tags[3].b_bi_flag, 1, "end of sentence is always 1");
    }

    #[test]
    fn set_morph_union_test() {
        let words = vec![
            make_word(&[[2, 3, 1]], b"0"),
            make_word(&[[2, 3, 1], [4, 3, 1]], b"0]"),
            make_word(&[[2, 3, 1], [4, 3, 1]], b"@0"),
        ];
        let mut tags = vec![BiTagInfo::default(); 5];
        get_morph_info(&words, &mut tags);
        set_morph_union(3, &mut tags);
        assert_eq!(tags[2].b_sp_flag, 2, "']' → union target");
        assert_eq!(tags[3].b_sp_flag, 1, "'@' start → SP candidate");
    }

    #[test]
    fn set_short_pause_viterbi_runs() {
        let prob = load_prob();
        let words = vec![
            make_word(&[[2, 3, 1], [4, 3, 1]], b"33"),
            make_word(&[[5, 3, 1], [7, 3, 1]], b"30"),
            make_word(&[[8, 3, 1]], b"0"),
        ];
        let mut tags = vec![BiTagInfo::default(); 5];
        get_morph_info(&words, &mut tags);
        let breaks = set_short_pause(3, &tags, &prob);
        assert_eq!(breaks.len(), 4);
        assert_eq!(breaks[3], 0x15, "end of sentence is always B4");
        for &b in &breaks[1..3] {
            assert!(
                b == 0x00 || b == 0x0a || b == 0x0b || b == 0x14,
                "boundary value {b:#x}"
            );
        }
        assert_eq!(tags[4].ch_tag1, b'O');
        assert_eq!(tags[4].ch_tag2, b'P');
    }

    #[test]
    fn bi_proc_full_pipeline() {
        let rule = load_rule();
        let prob = load_prob();
        let mut words = vec![
            make_word(&[[2, 3, 1], [4, 3, 1]], b"33"),
            make_word(&[[5, 3, 1], [7, 3, 1]], b"30"),
            make_word(&[[8, 3, 1]], b"0"),
        ];
        let out = bi_proc(&mut words, &rule, &prob, true);
        assert_eq!(out.len(), 3);
        assert_eq!(
            out[2], 0x15,
            "boundary after the final word is end-of-sentence B4"
        );
        for i in 0..3 {
            assert_eq!(words[i].b_break_info, out[i]);
        }
    }

    fn anthem_words() -> Vec<BiWord> {
        let data: &[(&str, &str, &[&str])] = &[
            ("아치믄", "3]", &["아침", "은"]),
            ("빈나라", "@g", &["빛나", "라"]),
            ("이", "F", &["이"]),
            ("강산", "0", &["강산"]),
            ("은그메", "0W", &["은금", "에"]),
            ("자원도", "0]", &["자원", "도"]),
            ("가드칸", "2D_", &["가득", "하", "ㄴ"]),
            ("이", "F", &["이"]),
            ("세상", "0", &["세상"]),
            ("아름다운", "2D_", &["아름", "답", "ㄴ"]),
            ("내", "F", &["내"]),
            ("조국", "0", &["조국"]),
            ("반만년", "0", &["반만년"]),
            ("오랜", "C_", &["오래", "ㄴ"]),
            ("력싸에", "0W", &["력사", "에"]),
        ];
        data.iter()
            .map(|&(surf, pos, morphs)| {
                let morph_surfaces: Vec<Vec<u16>> =
                    morphs.iter().map(|s| s.encode_utf16().collect()).collect();
                BiWord {
                    surface: surf.encode_utf16().collect(),
                    root: morph_surfaces.first().cloned().unwrap_or_default(),
                    to_word: morph_surfaces.last().cloned().unwrap_or_default(),
                    morph_pos: pos.bytes().collect(),
                    morph_surfaces,
                    word_of_sen: surf.encode_utf16().collect(),
                    b_word_sen: 0,
                    w_morph_cnt: pos.len() as u16,
                    b_break_info: 0,
                }
            })
            .collect()
    }

    #[test]
    fn anthem_biflag_trace_matches_shim6() {
        let rule = load_rule();
        let words = anthem_words();
        let n = words.len();
        let mut tags = vec![BiTagInfo::default(); n + 2];
        get_morph_info(&words, &mut tags);
        set_break_info(&words, &mut tags, &rule);
        let after_break: Vec<u8> = tags[1..=n].iter().map(|t| t.b_bi_flag).collect();
        assert_eq!(
            after_break,
            vec![0, 1, 255, 255, 0, 0, 0, 255, 255, 0, 255, 255, 255, 0, 1],
            "bBiFlag after SetBreakInfo matches shim6 measurement"
        );
        set_morph_union(n, &mut tags);
        let sp: Vec<u8> = tags[1..=n].iter().map(|t| t.b_sp_flag).collect();
        assert_eq!(
            sp,
            vec![2, 1, 0, 0, 0, 2, 1, 0, 0, 1, 0, 0, 0, 1, 0],
            "bSP after SetMorphUnion matches shim6 measurement"
        );
        set_hubo_break_info(&words, n, &mut tags);
        let after_hubo: Vec<u8> = tags[1..=n].iter().map(|t| t.b_bi_flag).collect();
        assert_eq!(
            after_hubo, after_break,
            "SetHuBoBreakInfo leaves it unchanged (shim6)"
        );
        set_correction_ip(&words, n, &mut tags);
        let after_corr: Vec<u8> = tags[1..=n].iter().map(|t| t.b_bi_flag).collect();
        assert_eq!(
            after_corr,
            vec![0, 1, 255, 255, 0, 2, 0, 255, 255, 0, 255, 255, 255, 0, 1],
            "after SetCorrectionIP only pos6 (right after 자원도) is 2 (shim6 measurement)"
        );
    }

    #[test]
    fn anthem_bnd_matches_oracle() {
        let rule = load_rule();
        let prob = load_prob();
        let mut words = anthem_words();
        let out = bi_proc(&mut words, &rule, &prob, true);
        let oracle = [
            0x0a, 0x14, 0x0a, 0x0b, 0x0b, 0x14, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0b, 0x0a, 0x0a,
            0x15,
        ];
        assert_eq!(
            out,
            oracle.to_vec(),
            "the bnd of all 15 anthem words must match the oracle exactly"
        );
        assert_eq!(
            out[13], 0x0a,
            "the boundary right after word[13] 오랜 must be B1 (0x0a)"
        );
    }
}
