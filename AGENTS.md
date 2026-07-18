# Repository guidelines

The project contains a compiler and runner CLI for GPEx, a custom programming language that runs
entirely on GPU side.

CLI is written in Rust (minimal supported version and edition is in `Cargo.toml`).

## Project structure

See `doc/architecture.md`.

## Rust coding conventions to respect

See `doc/coding_conventions.md`.

## Test structure and conventions

See `doc/testing.md`.

## Commands

- To run to validate implementations:
    - `cargo test --no-fail-fast`: run tests
    - `bash .github/scripts/run_all_lints.sh --fast`: run fast linters
- To run at the end of a full task only:
    - `bash .github/scripts/run_all_lints.sh`: run all linters, including slower ones
- To run when checking a specific GPEx program:
    - `cargo run -- compile <source-dir> <output-file>`: compile a GPEx program.
    - `cargo run -- run <source-dir>`: compile and continuously execute a program on the GPU.
    - `cargo run -- run <compiled-file>`: continuously execute a program on the GPU.

## Branching conventions

- Number prefix at the start of a branch corresponds to the GitHub issue ID
- Branch can either correspond to the whole issue or to a sub-part of the issue

## Rules to absolutely respect

- Don't use any skill automatically, except if explicitly stated
- Always keep `README.md`, `AGENTS.md` and `doc/*.md` files up-to-date
- Take into account all `AGENTS.md` files within the path of a modified file. For example, the file
  `./a/b/c.rs` should respect `./AGENTS.md`, `./a/AGENTS.md` and , `./a/b/AGENTS.md` if they exist.
- Plans must follow `.agents/PLANS.md`
- Follow explicit and implicit conventions already existing in the codebase
- New item names should be aligned with other existing items.
- Always test implementations in `tests/integration/`
