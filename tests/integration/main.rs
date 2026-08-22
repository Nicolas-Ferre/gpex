//! Integration tests to ensure that the compiler behaves as expected.

mod case_generation;
mod case_kind;
mod expected_values;
mod files;
mod warnings_ignore;
mod wgsl;

use case_kind::CaseKind;
use gpex::{Log, Program, Runner};
use itertools::Itertools;
use libtest_mimic::{Arguments, Failed, Trial};
use pretty_assertions::assert_eq;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{env, fs};
use tokio::runtime::Runtime;

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
        if let Some(case_kind) = case_kind::case_kind(&root_path) {
            let trial_name = root_path.to_string_lossy().to_string();
            let runtime = runtime.clone();
            trials.push(Trial::test(trial_name, move || {
                run_case(&root_path, case_kind, runtime)
            }));
        } else if files::is_dir_containing_gpex_file(&root_path)? {
            let dir_name = files::path_file_name(&root_path);
            return Err(format!(
                "test directory should have 'ok_', 'wgsl_' or 'nok_' prefix: {dir_name}"
            )
            .into());
        } else {
            collect_case_dirs(&root_path, runtime, trials)?;
        }
    }
    Ok(())
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
    let warning_ignore_paths = warnings_ignore::warning_ignore_paths(path)?;
    let generated_path = case_generation::generate(path)?;
    let program = compile_ok_case(&generated_path, &warning_ignore_paths)?;
    let expected_values = expected_values::collect(&generated_path, &generated_path)?;
    let frame_count = expected_values
        .iter()
        .map(|expected_var| expected_var.values.len())
        .max()
        .unwrap_or(1);
    let mut runner = convert_gpex_result(Runner::new(program).await)?;
    for frame_index in 0..frame_count {
        runner.run_step();
        expected_values::assert(&expected_values, frame_index, &runner)?;
    }
    if is_wgsl_check_enabled {
        check_wgsl_output(path, runner.program())?;
    }
    Ok(())
}

fn compile_ok_case(path: &Path, warning_ignored_paths: &[PathBuf]) -> Result<Program, Failed> {
    let (program, logs) = convert_gpex_result(gpex::compile_program(path, false))?;
    let merged_logs = logs
        .into_iter()
        .filter(|log| !warnings_ignore::is_ignored_warning(log, path, warning_ignored_paths))
        .map(|log| log.to_string())
        .join("");
    if merged_logs.is_empty() {
        Ok(program)
    } else {
        let cleaned_logs = replace_paths_in_logs(&merged_logs, path);
        Err((String::from("unexpected compiler logs:\n") + &cleaned_logs).into())
    }
}

fn run_nok_cases(path: &Path) -> Result<(), Failed> {
    let logs = gpex::compile_program(path, true).err().unwrap_or_default();
    if logs.is_empty() {
        Err("no compiler log returned".into())
    } else {
        let merged_logs = logs.iter().map(Log::to_string).join("");
        let actual = replace_paths_in_logs(&merged_logs, path);
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

fn replace_paths_in_logs(logs: &str, root_path: &Path) -> String {
    logs.replace(&root_path.display().to_string(), "<root>")
        .replace(env!("CARGO_MANIFEST_DIR"), "<project>")
}

fn convert_gpex_result<T>(result: Result<T, Vec<Log>>) -> Result<T, Failed> {
    match result {
        Ok(output) => Ok(output),
        Err(logs) => Err((String::from("compilation failed:\n")
            + &logs.iter().map(ToString::to_string).join(""))
            .into()),
    }
}

fn check_wgsl_output(path: &Path, program: &Program) -> Result<(), Failed> {
    let actual_code = format!(
        "// INIT SHADER\n\n{}\n\n// UPDATE SHADER\n\n{}",
        wgsl::format(&program.init_shader)?,
        wgsl::format(&program.update_shader)?
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
