//! Tests for generated compilation logs.

#[path = "../common/mod.rs"]
mod common;

use crate::common::Error;
use gpex::Log;
use itertools::Itertools;
use std::fs;
use std::path::Path;

// TODO: rename test names according to hierarchy

#[test]
fn compile_with_circular_items_errors() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_circular_items"))
}

#[test]
fn compile_with_disallowed_item_errors() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_disallowed_items"))
}

#[test]
fn compile_with_missing_statement_errors() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_missing_statements"))
}

#[test]
fn compile_with_multiple_definition_errors() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_multiple_definitions"))
}

#[test]
fn compile_with_non_const_expr_errors() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_non_const_exprs"))
}

#[test]
fn compile_with_not_found_import_errors() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_not_found_imports"))
}

#[test]
fn compile_with_not_found_item_after_ref_errors() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_not_found_items_after_ref"))
}

#[test]
fn compile_with_not_found_item_before_ref_errors() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_not_found_items_before_ref"))
}

#[test]
fn compile_with_not_found_parent_item_errors() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_not_found_parent_items"))
}

#[test]
fn compile_with_not_found_priv_item_in_pub_import_errors() -> Result<(), Error> {
    compile_and_check_logs(Path::new(
        "tests/logs/error_not_found_priv_item_in_pub_import",
    ))
}

#[test]
fn compile_with_not_found_pub_item_in_priv_import_errors() -> Result<(), Error> {
    compile_and_check_logs(Path::new(
        "tests/logs/error_not_found_pub_item_in_priv_import",
    ))
}

#[test]
fn compile_with_not_ref_expr_errors() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_not_ref_exprs"))
}

#[test]
fn compile_with_out_of_bounds_errors() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_out_of_bounds"))
}

#[test]
fn compile_with_syntax_errors() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_syntax"))
}

#[test]
fn compile_with_type_errors() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_type_comparison"))
}

#[test]
fn compile_with_empty_warnings() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/warning_empty"))
}

#[test]
fn compile_with_naming_case_warnings() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/warning_naming_case"))
}

#[test]
fn compile_with_naming_single_letter_warnings() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/warning_naming_single_letter"))
}

#[test]
fn compile_with_unused_warnings() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/warning_unused"))
}

#[test]
fn compile_with_used_with_underscore_prefix_warnings() -> Result<(), Error> {
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
    if let Ok(expected) = fs::read_to_string(&expected_path) {
        assert_eq!(actual, expected);
    } else {
        fs::write(expected_path, actual).map_err(Error::Io)?;
        panic!("expected logs saved on disk");
    }
    for case_name in &case_names {
        assert!(
            actual.contains(case_name),
            "'{case_name}' case didn't generate any error"
        );
    }
    Ok(())
}
