use gpex::{Log, LogLevel};
use std::path::Path;

#[test]
fn compile_valid_project() -> Result<(), Vec<Log>> {
    let (program, logs) = gpex::compile(Path::new("tests/lib/valid"), false)?;
    assert!(logs.is_empty());
    assert_eq!(program.buffer.size, 8);
    let fields = &program.buffer.fields;
    assert_eq!(fields.len(), 2);
    assert!(fields.contains_key("root:_root_value"));
    assert!(fields.contains_key("inner.inner2.inner:_inner_value"));
    Ok(())
}

#[test]
fn compile_with_warning() -> Result<(), Vec<Log>> {
    let (_, logs) = gpex::compile(Path::new("tests/lib/warning"), false)?;
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].level, LogLevel::Warning);
    Ok(())
}

#[test]
fn compile_with_warning_as_error() {
    let result = gpex::compile(Path::new("tests/lib/warning"), true);
    let errors = result.err();
    assert_eq!(errors.as_ref().map(Vec::len), Some(1));
    assert_eq!(errors.as_ref().map(|e| e[0].level), Some(LogLevel::Warning));
}

#[test]
fn compile_with_error() {
    let result = gpex::compile(Path::new("tests/lib/error"), false);
    let errors = result.err();
    assert_eq!(errors.as_ref().map(Vec::len), Some(1));
    assert_eq!(errors.as_ref().map(|e| e[0].level), Some(LogLevel::Error));
}

#[test]
fn compile_missing_folder() {
    let result = gpex::compile(Path::new("tests/lib/missing"), false);
    let errors = result.err();
    assert_eq!(errors.as_ref().map(Vec::len), Some(1));
    assert_eq!(errors.as_ref().map(|e| e[0].level), Some(LogLevel::Error));
    assert!(errors.as_ref().is_some_and(|e| e[0].loc.is_none()));
    assert_eq!(errors.as_ref().map(|e| e[0].inner.len()), Some(0));
    assert!(
        errors
            .as_ref()
            .is_some_and(|e| e[0].msg.starts_with("cannot read \"tests/lib/missing\": "))
    );
}
