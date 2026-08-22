pub(crate) fn round_up(rounded_to: u32, value: u32) -> u32 {
    if rounded_to == 0 {
        0
    } else {
        value.div_ceil(rounded_to) * rounded_to
    }
}
