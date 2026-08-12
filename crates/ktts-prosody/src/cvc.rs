const CVC_GROUP_TABLE: [u8; 84] = [
    3, 0, 0, 4, 3, 0, 0, 2, 2, 0, 1, 1, 1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5,
    5, 6, 6, 5, 0, 0, 5, 6, 6, 5, 6, 6, 0, 0, 6, 6, 5, 6, 6, 6, 0, 0, 6, 5, 6, 5, 0, 0, 0, 0, 0, 0,
    0, 3, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 3,
];

#[must_use]
pub const fn cvc_group_index(code: u8) -> u8 {
    let idx = code.wrapping_sub(4) as usize;
    if idx < CVC_GROUP_TABLE.len() {
        CVC_GROUP_TABLE[idx]
    } else {
        0
    }
}

#[must_use]
pub const fn syllable_index(cvc: [u8; 3]) -> u8 {
    let has_cho = cvc[0] != 13 && cvc[0] != 1;
    let has_jong = cvc[2] != 1;
    match (has_cho, has_jong) {
        (false, false) => 0,
        (true, false) => 1,
        (false, true) => 2,
        (true, true) => 3,
    }
}

#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "C port: index/math casts with wrap semantics"
)]
#[expect(
    clippy::similar_names,
    reason = "C port: disassembler/domain variable names kept as-is"
)]
pub fn cvc_to_char(cvc: [u8; 3]) -> u16 {
    let (cho, jung, jong) = (cvc[0], cvc[1], cvc[2]);
    if cho != 1 && jung != 1 {
        let cho_idx = cho.saturating_sub(2) as usize;
        let jung_idx = JUNG_CODE_TO_IDX
            .iter()
            .position(|&c| c == jung)
            .unwrap_or(0);
        let jong_idx = jong_no(jong) as usize;
        let w = (cho_idx as u32) * 588 + (jung_idx as u32) * 28 + jong_idx as u32 + 0xAC00;
        return w as u16;
    }
    if jung == 1 && jong == 1 && cho != 1 {
        return CHO_CODE_TO_JAMO
            .get(cho as usize)
            .copied()
            .unwrap_or(0x3131);
    }
    if cho == 1 && jong == 1 && jung != 1 {
        let idx = JUNG_CODE_TO_IDX
            .iter()
            .position(|&c| c == jung)
            .unwrap_or(0);
        return 0x314F + idx as u16;
    }
    if cho == 1 && jung == 1 && jong != 1 {
        let idx = u16::from(jong_no(jong).saturating_sub(1));
        return 0x3131 + idx;
    }
    0
}

const CHO_CODE_TO_JAMO: [u16; 21] = [
    0x3131, 0x3131, 0x3132, 0, 0x3134, 0x3137, 0x3138, 0x3139, 0x3141, 0x3142, 0x3143, 0x3145,
    0x3146, 0x3147, 0x3148, 0x3149, 0x314A, 0x314B, 0x314C, 0x314D, 0x314E,
];

pub const JUNG_CODE_TO_IDX: [u8; 21] = [
    3, 4, 5, 6, 7, 10, 11, 12, 13, 14, 15, 18, 19, 20, 21, 22, 23, 26, 27, 28, 29,
];

#[must_use]
pub const fn jong_no(code: u8) -> u8 {
    match code {
        2..=17 => code - 1,
        19..=29 => code - 2,
        _ => 0,
    }
}

#[must_use]
pub const fn jong_jamo_idx(code: u8) -> u8 {
    jong_no(code).saturating_sub(1)
}

#[must_use]
pub fn cho_idx(code: u8) -> u8 {
    code.saturating_sub(2).min(18)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cvc_group_table_spot_checks() {
        assert_eq!(cvc_group_index(4), 3);
        assert_eq!(cvc_group_index(7), 4);
        assert_eq!(cvc_group_index(11), 2);
        assert_eq!(cvc_group_index(13), 0);
        assert_eq!(cvc_group_index(14), 1);
        assert_eq!(cvc_group_index(20), 2);
        assert_eq!(cvc_group_index(35), 5);
        assert_eq!(cvc_group_index(37), 6);
        assert_eq!(cvc_group_index(45), 5);
        assert_eq!(cvc_group_index(46), 6);
        assert_eq!(cvc_group_index(61), 5);
        assert_eq!(cvc_group_index(69), 3);
        assert_eq!(cvc_group_index(73), 4);
        assert_eq!(cvc_group_index(81), 3);
        assert_eq!(cvc_group_index(87), 3);
        assert_eq!(cvc_group_index(2), 0);
        assert_eq!(cvc_group_index(3), 0);
        assert_eq!(cvc_group_index(90), 0);
        assert_eq!(cvc_group_index(0), 0);
    }

    #[test]
    fn syllable_index_cases() {
        assert_eq!(syllable_index([13, 3, 5]), 2);
        assert_eq!(syllable_index([13, 3, 1]), 0);
        assert_eq!(syllable_index([2, 3, 1]), 1);
        assert_eq!(syllable_index([13, 3, 6]), 2);
        assert_eq!(syllable_index([20, 3, 5]), 3);
    }

    #[test]
    fn cvc_to_char_basic() {
        assert_eq!(cvc_to_char([2, 3, 1]), '가' as u16);
        assert_eq!(cvc_to_char([4, 3, 1]), '나' as u16);
        assert_eq!(cvc_to_char([20, 3, 5]), '한' as u16);
        assert_eq!(cvc_to_char([13, 3, 5]), '안' as u16);
        assert_eq!(cvc_to_char([13, 27, 5]), '은' as u16);
        assert_eq!(cvc_to_char([5, 10, 1]), '데' as u16);
        assert_eq!(cvc_to_char([2, 10, 1]), '게' as u16);
        assert_eq!(cvc_to_char([4, 23, 5]), '뉜' as u16);
    }
}
