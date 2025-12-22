use gpex::GpuValue;

#[test]
fn display_i32() {
    assert_eq!(format!("{}", GpuValue::I32(123)), "123");
}
