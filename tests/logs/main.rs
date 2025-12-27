//! Tests for generated compilation logs.

use gpex::Log;
use itertools::Itertools;
use std::path::Path;
use std::{fs, io};

#[test]
fn compile_with_syntax_errors() -> io::Result<()> {
    compile_and_check_logs(Path::new("tests/logs/error_syntax"))
}

#[test]
fn compile_with_circular_dependency_errors() -> io::Result<()> {
    compile_and_check_logs(Path::new("tests/logs/error_circular_dependencies"))
}

#[test]
fn compile_with_disallowed_item_errors() -> io::Result<()> {
    compile_and_check_logs(Path::new("tests/logs/error_disallowed_items"))
}

#[test]
fn compile_with_not_found_item_errors() -> io::Result<()> {
    compile_and_check_logs(Path::new("tests/logs/error_not_found_items"))
}

#[test]
fn compile_with_out_of_bounds_errors() -> io::Result<()> {
    compile_and_check_logs(Path::new("tests/logs/error_out_of_bounds"))
}

#[test]
fn compile_with_multiple_definitions_errors() -> io::Result<()> {
    compile_and_check_logs(Path::new("tests/logs/error_multiple_definitions"))
}

#[test]
fn compile_with_const_errors() -> io::Result<()> {
    compile_and_check_logs(Path::new("tests/logs/error_const"))
}

#[test]
fn compile_with_unused_warnings() -> io::Result<()> {
    compile_and_check_logs(Path::new("tests/logs/warning_unused"))
}

#[test]
fn compile_with_naming_warnings() -> io::Result<()> {
    compile_and_check_logs(Path::new("tests/logs/warning_naming"))
}

fn compile_and_check_logs(path: &Path) -> io::Result<()> {
    let logs = gpex::compile(path, true).err().unwrap_or_default();
    let actual = logs.iter().map(Log::to_string).join("");
    let expected_path = path.join(".expected");
    if let Ok(expected) = fs::read_to_string(&expected_path) {
        assert_eq!(actual, expected);
    } else {
        fs::write(expected_path, actual)?;
        panic!("expected logs saved on disk");
    }
    Ok(())
}
