pub(crate) fn to_portable_u32x2(value: u64) -> [u32; 2] {
    let bytes = value.to_be_bytes();
    [
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
    ]
}

pub(crate) fn from_portable_u32x2(bytes: &[u8]) -> u64 {
    debug_assert_eq!(bytes.len(), 8);
    let left = u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let right = u32::from_ne_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    (u64::from(left) << 32) | u64::from(right)
}
