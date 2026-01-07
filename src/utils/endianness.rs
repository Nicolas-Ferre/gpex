pub(crate) fn to_portable_u32x2(value: u64) -> [u32; 2] {
    let bytes = value.to_be_bytes();
    [
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
    ]
}

pub(crate) fn from_portable_u32x2(bytes: &[u8]) -> u64 {
    assert_eq!(bytes.len(), 8);
    let left_bytes = u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]).to_be_bytes();
    let right_bytes = u32::from_ne_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]).to_be_bytes();
    u64::from_be_bytes([
        left_bytes[0],
        left_bytes[1],
        left_bytes[2],
        left_bytes[3],
        right_bytes[0],
        right_bytes[1],
        right_bytes[2],
        right_bytes[3],
    ])
}
