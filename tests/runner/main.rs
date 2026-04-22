//! Tests for runner.

use gpex::{Log, Runner};
use itertools::{Itertools, MultiProduct};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::vec::IntoIter;
use std::{env, fs, io};

const CASE_KEY_PLACEHOLDER: &str = "$$";

#[tokio::test]
async fn run_empty() -> Result<(), Error> {
    run_test_folder(Path::new("tests/runner/empty")).await
}

#[tokio::test]
async fn run_expr_sources() -> Result<(), Error> {
    run_test_folder(Path::new("tests/runner/expr_sources")).await
}

#[tokio::test]
async fn run_exprs() -> Result<(), Error> {
    run_test_folder(Path::new("tests/runner/exprs")).await
}

#[tokio::test]
async fn run_fn_param_aliasing() -> Result<(), Error> {
    run_test_folder(Path::new("tests/runner/fn_param_aliasing")).await
}

#[tokio::test]
async fn run_fn_aliasing() -> Result<(), Error> {
    run_test_folder(Path::new("tests/runner/fn_aliasing")).await
}

#[tokio::test]
async fn run_imports() -> Result<(), Error> {
    run_test_folder(Path::new("tests/runner/imports")).await
}

#[tokio::test]
async fn run_import_priority() -> Result<(), Error> {
    run_test_folder(Path::new("tests/runner/import_priority")).await
}

#[tokio::test]
async fn run_literals() -> Result<(), Error> {
    run_test_folder(Path::new("tests/runner/literals")).await
}

#[tokio::test]
async fn run_prelude_types() -> Result<(), Error> {
    run_test_folder(Path::new("tests/runner/prelude_types")).await
}

#[tokio::test]
async fn run_repeats() -> Result<(), Error> {
    run_test_folder(Path::new("tests/runner/repeats")).await
}

#[tokio::test]
async fn run_syntax() -> Result<(), Error> {
    run_test_folder(Path::new("tests/runner/syntax")).await
}

async fn run_test_folder(path: &Path) -> Result<(), Error> {
    let generated_dir = generate_cases(path)?;
    let (program, _) = gpex::compile_program(&generated_dir, false).map_err(Error::Gpex)?;
    let mut runner = Runner::new(program).await.map_err(Error::Gpex)?;
    runner.run_step();
    check_global_vars(&generated_dir, &generated_dir, &runner)?;
    Ok(())
}

fn generate_cases(path: &Path) -> Result<PathBuf, Error> {
    let cases_file = path.join("cases.yaml");
    if cases_file.exists() {
        let test_dir = env::temp_dir().join(path);
        _ = fs::remove_dir_all(&test_dir);
        let cases_file = File::open(cases_file).map_err(Error::Io)?;
        let cases: Cases = serde_yml::from_reader(cases_file).map_err(Error::Yaml)?;
        let case_combinations = case_combinations(&cases)
            .filter(|combination| !is_case_combination_excluded(&cases, combination))
            .collect::<Vec<_>>();
        generate_dir(path, &case_combinations)?;
        Ok(test_dir)
    } else {
        Ok(path.to_path_buf())
    }
}

fn case_combinations(cases: &Cases) -> MultiProduct<IntoIter<DimensionCase>> {
    cases
        .dimensions
        .iter()
        .map(|dimension| {
            dimension
                .cases
                .iter()
                .map(|(key, value)| DimensionCase {
                    dimension: dimension.id.clone(),
                    name: key.clone(),
                    key_values: value.clone(),
                })
                .collect::<Vec<_>>()
        })
        .multi_cartesian_product()
}

fn is_case_combination_excluded(cases: &Cases, combination: &[DimensionCase]) -> bool {
    for exclusion in &cases.exclusions {
        if exclusion.iter().all(|(dimension, cases)| {
            cases.contains(find_case_from_dimension(combination, dimension))
        }) {
            return true;
        }
    }
    false
}

fn find_case_from_dimension<'a>(
    combination: &'a [DimensionCase],
    dimension: &String,
) -> &'a String {
    &combination
        .iter()
        .find(|case| &case.dimension == dimension)
        .unwrap_or_else(|| panic!("cases.yaml: '{dimension}' dimension not found"))
        .name
}

fn generate_dir(current_path: &Path, cases: &[Vec<DimensionCase>]) -> Result<(), Error> {
    for entry in current_path.read_dir().map_err(Error::Io)? {
        let entry = entry.map_err(Error::Io)?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(Error::Io)?;
        if file_type.is_dir() {
            generate_dir(&path, cases)?;
        } else if path.extension() == Some(OsStr::new("gpex")) {
            generate_file(&path, cases)?;
        }
    }
    Ok(())
}

fn generate_file(file_path: &Path, cases: &[Vec<DimensionCase>]) -> Result<(), Error> {
    if !is_path_containing(file_path, CASE_KEY_PLACEHOLDER) {
        let output_path = env::temp_dir().join(file_path);
        // Helps to detect missing case placeholder in case the file includes key placeholders.
        fs::create_dir_all(path_parent(&output_path)).map_err(Error::Io)?;
        fs::copy(file_path, &output_path).map_err(Error::Io)?;
        return Ok(());
    }
    let placeholder_regex = Regex::new(r"\{\{.*}}").map_err(Error::Regex)?;
    for case in cases {
        let case_name = case.iter().map(|dimension| &dimension.name).join("__");
        let mut content = fs::read_to_string(file_path)
            .map_err(Error::Io)?
            .replace(CASE_KEY_PLACEHOLDER, &case_name);
        for dimension in case {
            for (key, value) in &dimension.key_values {
                content =
                    content.replace(&format!("{{{{{}.{}}}}}", dimension.dimension, key), value);
            }
        }
        let output_path =
            env::temp_dir().join(replace_in_path(file_path, CASE_KEY_PLACEHOLDER, &case_name));
        content = placeholder_regex.replace_all(&content, "").to_string();
        fs::create_dir_all(path_parent(&output_path)).map_err(Error::Io)?;
        fs::write(&output_path, &content).map_err(Error::Io)?;
    }
    Ok(())
}

fn path_parent(path: &Path) -> &Path {
    path.parent()
        .unwrap_or_else(|| unreachable!("parent should be at least tmp folder"))
}

fn is_path_containing(file_path: &Path, pattern: &str) -> bool {
    file_path.iter().any(|component| {
        component
            .to_str()
            .unwrap_or_else(|| unreachable!("invalid test file path"))
            .contains(pattern)
    })
}

fn replace_in_path(file_path: &Path, pattern: &str, replacement: &str) -> PathBuf {
    file_path
        .iter()
        .map(|component| {
            component
                .to_str()
                .unwrap_or_else(|| unreachable!("invalid test file path"))
                .replace(pattern, replacement)
        })
        .collect::<PathBuf>()
}

fn check_global_vars(dir_path: &Path, root_path: &Path, runner: &Runner) -> Result<(), Error> {
    let expected_regex = Regex::new(r"var *(\w+) *=.*// expected: *(.+)").map_err(Error::Regex)?;
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
                let expected_value = &capture[2];
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

#[derive(Debug, Deserialize)]
struct Cases {
    dimensions: Vec<Dimension>,
    #[serde(default)]
    exclusions: Vec<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Deserialize)]
struct Dimension {
    id: String,
    cases: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone)]
struct DimensionCase {
    dimension: String,
    name: String,
    key_values: HashMap<String, String>,
}

#[derive(Debug)]
#[expect(dead_code)] // variant data is useful for debugging tests
enum Error {
    Io(io::Error),
    Regex(regex::Error),
    Yaml(serde_yml::Error),
    Gpex(Vec<Log>),
}
