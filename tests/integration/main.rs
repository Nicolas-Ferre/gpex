//! Integration tests to ensure that the compiler behaves as expected.

use gpex::{Log, Program, Runner};
use itertools::Itertools;
use libtest_mimic::{Arguments, Failed, Trial};
use pretty_assertions::assert_eq;
use regex::{Captures, Regex};
use std::ffi::OsStr;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::{env, fs};
use tokio::runtime::Runtime;
use wgsl_parse::syntax::TranslationUnit;

const EXPECTED_VAR_REGEX: &str = r"var +(\w+) *= *[^;]*; *// expected: *(.+)";
const EXPECTED_CONST_REGEX: &str = r"const +(\w+) *= *([^;]*); *// expected: *(.+)";
const EXPECTED_PATTERN: &str = "// expected";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = Arc::new(Runtime::new()?);
    let args = Arguments::from_args();
    let mut trials = vec![];
    for root_entry in fs::read_dir(Path::new("tests/integration"))? {
        let root_entry = root_entry?;
        let root_path = root_entry.path();
        if !root_path.is_dir() {
            continue;
        }
        for inner_entry in fs::read_dir(root_path)? {
            let inner_entry = inner_entry?;
            let inner_path = inner_entry.path();
            if !inner_path.is_dir() {
                continue;
            }
            let trial_name = inner_path.to_string_lossy().to_string();
            let runtime = runtime.clone();
            trials.push(Trial::test(trial_name, move || {
                run_case(&inner_path, runtime)
            }));
        }
    }
    libtest_mimic::run(&args, trials).exit();
}

fn run_case(path: &Path, runtime: Arc<Runtime>) -> Result<(), Failed> {
    let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
    runtime.block_on(async move {
        if dir_name.starts_with("ok_") {
            run_ok_cases(path, false).await
        } else if dir_name.starts_with("wgsl_") {
            run_ok_cases(path, true).await
        } else if dir_name.starts_with("nok_") {
            run_nok_cases(path)
        } else {
            Err(
                format!("Test directory should have 'ok_', 'wgsl_' or 'nok_' prefix: {dir_name}")
                    .into(),
            )
        }
    })
}

async fn run_ok_cases(path: &Path, is_wgsl_check_enabled: bool) -> Result<(), Failed> {
    let generated_path = generate_case(path)?;
    let (program, _) = convert_gpex_result(gpex::compile_program(&generated_path, true))?;
    let mut runner = convert_gpex_result(Runner::new(program).await)?;
    runner.run_step();
    check_global_vars(&generated_path, &generated_path, &runner)?;
    if is_wgsl_check_enabled {
        check_wgsl_output(path, runner.program())?;
    }
    Ok(())
}

fn run_nok_cases(path: &Path) -> Result<(), Failed> {
    let logs = gpex::compile_program(path, true).err().unwrap_or_default();
    if logs.is_empty() {
        Err("no compiler log returned".into())
    } else {
        let actual = logs
            .iter()
            .map(Log::to_string)
            .join("")
            .replace(&path.display().to_string(), "<root>");
        let expected_path = path.join(".expected");
        if expected_path.exists() {
            let expected = fs::read_to_string(&expected_path)?;
            assert_eq!(actual.trim(), expected.trim());
            Ok(())
        } else {
            fs::write(&expected_path, actual)?;
            Err(format!("expected logs saved on disk in {}", expected_path.display()).into())
        }
    }
}

fn convert_gpex_result<T>(result: Result<T, Vec<Log>>) -> Result<T, Failed> {
    match result {
        Ok(output) => Ok(output),
        Err(logs) => Err((String::from("Compilation failed:\n")
            + &logs.iter().map(ToString::to_string).join(""))
            .into()),
    }
}

fn generate_case(root_path: &Path) -> Result<PathBuf, Failed> {
    let test_dir = env::temp_dir().join(root_path);
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir)?;
    }
    generate_case_dir(root_path)?;
    Ok(test_dir)
}

fn generate_case_dir(dir_path: &Path) -> Result<(), Failed> {
    let expected_const_regex = Regex::new(EXPECTED_CONST_REGEX)?;
    let expected_var_regex = Regex::new(EXPECTED_VAR_REGEX)?;
    for entry in dir_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            generate_case_dir(&path)?;
        } else if path.extension() == Some(OsStr::new("gpex")) {
            let code = fs::read_to_string(&path)?;
            let code = expected_const_regex.replace_all(&code, |caps: &Captures<'_>| {
                let const_name = caps[1].strip_prefix("_").unwrap_or(&caps[1]);
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
                expected_var_regex.captures_iter(&code).count(),
                code.matches(EXPECTED_PATTERN).count(),
                "some of the expected values are ignored in '{}'",
                path.display()
            );
            let generated_path = env::temp_dir().join(path);
            fs::create_dir_all(path_parent(&generated_path))?;
            fs::write(&generated_path, code.as_ref())?;
        }
    }
    Ok(())
}

fn check_global_vars(dir_path: &Path, root_path: &Path, runner: &Runner) -> Result<(), Failed> {
    let mut all_actual = String::new();
    let mut all_expected = String::new();
    let expected_regex = Regex::new(EXPECTED_VAR_REGEX)?;
    for entry in dir_path.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            check_global_vars(&path, root_path, runner)?;
        } else if path.extension() == Some(OsStr::new("gpex")) {
            let code = fs::read_to_string(&path)?;
            let dot_path = to_dot_path(&path, root_path);
            for capture in expected_regex.captures_iter(&code) {
                let var_name = &capture[1];
                let expected_value = capture[2].trim();
                let var_path = format!("{dot_path}:{var_name}");
                let actual_value = runner
                    .read_var(&var_path)
                    .map_or_else(|| "<unknown>".into(), |value| value.to_string());
                writeln!(all_actual, "{var_path}={actual_value}")?;
                writeln!(all_expected, "{var_path}={expected_value}")?;
            }
        }
    }
    assert_eq!(all_actual, all_expected);
    Ok(())
}

fn path_parent(path: &Path) -> &Path {
    path.parent()
        .unwrap_or_else(|| unreachable!("parent should be at least temporary folder"))
}

fn check_wgsl_output(path: &Path, program: &Program) -> Result<(), Failed> {
    let actual_code = format!(
        "// INIT SHADER\n\n{}\n\n// UPDATE SHADER\n\n{}\n",
        format_wgsl(&program.init_shader)?,
        format_wgsl(&program.update_shader)?
    );
    let expected_path = path.join(".expected");
    if expected_path.exists() {
        let expected_code = fs::read_to_string(&expected_path)?;
        assert_eq!(actual_code, expected_code);
        Ok(())
    } else {
        fs::write(&expected_path, actual_code)?;
        Err(format!(
            "expected WGSL code saved on disk in {}",
            expected_path.display()
        )
        .into())
    }
}

fn format_wgsl(code: &str) -> Result<String, Failed> {
    Ok(TranslationUnit::from_str(code)?.to_string())
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
