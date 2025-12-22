//! CLI of `GPEx` language.
#![allow(clippy::print_stdout)] // needed to display messages

// coverage: off (difficult to test)

use clap::Parser;
use gpex::{Log, Program, Runner};
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
    fail_on_warning: bool,
}

#[derive(Debug, Parser)]
struct RunArgs {
    /// Path to either the compiled program or the source folder to run.
    input: PathBuf,
    /// List of variables to display at each step.
    #[arg(short, long, num_args(0..), default_values_t = Vec::<String>::new())]
    pub var: Vec<String>,
}

#[tokio::main]
async fn main() {
    match Args::parse() {
        Args::Compile(args) => compile(&args),
        Args::Run(args) => run(&args).await,
    }
}

fn compile(args: &CompileArgs) {
    let program = compile_folder(&args.input, args.fail_on_warning);
    if let Err(errors) = gpex::save_compiled(&program, &args.output) {
        display_log(&errors);
        std::process::exit(1);
    } else {
        println!("info: program saved in \"{}\"", args.output.display());
    }
}

async fn run(args: &RunArgs) {
    if args.input.is_dir() {
        run_program(compile_folder(&args.input, false), args).await;
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

fn compile_folder(folder_path: &Path, fail_on_warning: bool) -> Program {
    match gpex::compile(folder_path, fail_on_warning) {
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
    runner.run_step();
    for var_path in &args.var {
        if let Some(value) = runner.read_var(var_path) {
            println!("info: {var_path} = `{value}`");
        } else {
            println!("warning: `{var_path}` variable not found");
        }
    }
}

fn display_log(logs: &[Log]) {
    for log in logs {
        print!("{log}");
    }
}
