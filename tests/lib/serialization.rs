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
        let _ = fs::remove_file(out_path);
    }
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn save_in_non_existing_folder() -> Result<(), Vec<Log>> {
    let (program, _) = gpex::compile(Path::new("tests/lib/valid"), false)?;
    let result = gpex::save_compiled(&program, Path::new("tests/missing/out.json"));
    let errors = result.err();
    assert_eq!(errors.as_ref().map(Vec::len), Some(1));
    assert_eq!(errors.as_ref().map(|e| e[0].level), Some(LogLevel::Error));
    assert!(errors.as_ref().is_some_and(|e| e[0].loc.is_none()));
    assert_eq!(errors.as_ref().map(|e| e[0].inner.len()), Some(0));
    assert!(errors.as_ref().is_some_and(|e| {
        e[0].msg
            .as_str()
            .starts_with("cannot write \"tests/missing/out.json\": ")
    }));
    Ok(())
}

#[test]
fn load_non_existing_file() -> Result<(), Vec<Log>> {
    let (program, _) = gpex::compile(Path::new("tests/lib/valid"), false)?;
    let out_path = Path::new("tests/lib/out2.json");
    gpex::save_compiled(&program, out_path)?;
    let result = gpex::load_compiled(Path::new("tests/missing/out.json"));
    if out_path.is_file() {
        let _ = fs::remove_file(out_path);
    }
    let errors = result.err();
    assert_eq!(errors.as_ref().map(Vec::len), Some(1));
    assert_eq!(errors.as_ref().map(|e| e[0].level), Some(LogLevel::Error));
    assert!(errors.as_ref().is_some_and(|e| e[0].loc.is_none()));
    assert_eq!(errors.as_ref().map(|e| e[0].inner.len()), Some(0));
    assert!(errors.as_ref().is_some_and(|e| {
        e[0].msg
            .as_str()
            .starts_with("cannot read \"tests/missing/out.json\": ")
    }));
    Ok(())
}

#[test]
fn load_invalid_file() {
    let result = gpex::load_compiled(Path::new("tests/lib/main.rs"));
    let errors = result.err();
    assert_eq!(errors.as_ref().map(Vec::len), Some(1));
    assert_eq!(errors.as_ref().map(|e| e[0].level), Some(LogLevel::Error));
    assert!(errors.as_ref().is_some_and(|e| e[0].loc.is_none()));
    assert_eq!(errors.as_ref().map(|e| e[0].inner.len()), Some(0));
    assert_eq!(
        errors.as_ref().map(|e| e[0].msg.as_str()),
        Some("invalid compiled program \"tests/lib/main.rs\"")
    );
}
