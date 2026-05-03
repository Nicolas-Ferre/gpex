//! Tests for runner.

#[path = "../common/mod.rs"]
mod common;

use crate::common::Error;
use gpex::{Program, Runner};
use itertools::Itertools;
use regex::Regex;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use wgsl_parse::syntax::TranslationUnit;

#[tokio::test]
async fn run_const_optimizations() -> Result<(), Error> {
    run_test_dir(
        Path::new("tests/runner/const_optimizations"),
        TranspiledCodeAction::Checked,
        WarningAction::Failing,
    )
    .await
}

#[tokio::test]
async fn run_empty() -> Result<(), Error> {
    run_test_dir(
        Path::new("tests/runner/empty"),
        TranspiledCodeAction::Checked,
        WarningAction::Failing,
    )
    .await
}

#[tokio::test]
async fn run_expr_sources() -> Result<(), Error> {
    run_test_dir(
        Path::new("tests/runner/expr_sources"),
        TranspiledCodeAction::Ignored,
        WarningAction::Ignored,
    )
    .await
}

#[tokio::test]
async fn run_exprs() -> Result<(), Error> {
    run_test_dir(
        Path::new("tests/runner/exprs"),
        TranspiledCodeAction::Ignored,
        WarningAction::Ignored,
    )
    .await
}

#[tokio::test]
async fn run_fn_param_aliasing() -> Result<(), Error> {
    run_test_dir(
        Path::new("tests/runner/fn_param_aliasing"),
        TranspiledCodeAction::Ignored,
        WarningAction::Ignored,
    )
    .await
}

#[tokio::test]
async fn run_fn_aliasing() -> Result<(), Error> {
    run_test_dir(
        Path::new("tests/runner/fn_aliasing"),
        TranspiledCodeAction::Ignored,
        WarningAction::Ignored,
    )
    .await
}

#[tokio::test]
async fn run_imports() -> Result<(), Error> {
    run_test_dir(
        Path::new("tests/runner/imports"),
        TranspiledCodeAction::Ignored,
        WarningAction::Failing,
    )
    .await
}

#[tokio::test]
async fn run_item_naming() -> Result<(), Error> {
    run_test_dir(
        Path::new("tests/runner/item_naming"),
        TranspiledCodeAction::Ignored,
        WarningAction::Failing,
    )
    .await
}

#[tokio::test]
async fn run_import_priority() -> Result<(), Error> {
    run_test_dir(
        Path::new("tests/runner/import_priority"),
        TranspiledCodeAction::Ignored,
        WarningAction::Failing,
    )
    .await
}

#[tokio::test]
async fn run_literals() -> Result<(), Error> {
    run_test_dir(
        Path::new("tests/runner/literals"),
        TranspiledCodeAction::Checked,
        WarningAction::Failing,
    )
    .await
}

#[tokio::test]
async fn run_prelude_types() -> Result<(), Error> {
    run_test_dir(
        Path::new("tests/runner/prelude_types"),
        TranspiledCodeAction::Checked,
        WarningAction::Failing,
    )
    .await
}

#[tokio::test]
async fn run_repeats() -> Result<(), Error> {
    run_test_dir(
        Path::new("tests/runner/repeats"),
        TranspiledCodeAction::Ignored,
        WarningAction::Failing,
    )
    .await
}

#[tokio::test]
async fn run_syntax() -> Result<(), Error> {
    run_test_dir(
        Path::new("tests/runner/syntax"),
        TranspiledCodeAction::Ignored,
        WarningAction::Failing,
    )
    .await
}

async fn run_test_dir(
    path: &Path,
    transpiled_code_action: TranspiledCodeAction,
    warning_action: WarningAction,
) -> Result<(), Error> {
    let (generated_dir, _) = common::generate_cases(path)?;
    let is_warning_treated_as_error = warning_action == WarningAction::Failing;
    let (program, _) =
        gpex::compile_program(&generated_dir, is_warning_treated_as_error).map_err(Error::Gpex)?;
    let mut runner = Runner::new(program).await.map_err(Error::Gpex)?;
    runner.run_step();
    check_global_vars(&generated_dir, &generated_dir, &runner)?;
    if transpiled_code_action == TranspiledCodeAction::Checked {
        check_transpiled_code(path, runner.program())?;
    }
    Ok(())
}

fn check_transpiled_code(path: &Path, program: &Program) -> Result<(), Error> {
    let actual_code = format!(
        "// INIT SHADER\n\n{}\n\n// UPDATE SHADER\n\n{}\n",
        format_wgsl(&program.init_shader)?,
        format_wgsl(&program.update_shader)?
    );
    let expected_path = path.join(".expected");
    if expected_path.exists() {
        let expected_code = fs::read_to_string(&expected_path).map_err(Error::Io)?;
        assert_eq!(actual_code, expected_code);
    } else {
        fs::write(&expected_path, actual_code).map_err(Error::Io)?;
        panic!(
            "expected transpiled code saved on disk in {}",
            expected_path.display()
        );
    }
    Ok(())
}

fn check_global_vars(dir_path: &Path, root_path: &Path, runner: &Runner) -> Result<(), Error> {
    let expected_regex = Regex::new(r"var +(\w+) *=.*// expected: *(.+)").map_err(Error::Regex)?;
    for entry in dir_path.read_dir().map_err(Error::Io)? {
        let entry = entry.map_err(Error::Io)?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(Error::Io)?;
        if file_type.is_dir() {
            check_global_vars(&path, root_path, runner)?;
        } else if path.extension() == Some(OsStr::new("gpex")) {
            let code = fs::read_to_string(&path).map_err(Error::Io)?;
            let dot_path = to_dot_path(&path, root_path);
            for capture in expected_regex.captures_iter(&code) {
                let var_name = &capture[1];
                let expected_value = capture[2].trim();
                let var_path = format!("{dot_path}:{var_name}");
                let actual_value = runner.read_var(&var_path);
                assert_eq!(
                    Some(expected_value.into()),
                    actual_value.map(|value| value.to_string()),
                    "`{var_path}` variable"
                );
            }
        }
    }
    Ok(())
}

fn to_dot_path(file_path: &Path, root_path: &Path) -> String {
    file_path
        .iter()
        .skip(root_path.iter().count())
        .collect::<PathBuf>()
        .with_extension("")
        .iter()
        .map(|segment| segment.to_str().unwrap_or("<invalid>"))
        .join(".")
}

fn format_wgsl(code: &str) -> Result<String, Error> {
    let module = TranslationUnit::from_str(code).map_err(convert_error)?;
    Ok(module.to_string())
}

fn convert_error(error: impl std::error::Error + 'static) -> Error {
    Error::Other(Box::new(error) as _)
}

#[derive(Debug, PartialEq, Eq)]
enum TranspiledCodeAction {
    Checked,
    Ignored,
}

#[derive(Debug, PartialEq, Eq)]
enum WarningAction {
    Failing,
    Ignored,
}
