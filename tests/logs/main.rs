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
fn compile_error_fns_not_found_with_unknown_return_type() -> Result<(), Error> {
    compile_and_check_logs(Path::new(
        "tests/logs/error_fns_not_found_with_unknown_return_type",
    ))
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
fn compile_error_operator_fns() -> Result<(), Error> {
    compile_and_check_logs(Path::new("tests/logs/error_operator_fns"))
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

fn compile_and_check_logs(path: &Path) -> Result<(), Error> {
    let (generated_dir, case_names) = common::generate_cases(path)?;
    let logs = gpex::compile_program(&generated_dir, true)
        .err()
        .unwrap_or_default();
    if path.join(".expected__$$").exists() {
        assert_logs_per_case(&generated_dir, &case_names, &logs)
    } else {
        assert_logs_globally(path, &generated_dir, &case_names, &logs)
    }
}

fn assert_logs_per_case(
    generated_dir: &Path,
    case_names: &[String],
    logs: &[Log],
) -> Result<(), Error> {
    for case_name in case_names {
        let expected_path = generated_dir.join(format!(".expected__{case_name}"));
        let actual = logs
            .iter()
            .filter(|log| is_log_related_to_case(log, case_name))
            .map(Log::to_string)
            .join("")
            .replace(&generated_dir.display().to_string(), "<root>");
        let expected = fs::read_to_string(&expected_path).map_err(Error::Io)?;
        assert_eq!(actual.trim(), expected.trim());
    }
    Ok(())
}

fn assert_logs_globally(
    path: &Path,
    generated_dir: &Path,
    case_names: &[String],
    logs: &[Log],
) -> Result<(), Error> {
    let actual = logs
        .iter()
        .map(Log::to_string)
        .join("")
        .replace(&generated_dir.display().to_string(), "<root>");
    let expected_path = path.join(".expected");
    if expected_path.exists() {
        let expected = fs::read_to_string(&expected_path).map_err(Error::Io)?;
        assert_eq!(actual.trim(), expected.trim());
    } else {
        fs::write(&expected_path, actual).map_err(Error::Io)?;
        panic!("expected logs saved on disk in {}", expected_path.display());
    }
    for case_name in case_names {
        assert!(
            contains_case_path(&actual, case_name),
            "'{case_name}' case did not generate any error"
        );
    }
    Ok(())
}

fn is_log_related_to_case(log: &Log, case_name: &str) -> bool {
    log.location
        .as_ref()
        .is_some_and(|location| contains_case_path(&location.path.to_string_lossy(), case_name))
}

fn contains_case_path(string: &str, case_name: &str) -> bool {
    string.contains(&format!("__{case_name}.gpex")) || string.contains(&format!("__{case_name}/"))
}
