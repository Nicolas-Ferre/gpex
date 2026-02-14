# CLAUDE.md

GPEx is a GPU programming language written in Rust that compiles to WGSL (WebGPU Shading Language)
and runs on the GPU via wgpu. The CLI provides `compile` and `run` subcommands.

## Build & Development Commands

```bash
cargo build                                    # Build
cargo test --no-fail-fast                      # Run all tests
cargo test <test_name>                         # Run a single test (e.g. cargo test compile_valid_project)
cargo fmt                                      # Format
cargo clippy --all-targets --no-deps -- -D warnings  # Lint (strict: all+pedantic+nursery+cargo)
```

Minimum Rust version: 1.92.0 (edition 2024).

## Code Style & Lint Rules

Clippy is configured very strictly (all+pedantic+nursery+cargo at warn, `-D warnings` in CI).
Notable rules from `clippy.toml`:

- **Naming**: No abbreviations like `err`, `ctx`, `var`, `semi`, `loc`. No full names like `ident`,
  `reference`, `argument` when abbreviation is preferred. Min identifier length is 2 chars.
- **Allowed short identifiers**: `'_`, `_`, `io`, `Io`, `fs`, `id`
- **Forbidden patterns**: `unwrap_used`, `expect_used`, `todo`, `unimplemented`, `dbg_macro`,
  `print_stdout`
- **Formatting**: Unix line endings, field init shorthand (`rustfmt.toml`)

GPEx language conventions: `snake_case` for functions (except those returning `typeref` which may be
`PascalCase`), `PascalCase` for structs.
