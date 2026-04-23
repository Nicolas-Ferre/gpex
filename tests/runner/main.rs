//! Tests for runner.

#[path = "../common/mod.rs"]
mod common;

use crate::common::Error;
use gpex::Runner;
use itertools::Itertools;
use regex::Regex;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

#[tokio::test]
async fn run_empty() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/empty")).await
}

#[tokio::test]
async fn run_expr_sources() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/expr_sources")).await
}

#[tokio::test]
async fn run_exprs() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/exprs")).await
}

#[tokio::test]
async fn run_fn_param_aliasing() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/fn_param_aliasing")).await
}

#[tokio::test]
async fn run_fn_aliasing() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/fn_aliasing")).await
}

#[tokio::test]
async fn run_imports() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/imports")).await
}

#[tokio::test]
async fn run_import_priority() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/import_priority")).await
}

#[tokio::test]
async fn run_literals() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/literals")).await
}

#[tokio::test]
async fn run_prelude_types() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/prelude_types")).await
}

#[tokio::test]
async fn run_repeats() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/repeats")).await
}

#[tokio::test]
async fn run_syntax() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/syntax")).await
}

async fn run_test_dir(path: &Path) -> Result<(), Error> {
    let generated_dir = common::generate_cases(path)?;
    let (program, _) = gpex::compile_program(&generated_dir, false).map_err(Error::Gpex)?;
    let mut runner = Runner::new(program).await.map_err(Error::Gpex)?;
    runner.run_step();
    check_global_vars(&generated_dir, &generated_dir, &runner)?;
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
