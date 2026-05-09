use gpex::Log;
use itertools::Itertools;
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

const CASE_KEY_PLACEHOLDER: &str = "$$";
const EXCLUDE_TAG: &str = "<EXCLUDE>";

#[derive(Debug)]
#[expect(dead_code)] // variant data is useful for debugging tests
pub(crate) enum Error {
    Io(io::Error),
    Regex(regex::Error),
    Yaml(serde_norway::Error),
    Gpex(Vec<Log>),
    Other(Box<dyn std::error::Error>),
}

#[derive(Debug, Deserialize)]
struct Cases {
    dimensions: Vec<Dimension>,
}

#[derive(Debug, Deserialize)]
struct Dimension {
    id: String,
    cases: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone)]
#[derive_where::derive_where(PartialEq, Eq, Hash)]
struct DimensionCase {
    dimension: String,
    name: String,
    #[derive_where(skip)]
    key_values: HashMap<String, String>,
}

pub(crate) fn generate_cases(path: &Path) -> Result<(PathBuf, Vec<String>), Error> {
    let cases_path = path.join("cases.yaml");
    if cases_path.exists() {
        let test_dir = env::temp_dir().join(path);
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).map_err(Error::Io)?;
        }
        let cases_file = File::open(cases_path).map_err(Error::Io)?;
        let cases: Cases = serde_norway::from_reader(cases_file).map_err(Error::Yaml)?;
        let case_combinations = generate_case_combinations(&cases);
        let included_case_combinations = generate_dir(path, &case_combinations)?;
        let case_names = included_case_combinations
            .into_iter()
            .unique()
            .map(case_name)
            .collect();
        Ok((test_dir, case_names))
    } else {
        Ok((path.to_path_buf(), vec![]))
    }
}

fn generate_case_combinations(cases: &Cases) -> Vec<Vec<DimensionCase>> {
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
        .collect()
}

fn generate_dir<'a>(
    current_path: &Path,
    cases: &'a [Vec<DimensionCase>],
) -> Result<Vec<&'a [DimensionCase]>, Error> {
    let mut included_case_combinations = vec![];
    for entry in current_path.read_dir().map_err(Error::Io)? {
        let entry = entry.map_err(Error::Io)?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(Error::Io)?;
        if file_type.is_dir() {
            included_case_combinations.extend(generate_dir(&path, cases)?);
        } else if path.extension() == Some(OsStr::new("gpex")) {
            included_case_combinations.extend(generate_file(&path, cases)?);
        }
    }
    Ok(included_case_combinations)
}

fn generate_file<'a>(
    file_path: &Path,
    cases: &'a [Vec<DimensionCase>],
) -> Result<Vec<&'a [DimensionCase]>, Error> {
    if !is_path_containing(file_path, CASE_KEY_PLACEHOLDER) {
        // A file containing a placeholder should have a $$ placeholder in its path.
        // If not, an error will be triggered as the placeholders will not be replaced.
        // This helps to avoid case files being unexpectedly erased by another case.
        let output_path = env::temp_dir().join(file_path);
        fs::create_dir_all(path_parent(&output_path)).map_err(Error::Io)?;
        fs::copy(file_path, &output_path).map_err(Error::Io)?;
        return Ok(vec![]);
    }
    let content = fs::read_to_string(file_path).map_err(Error::Io)?;
    cases
        .iter()
        .map(|case| replace_placeholders(&content, case))
        .filter(|(_, generated_content)| !generated_content.contains(EXCLUDE_TAG))
        .map(|(case, generated_content)| save_generated_file(case, file_path, generated_content)?)
        .collect::<Result<_, _>>()
}

fn replace_placeholders<'a>(
    content: &str,
    case: &'a [DimensionCase],
) -> (&'a [DimensionCase], String) {
    let mut generated_content = content.to_string();
    for dimension in case {
        for (key, value) in &dimension.key_values {
            generated_content = generated_content
                .replace(&format!("{{{{{}.{}}}}}", dimension.dimension, key), value);
        }
    }
    (case, generated_content)
}

fn save_generated_file<'a>(
    case: &'a [DimensionCase],
    file_path: &Path,
    generated_content: String,
) -> Result<Result<&'a [DimensionCase], Error>, Error> {
    let case_name = case_name(case);
    let generated_content = generated_content.replace(CASE_KEY_PLACEHOLDER, &case_name);
    let output_path =
        env::temp_dir().join(replace_in_path(file_path, CASE_KEY_PLACEHOLDER, &case_name));
    fs::create_dir_all(path_parent(&output_path)).map_err(Error::Io)?;
    fs::write(&output_path, &generated_content).map_err(Error::Io)?;
    Ok(Ok(case))
}

fn case_name(case: &[DimensionCase]) -> String {
    case.iter().map(|dimension| &dimension.name).join("__")
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
