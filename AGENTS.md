# Repository guidelines

The project contains a compiler and runner CLI for GPEx, a custom programming language that runs
entirely on the GPU side.

The CLI is written in Rust (minimal supported version and edition is in `Cargo.toml`).

## Project structure

See `doc/architecture.md`.

## Test structure

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

- Always keep `README.md`, `AGENTS.md` and `doc/*.md` files up-to-date
- Take into account all `AGENTS.md` files within the path of a modified file. For example, the file
  `./a/b/c.rs` should respect `./AGENTS.md`, `./a/AGENTS.md` and `./a/b/AGENTS.md` if they exist.
- Never use the question tool, always ask them in Markdown:
    - Ask them one-by-one.
    - Display them using the following template:
      ```
      Question: **<the question>**
      1. **<first answer>**
      <brief details and snippets about first answer>
      2. **<second answer>**
      <brief details and snippets about second answer>
      3. **<third answer>**
      <brief details and snippets about third answer>
    
      Please choose option 1, 2, or 3.
      ```
    - Questions must be as straightforward, easy to read and easy to understand as possible.
    - Illustrate each answer choice with a code snippet whenever possible to facilitate comparison
      of answers.

## General coding guidelines

- Follow explicit and implicit conventions already existing in the codebase
- New item names should be aligned with other existing items.
- For boolean variables and functions returning booleans:
    - Name should start with `is_`, `are_`, `has_` or `have_`.
    - Noun is placed before the adjective (e.g. `is_user_connected` instead of
      `is_connected_user`).
    - Avoid including a negation in the name (e.g. avoid `is_not_active`,
      `is_disconnected`, ...).
