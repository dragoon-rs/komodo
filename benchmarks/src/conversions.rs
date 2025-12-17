#[allow(dead_code)]
pub(crate) fn u128_to_le_bytes(i: u128) -> Vec<u8> {
    vec![
        (i >> 8 * 0) as u8,
        (i >> 8 * 1) as u8,
        (i >> 8 * 2) as u8,
        (i >> 8 * 3) as u8,
        (i >> 8 * 4) as u8,
        (i >> 8 * 5) as u8,
        (i >> 8 * 6) as u8,
        (i >> 8 * 7) as u8,
        (i >> 8 * 8) as u8,
        (i >> 8 * 9) as u8,
        (i >> 8 * 10) as u8,
        (i >> 8 * 11) as u8,
        (i >> 8 * 12) as u8,
        (i >> 8 * 13) as u8,
        (i >> 8 * 14) as u8,
        (i >> 8 * 15) as u8,
    ]
}

#[allow(dead_code)]
pub(crate) fn u64_to_le_bytes(i: u64) -> Vec<u8> {
    vec![
        (i >> 8 * 0) as u8,
        (i >> 8 * 1) as u8,
        (i >> 8 * 2) as u8,
        (i >> 8 * 3) as u8,
        (i >> 8 * 4) as u8,
        (i >> 8 * 5) as u8,
        (i >> 8 * 6) as u8,
        (i >> 8 * 7) as u8,
    ]
}

#[allow(dead_code)]
pub(crate) fn u32_to_le_bytes(i: u32) -> Vec<u8> {
    vec![
        (i >> 8 * 0) as u8,
        (i >> 8 * 1) as u8,
        (i >> 8 * 2) as u8,
        (i >> 8 * 3) as u8,
    ]
}
