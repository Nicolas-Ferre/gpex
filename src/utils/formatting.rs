pub(crate) fn f32_to_string(value: f32) -> String {
    let string_value = format!("{value}");
    if string_value.contains('.') {
        string_value
    } else {
        format!("{string_value}.0")
    }
}
