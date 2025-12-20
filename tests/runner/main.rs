//! Tests for runner.

use gpex::{Log, Runner};
use itertools::Itertools;
use regex::Regex;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{fs, io};

#[tokio::test]
async fn run_with_exprs() -> Result<(), Error> {
    compile_and_run(Path::new("tests/runner/exprs")).await
}

#[tokio::test]
async fn run_with_syntax_specificities() -> Result<(), Error> {
    compile_and_run(Path::new("tests/runner/syntax")).await
}

async fn compile_and_run(path: &Path) -> Result<(), Error> {
    let (program, _) = gpex::compile(path, true).map_err(Error::Gpex)?;
    let mut runner = Runner::new(program).await.map_err(Error::Gpex)?;
    runner.run_step();
    check_global_vars(path, path, &runner)?;
    Ok(())
}

fn check_global_vars(folder_path: &Path, root_path: &Path, runner: &Runner) -> Result<(), Error> {
    let expected_regex = Regex::new(r"var (\w+) = .* // expected: (.+)").map_err(Error::Regex)?;
    let ignored_regex = Regex::new(r"var .* // ignore").map_err(Error::Regex)?;
    let var_regex = Regex::new(r"var .*").map_err(Error::Regex)?;
    for entry in folder_path.read_dir().map_err(Error::Io)? {
        let entry = entry.map_err(Error::Io)?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(Error::Io)?;
        if file_type.is_dir() {
            check_global_vars(&path, root_path, runner)?;
        } else if path.extension() == Some(OsStr::new("gpex")) {
            let code = fs::read_to_string(&path).map_err(Error::Io)?;
            let var_count = var_regex.find_iter(&code).count();
            let ignored_count = ignored_regex.find_iter(&code).count();
            let expected_count = expected_regex.find_iter(&code).count();
            assert_eq!(var_count, expected_count + ignored_count);
            let dot_path = to_dot_path(&path, root_path);
            for capture in expected_regex.captures_iter(&code) {
                let var_name = &capture[1];
                let expected_value = &capture[2];
                let var_path = format!("{dot_path}:{var_name}");
                let actual_value = runner.read_var(&var_path);
                assert_eq!(
                    Some(expected_value.into()),
                    actual_value.map(|v| v.to_string()),
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
        .map(|c| c.to_str().unwrap_or("<invalid>"))
        .join(".")
}

#[derive(Debug)]
#[allow(dead_code)] // variant data is useful for debugging tests
enum Error {
    Io(io::Error),
    Regex(regex::Error),
    Gpex(Vec<Log>),
}
