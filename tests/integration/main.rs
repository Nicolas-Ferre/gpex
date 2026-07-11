//! Integration tests to ensure that the compiler behaves as expected.

use gpex::{Log, Program, Runner};
use itertools::Itertools;
use libtest_mimic::{Arguments, Failed, Trial};
use pretty_assertions::assert_eq;
use regex::{Captures, Regex};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt::Write;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use std::{env, fs};
use tokio::runtime::Runtime;
use wgsl_parse::syntax::TranslationUnit;

const NUMBERED_IDENT_REGEX: &str = r"[v_]+([0-9]+)";
const EXPECTED_VAR_REGEX: &str = r"var +(\w+) *= *[^;]*; *// expected: *(.+)";
const EXPECTED_CONST_REGEX: &str = r"const +(\w+) *= *([^;]*); *// expected: *(.+)";
const EXPECTED_PATTERN: &str = "// expected";
const GPEX_EXT: &str = "gpex";

#[derive(Clone, Copy)]
enum CaseKind {
    Ok,
    Wgsl,
    Nok,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    owo_colors::set_override(false);
    let runtime = Arc::new(Runtime::new()?);
    let args = Arguments::from_args();
    let mut trials = vec![];
    collect_case_dirs(Path::new("tests/integration"), &runtime, &mut trials)?;
    libtest_mimic::run(&args, trials).exit();
}

fn collect_case_dirs(
    path: &Path,
    runtime: &Arc<Runtime>,
    trials: &mut Vec<Trial>,
) -> Result<(), Box<dyn std::error::Error>> {
    for root_entry in fs::read_dir(path)? {
        let root_entry = root_entry?;
        let root_path = root_entry.path();
        if !root_path.is_dir() {
            continue;
        }
        if let Some(case_kind) = case_kind(&root_path) {
            let trial_name = root_path.to_string_lossy().to_string();
            let runtime = runtime.clone();
            trials.push(Trial::test(trial_name, move || {
                run_case(&root_path, case_kind, runtime)
            }));
        } else if has_gpex_file(&root_path)? {
            let dir_name = path_file_name(&root_path);
            return Err(format!(
                "Test directory should have 'ok_', 'wgsl_' or 'nok_' prefix: {dir_name}"
            )
            .into());
        } else {
            collect_case_dirs(&root_path, runtime, trials)?;
        }
    }
    Ok(())
}

fn case_kind(path: &Path) -> Option<CaseKind> {
    let dir_name = path_file_name(path);
    if dir_name.starts_with("ok_") {
        Some(CaseKind::Ok)
    } else if dir_name.starts_with("wgsl_") {
        Some(CaseKind::Wgsl)
    } else if dir_name.starts_with("nok_") {
        Some(CaseKind::Nok)
    } else {
        None
    }
}

fn has_gpex_file(path: &Path) -> Result<bool, IoError> {
    for entry in fs::read_dir(path)? {
        if entry?.path().extension() == Some(OsStr::new(GPEX_EXT)) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn run_case(path: &Path, case_kind: CaseKind, runtime: Arc<Runtime>) -> Result<(), Failed> {
    runtime.block_on(async move {
        match case_kind {
            CaseKind::Ok => run_ok_cases(path, false).await,
            CaseKind::Wgsl => run_ok_cases(path, true).await,
            CaseKind::Nok => run_nok_cases(path),
        }
    })
}

async fn run_ok_cases(path: &Path, is_wgsl_check_enabled: bool) -> Result<(), Failed> {
    let is_warning_treated_as_error = !path.join(".allow_warnings").exists();
    let generated_path = generate_case(path)?;
    let (program, _) = convert_gpex_result(gpex::compile_program(
        &generated_path,
        is_warning_treated_as_error,
    ))?;
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
        let expected_path = path.join(".expected.stderr");
        if expected_path.exists() {
            let expected = fs::read_to_string(&expected_path)?;
            assert_eq!(expected.trim(), actual.trim());
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
    fs::create_dir_all(&test_dir)?;
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
        } else if path.extension() == Some(OsStr::new(GPEX_EXT)) {
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
                code.matches(EXPECTED_PATTERN).count(),
                expected_var_regex.captures_iter(&code).count(),
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
        } else if path.extension() == Some(OsStr::new(GPEX_EXT)) {
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
    assert_eq!(all_expected, all_actual);
    Ok(())
}

fn path_parent(path: &Path) -> &Path {
    path.parent()
        .unwrap_or_else(|| unreachable!("parent should be at least temporary folder"))
}

fn path_file_name(path: &Path) -> Cow<'_, str> {
    path.file_name().unwrap_or_default().to_string_lossy()
}

fn check_wgsl_output(path: &Path, program: &Program) -> Result<(), Failed> {
    let actual_code = format!(
        "// INIT SHADER\n\n{}\n\n// UPDATE SHADER\n\n{}",
        format_wgsl(&program.init_shader)?,
        format_wgsl(&program.update_shader)?
    );
    let expected_path = path.join(".expected.wgsl");
    if expected_path.exists() {
        let expected_code = fs::read_to_string(&expected_path)?;
        assert_eq!(expected_code, actual_code);
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
    let mut formatted_wgsl = TranslationUnit::from_str(code)?.to_string();
    let numbered_ident_regex = Regex::new(NUMBERED_IDENT_REGEX)?;
    let replaced_idents = numbered_ident_regex
        .captures_iter(&formatted_wgsl)
        .filter_map(|captures| {
            let ident = captures.get(0)?.as_str();
            let number = captures.get(1)?.as_str().parse::<u64>().ok()?;
            Some((ident, number))
        })
        .unique_by(|(ident, _)| *ident)
        .sorted_unstable_by_key(|(_, number)| *number)
        .enumerate()
        .map(|(index, (ident, _))| (ident.to_string(), format!("ident{index}")))
        .sorted_unstable_by_key(|(old_name, _)| usize::MAX - old_name.len()) // to avoid replacing variable prefixes
        .collect::<Vec<_>>();
    for (old_name, new_name) in &replaced_idents {
        formatted_wgsl = formatted_wgsl.replace(old_name, new_name);
    }
    Ok(formatted_wgsl)
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
