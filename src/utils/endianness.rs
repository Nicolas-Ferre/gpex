pub(crate) fn to_portable_u32x2(value: u64) -> [u32; 2] {
    let bytes = value.to_be_bytes();
    [
        big_endian_bytes_to_u32(&bytes[0..4]),
        big_endian_bytes_to_u32(&bytes[4..8]),
    ]
}

pub(crate) fn from_portable_u32x2(bytes: &[u8]) -> u64 {
    let left_bytes = native_endian_bytes_to_u32(&bytes[0..4]).to_be_bytes();
    let right_bytes = native_endian_bytes_to_u32(&bytes[4..8]).to_be_bytes();
    u64::from_be_bytes(
        [left_bytes, right_bytes]
            .concat()
            .try_into()
            .unwrap_or_else(|_| unreachable!("merged array should be 8 bytes")),
    )
}

fn big_endian_bytes_to_u32(bytes: &[u8]) -> u32 {
    u32::from_be_bytes(
        bytes
            .try_into()
            .unwrap_or_else(|_| unreachable!("4 bytes should be provided")),
    )
}

fn native_endian_bytes_to_u32(bytes: &[u8]) -> u32 {
    u32::from_ne_bytes(
        bytes
            .try_into()
            .unwrap_or_else(|_| unreachable!("4 bytes should be provided")),
    )
}
