use crate::expected_values::EXPECTED_VAR_REGEX;
use crate::files;
use libtest_mimic::Failed;
use pretty_assertions::assert_eq;
use regex::{Captures, Regex};
use std::path::{Path, PathBuf};
use std::{env, fs};

const EXPECTED_CONST_REGEX: &str = r"const +(\w+) *= *([^;]*); *// expected: *(.+)";
const EXPECTED_PATTERN: &str = "// expected";

pub(crate) fn generate(root_path: &Path) -> Result<PathBuf, Failed> {
    let test_dir = env::temp_dir().join(root_path);
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir)?;
    }
    fs::create_dir_all(&test_dir)?;
    generate_dir(root_path)?;
    Ok(test_dir)
}

fn generate_dir(dir_path: &Path) -> Result<(), Failed> {
    let expected_const_regex = Regex::new(EXPECTED_CONST_REGEX)?;
    let expected_var_regex = Regex::new(EXPECTED_VAR_REGEX)?;
    for entry in dir_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            generate_dir(&path)?;
        } else if files::is_file_gpex(&path) {
            generate_file(&path, &expected_const_regex, &expected_var_regex)?;
        }
    }
    Ok(())
}

fn generate_file(
    path: &Path,
    expected_const_regex: &Regex,
    expected_var_regex: &Regex,
) -> Result<(), Failed> {
    let code = fs::read_to_string(path)?;
    let code = expected_const_regex.replace_all(&code, |caps: &Captures<'_>| {
        let const_name = caps[1].strip_prefix('_').unwrap_or(&caps[1]);
        format!(
            "const {} = {};\npub var {}_const = {}; // expected: {}",
            const_name,
            &caps[2],
            const_name.to_lowercase(),
            const_name,
            &caps[3],
        )
    });
    assert_eq!(
        code.matches(EXPECTED_PATTERN).count(),
        expected_var_regex.captures_iter(&code).count(),
        "some of the expected values are ignored in '{}'",
        path.display()
    );
    let generated_path = env::temp_dir().join(path);
    fs::create_dir_all(files::path_parent(&generated_path))?;
    fs::write(&generated_path, code.as_ref())?;
    Ok(())
}
