#[cfg(test)]
pub(crate) fn u32_to_u8_vec(num: u32) -> Vec<u8> {
    vec![
        (num & 0xFF) as u8,
        ((num >> 8) & 0xFF) as u8,
        ((num >> 16) & 0xFF) as u8,
        ((num >> 24) & 0xFF) as u8,
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn u32_to_u8_convertion() {
        assert_eq!(super::u32_to_u8_vec(0u32), vec![0u8, 0u8, 0u8, 0u8]);
        assert_eq!(super::u32_to_u8_vec(1u32), vec![1u8, 0u8, 0u8, 0u8]);
        assert_eq!(super::u32_to_u8_vec(256u32), vec![0u8, 1u8, 0u8, 0u8]);
        assert_eq!(super::u32_to_u8_vec(65536u32), vec![0u8, 0u8, 1u8, 0u8]);
        assert_eq!(super::u32_to_u8_vec(16777216u32), vec![0u8, 0u8, 0u8, 1u8]);
    }
}
