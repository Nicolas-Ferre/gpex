//! Tests for runner.

#[path = "../common/mod.rs"]
mod common;

use crate::common::Error;
use gpex::Runner;
use itertools::Itertools;
use naga::back::wgsl as wgsl_back;
use naga::front::wgsl;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use regex::Regex;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

#[tokio::test]
async fn run_const_optimizations() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/const_optimizations"), true, true).await
}

#[tokio::test]
async fn run_empty() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/empty"), true, true).await
}

#[tokio::test]
async fn run_expr_sources() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/expr_sources"), false, false).await
}

#[tokio::test]
async fn run_exprs() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/exprs"), false, false).await
}

#[tokio::test]
async fn run_fn_param_aliasing() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/fn_param_aliasing"), false, false).await
}

#[tokio::test]
async fn run_fn_aliasing() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/fn_aliasing"), false, false).await
}

#[tokio::test]
async fn run_imports() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/imports"), false, true).await
}

#[tokio::test]
async fn run_item_naming() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/item_naming"), false, true).await
}

#[tokio::test]
async fn run_import_priority() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/import_priority"), false, true).await
}

#[tokio::test]
async fn run_literals() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/literals"), true, true).await
}

#[tokio::test]
async fn run_prelude_types() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/prelude_types"), true, true).await
}

#[tokio::test]
async fn run_repeats() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/repeats"), false, true).await
}

#[tokio::test]
async fn run_syntax() -> Result<(), Error> {
    run_test_dir(Path::new("tests/runner/syntax"), false, true).await
}

async fn run_test_dir(
    path: &Path,
    save_code: bool,
    is_warning_treated_as_error: bool,
) -> Result<(), Error> {
    let (generated_dir, _) = common::generate_cases(path)?;
    let (program, _) =
        gpex::compile_program(&generated_dir, is_warning_treated_as_error).map_err(Error::Gpex)?;
    let actual_code = format!(
        "// INIT SHADER\n\n{}\n\n// UPDATE SHADER\n\n{}\n",
        format_wgsl(&program.init_shader)?,
        format_wgsl(&program.update_shader)?
    );
    let mut runner = Runner::new(program).await.map_err(Error::Gpex)?;
    runner.run_step();
    check_global_vars(&generated_dir, &generated_dir, &runner)?;
    if !save_code {
        return Ok(());
    }
    let expected_path = path.join(".expected");
    if let Ok(expected_code) = fs::read_to_string(&expected_path) {
        assert_eq!(actual_code, expected_code);
    } else {
        fs::write(expected_path, actual_code).map_err(Error::Io)?;
        panic!("expected transpiled code saved on disk");
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
    let module = wgsl::parse_str(code)
        .map_err(|err| Box::new(err) as _)
        .map_err(Error::Other)?;
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    let info = validator
        .validate(&module)
        .map_err(|err| Box::new(err) as _)
        .map_err(Error::Other)?;
    let mut output = String::new();
    let mut writer = wgsl_back::Writer::new(&mut output, wgsl_back::WriterFlags::empty());
    writer
        .write(&module, &info)
        .map_err(|err| Box::new(err) as _)
        .map_err(Error::Other)?;
    Ok(output)
}
