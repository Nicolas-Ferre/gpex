use gpex::{Log, LogLevel};
use std::fs;
use std::path::Path;

#[test]
fn save_and_load_program() -> Result<(), Vec<Log>> {
    let (program, _) = gpex::compile(Path::new("tests/lib/valid"), false)?;
    let out_path = Path::new("tests/lib/out1.json");
    gpex::save_compiled(&program, out_path)?;
    let result = gpex::load_compiled(out_path);
    if out_path.is_file() {
        _ = fs::remove_file(out_path);
    }
    assert!(result.is_ok());
    Ok(())
}

#[test]
#[expect(clippy::expect_used)]
fn save_in_non_existing_folder() -> Result<(), Vec<Log>> {
    let (program, _) = gpex::compile(Path::new("tests/lib/valid"), false)?;
    let result = gpex::save_compiled(&program, Path::new("tests/missing/out.json"));
    let errors = result.expect_err("saving should generate errors");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].level, LogLevel::Error);
    assert!(errors[0].location.is_none());
    assert_eq!(errors[0].inner.len(), 0);
    assert!(
        errors[0]
            .to_string()
            .starts_with("error: cannot write \"tests/missing/out.json\": ")
    );
    Ok(())
}

#[test]
#[expect(clippy::expect_used)]
fn load_non_existing_file() {
    let result = gpex::load_compiled(Path::new("tests/missing/out.json"));
    let errors = result.expect_err("loading should generate errors");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].level, LogLevel::Error);
    assert!(errors[0].location.is_none());
    assert_eq!(errors[0].inner.len(), 0);
    assert!(
        errors[0]
            .to_string()
            .starts_with("error: cannot read \"tests/missing/out.json\": ")
    );
}

#[test]
#[expect(clippy::expect_used)]
fn load_invalid_file() {
    let result = gpex::load_compiled(Path::new("tests/lib/main.rs"));
    let errors = result.expect_err("loading should generate errors");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].level, LogLevel::Error);
    assert!(errors[0].location.is_none());
    assert_eq!(errors[0].inner.len(), 0);
    assert_eq!(
        errors[0].to_string(),
        "error: invalid compiled program \"tests/lib/main.rs\"\n"
    );
}
