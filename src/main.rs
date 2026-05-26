//! CLI of `GPEx` language.
#![allow(clippy::print_stdout)] // needed to display messages

// coverage: off (difficult to test)

use clap::Parser;
use gpex::{Log, LogLevel, Program, Runner};
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
enum Args {
    Compile(CompileArgs),
    Run(RunArgs),
}

#[derive(Debug, Parser)]
struct CompileArgs {
    /// Path to the source folder to compile.
    input: PathBuf,
    /// Path to the compiled file.
    output: PathBuf,
    /// Exit with code 1 in case there are warnings.
    #[arg(long, default_value_t = false)]
    is_warning_treated_as_error: bool,
}

#[derive(Debug, Parser)]
struct RunArgs {
    /// Path to either the compiled program or the source folder to run.
    input: PathBuf,
    /// List of variables to display at each step in the terminal, in the format `<module dot path>:<variable name>`.
    #[arg(short='v', long="var", num_args(0..), default_values_t = Vec::<String>::new())]
    pub var_paths: Vec<String>,
}

#[tokio::main]
async fn main() {
    match Args::parse() {
        Args::Compile(args) => compile(&args),
        Args::Run(args) => run(&args).await,
    }
}

fn compile(args: &CompileArgs) {
    let program = compile_dir(&args.input, args.is_warning_treated_as_error);
    if let Err(errors) = gpex::save_compiled(&program, &args.output) {
        display_log(&errors);
        std::process::exit(1);
    } else {
        let log = Log {
            level: LogLevel::Info,
            msg: format!("program saved in \"{}\"", args.output.display()),
            location: None,
            inner: vec![],
        };
        eprint!("{log}");
    }
}

async fn run(args: &RunArgs) {
    if args.input.is_dir() {
        run_program(compile_dir(&args.input, false), args).await;
    } else {
        match gpex::load_compiled(&args.input) {
            Ok(program) => run_program(program, args).await,
            Err(errors) => {
                display_log(&errors);
                std::process::exit(1);
            }
        }
    }
}

fn compile_dir(dir_path: &Path, is_warning_treated_as_error: bool) -> Program {
    match gpex::compile_program(dir_path, is_warning_treated_as_error) {
        Ok((program, logs)) => {
            display_log(&logs);
            program
        }
        Err(errors) => {
            display_log(&errors);
            std::process::exit(1);
        }
    }
}

async fn run_program(program: Program, args: &RunArgs) {
    let mut runner = match Runner::new(program).await {
        Ok(runner) => runner,
        Err(errors) => {
            display_log(&errors);
            std::process::exit(1);
        }
    };
    loop {
        runner.run_step();
        for var_path in &args.var_paths {
            if let Some(value) = runner.read_var(var_path) {
                println!("{var_path} = `{value}`");
            } else {
                let log = Log {
                    level: LogLevel::Warning,
                    msg: format!("`{var_path}` variable not found"),
                    location: None,
                    inner: vec![],
                };
                eprint!("{log}");
            }
        }
    }
}

fn display_log(logs: &[Log]) {
    for log in logs {
        eprint!("{log}");
    }
}
