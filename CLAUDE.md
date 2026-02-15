# CLAUDE.md

GPEx is a GPU programming language written in Rust that compiles to WGSL (WebGPU Shading Language)
and runs on the GPU via wgpu. The CLI provides `compile` and `run` subcommands.

## Build & development commands

```bash
cargo build                                    # Build
cargo test --no-fail-fast                      # Run all tests
cargo test <test_name>                         # Run a single test (e.g. cargo test compile_valid_project)
cargo fmt                                      # Format
cargo clippy --all-targets --no-deps -- -D warnings  # Lint (strict: all+pedantic+nursery+cargo)
```

Minimum Rust version: 1.92.0 (edition 2024).

## Documentation

Detailed documentation is available in `doc/`. Read the relevant file when working on the
corresponding area:

- @doc/architecture.md — Compiler pipeline and project structure. Read when modifying `src/` or
  understanding the compilation process.
- @doc/testing.md — Test organization, conventions and harnesses. Read when adding or modifying
  tests.
- @doc/maintenance.md — Code conventions and style rules. Read when writing or reviewing code.
- @clippy.toml — Naming rules (abbreviations, full names, allowed short identifiers). Read when
  naming variables, functions or types. Can be updated if needed.
