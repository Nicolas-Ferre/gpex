//! Tests for generated compilation logs.

#[path = "../common/mod.rs"]
mod common;

use crate::common::Error;
use gpex::Log;
use itertools::Itertools;
use std::fs;
use std::path::Path;

#[test]
fn compile_error_exprs_non_const() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_exprs_non_const"))
}

#[test]
fn compile_error_exprs_not_ref() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_exprs_not_ref"))
}

#[test]
fn compile_error_fns_without_return_type() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_fns_without_return_type"))
}

#[test]
fn compile_error_imports_not_found() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_imports_not_found"))
}

#[test]
fn compile_error_imports_not_found_outside_project() -> Result<(), Error> {
    compile_and_check_logs(Path::new(
        "tests/logs/error_imports_not_found_outside_project",
    ))
}

#[test]
fn compile_error_items_circular() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_items_circular"))
}

#[test]
fn compile_error_items_disallowed() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_items_disallowed"))
}

#[test]
fn compile_error_multiple_definitions() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_multiple_definitions"))
}

#[test]
fn compile_error_items_not_found_after_ref() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_items_not_found_after_ref"))
}

#[test]
fn compile_error_items_not_found_before_ref() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_items_not_found_before_ref"))
}

#[test]
fn compile_error_items_not_found_parent() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_items_not_found_parent"))
}

#[test]
fn compile_error_items_not_found_priv_in_pub_import() -> Result<(), Error> {
    compile_and_check_logs(Path::new(
        "tests/logs/error_items_not_found_priv_in_pub_import",
    ))
}

#[test]
fn compile_error_items_not_found_pub_in_priv_import() -> Result<(), Error> {
    compile_and_check_logs(Path::new(
        "tests/logs/error_items_not_found_pub_in_priv_import",
    ))
}

#[test]
fn compile_error_out_of_bounds() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_out_of_bounds"))
}

#[test]
fn compile_error_statements_missing() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_statements_missing"))
}

#[test]
fn compile_error_syntax() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_syntax"))
}

#[test]
fn compile_error_type_comparison() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_type_comparison"))
}

#[test]
fn compile_warning_empty() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/warning_empty"))
}

#[test]
fn compile_warning_naming_case() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/warning_naming_case"))
}

#[test]
fn compile_warning_naming_single_letter() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/warning_naming_single_letter"))
}

#[test]
fn compile_warning_unused() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/warning_unused"))
}

#[test]
fn compile_warning_used_with_underscore_prefix() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/warning_used_with_underscore_prefix"))
}

fn compile_and_check_logs(path: &Path) -> Result<(), Error> {
    let (generated_dir, case_names) = common::generate_cases(path)?;
    let logs = gpex::compile_program(&generated_dir, true)
        .err()
        .unwrap_or_default();
    let actual = logs
        .iter()
        .map(Log::to_string)
        .join("")
        .replace(&generated_dir.display().to_string(), "<root>");
    let expected_path = path.join(".expected");
    if expected_path.exists() {
        let expected = fs::read_to_string(&expected_path).map_err(Error::Io)?;
        assert_eq!(actual, expected);
    } else {
        fs::write(&expected_path, actual).map_err(Error::Io)?;
        panic!("expected logs saved on disk in {}", expected_path.display());
    }
    for case_name in &case_names {
        assert!(
            actual.contains(&format!("__{case_name}.gpex"))
                || actual.contains(&format!("__{case_name}/")),
            "'{case_name}' case did not generate any error"
        );
    }
    Ok(())
}
