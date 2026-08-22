use crate::files;
use gpex::Runner;
use libtest_mimic::Failed;
use pretty_assertions::assert_eq;
use regex::Regex;
use std::fmt::Write;
use std::fs;
use std::path::Path;

pub(crate) const EXPECTED_VAR_REGEX: &str = r"var +(\w+) *= *[^;]*; *// expected: *(.+)";

pub(crate) struct ExpectedVar {
    path: String,
    pub(crate) values: Vec<String>,
}

pub(crate) fn collect(dir_path: &Path, root_path: &Path) -> Result<Vec<ExpectedVar>, Failed> {
    let mut expected_vars = vec![];
    let expected_regex = Regex::new(EXPECTED_VAR_REGEX)?;
    for entry in dir_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            expected_vars.extend(collect(&path, root_path)?);
        } else if files::is_file_gpex(&path) {
            let code = fs::read_to_string(&path)?;
            let dot_path = files::to_dot_path(&path, root_path);
            for capture in expected_regex.captures_iter(&code) {
                let var_name = &capture[1];
                expected_vars.push(ExpectedVar {
                    path: format!("{dot_path}:{var_name}"),
                    values: capture[2]
                        .split(',')
                        .map(|value| value.trim().to_string())
                        .collect(),
                });
            }
        }
    }
    Ok(expected_vars)
}

pub(crate) fn assert(
    expected_vars: &[ExpectedVar],
    frame_index: usize,
    runner: &Runner,
) -> Result<(), Failed> {
    let frame_number = frame_index + 1;
    let mut all_actual = format!("frame {frame_number}:\n");
    let mut all_expected = format!("frame {frame_number}:\n");
    for expected_var in expected_vars {
        if let Some(expected_value) = expected_var.values.get(frame_index) {
            let var_path = &expected_var.path;
            let actual_value = runner
                .read_var(var_path)
                .map_or_else(|| "<unknown>".into(), |value| value.to_string());
            writeln!(all_actual, "{var_path}={actual_value}")?;
            writeln!(all_expected, "{var_path}={expected_value}")?;
        }
    }
    assert_eq!(all_expected, all_actual);
    Ok(())
}
