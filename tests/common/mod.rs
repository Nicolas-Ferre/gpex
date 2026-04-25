use gpex::Log;
use itertools::{Itertools, MultiProduct};
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::vec::IntoIter;
use std::{env, fs, io};

const CASE_KEY_PLACEHOLDER: &str = "$$";

#[derive(Debug)]
#[expect(dead_code)] // variant data is useful for debugging tests
pub(crate) enum Error {
    Io(io::Error),
    Regex(regex::Error),
    Yaml(serde_norway::Error),
    Gpex(Vec<Log>),
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

pub(crate) fn generate_cases(path: &Path) -> Result<(PathBuf, Vec<String>), Error> {
    let cases_path = path.join("cases.yaml");
    if cases_path.exists() {
        let test_dir = env::temp_dir().join(path);
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).map_err(Error::Io)?;
        }
        let cases_file = File::open(cases_path).map_err(Error::Io)?;
        let cases: Cases = serde_norway::from_reader(cases_file).map_err(Error::Yaml)?;
        let case_combinations = case_combinations(&cases)
            .filter(|combination| !is_case_combination_excluded(&cases, combination))
            .collect::<Vec<_>>();
        generate_dir(path, &case_combinations)?;
        let case_names = case_combinations
            .iter()
            .map(|combination| case_name(combination))
            .collect();
        Ok((test_dir, case_names))
    } else {
        Ok((path.to_path_buf(), vec![]))
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
        if exclusion.iter().all(|(dimension, case_names)| {
            case_names.contains(find_case_from_dimension(combination, dimension))
        }) {
            return true;
        }
    }
    false
}

fn find_case_from_dimension<'case>(
    combination: &'case [DimensionCase],
    dimension: &str,
) -> &'case String {
    &combination
        .iter()
        .find(|case| case.dimension == dimension)
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
        // A file containing a placeholder should have a $$ placeholder in its path.
        // If not, an error will be triggered as the placeholders will not be replaced.
        // This helps to avoid case files being unexpectedly erased by another case.
        let output_path = env::temp_dir().join(file_path);
        fs::create_dir_all(path_parent(&output_path)).map_err(Error::Io)?;
        fs::copy(file_path, &output_path).map_err(Error::Io)?;
        return Ok(());
    }
    let content = fs::read_to_string(file_path).map_err(Error::Io)?;
    for case in cases {
        let mut generated_content = content.clone();
        for dimension in case {
            for (key, value) in &dimension.key_values {
                generated_content = generated_content
                    .replace(&format!("{{{{{}.{}}}}}", dimension.dimension, key), value);
            }
        }
        let case_name = case_name(case);
        generated_content = generated_content.replace(CASE_KEY_PLACEHOLDER, &case_name);
        let output_path =
            env::temp_dir().join(replace_in_path(file_path, CASE_KEY_PLACEHOLDER, &case_name));
        fs::create_dir_all(path_parent(&output_path)).map_err(Error::Io)?;
        fs::write(&output_path, &generated_content).map_err(Error::Io)?;
    }
    Ok(())
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
