use gpex::{GpuValue, Log, Runner};
use std::path::Path;

#[tokio::test]
async fn run_program() -> Result<(), Vec<Log>> {
    let program = gpex::compile(Path::new("tests/lib/valid"), false)?.0;
    let mut runner = Runner::new(program).await?;
    runner.run_step();
    assert_eq!(
        runner.read_var("inner.inner2.inner:_inner_value"),
        Some(GpuValue::I32(1))
    );
    assert_eq!(runner.read_var("root:_root_value"), Some(GpuValue::I32(2)));
    assert_eq!(runner.read_var("module:invalid"), None);
    Ok(())
}
