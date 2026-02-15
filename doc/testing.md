# Testing

Three integration test suites, each with its own `main.rs` harness.

## Test organization

Tests for `runner/` and `logs/` can be organized along multiple dimensions. Two common ones are:

- **`locations/`** — Tests the same construct in every valid syntactic position. Each file is named
  after the position (e.g. `variable_global.gpex`, `statement_return.gpex`, ...).

- **`forms/`** — Tests different variants or shapes of the same construct. For literals this means
  different values (e.g. `zero.gpex`, `max.gpex`, ...). For items that can be imported, this
  includes multi-file scenarios (e.g. subdirectories with their own `main.gpex` + `imported.gpex`,
  in these scenarios, `main.gpex` imports `imported.gpex` — not the other way around).

Only create a dimension subdirectory when it contains multiple entries. When a feature has only one
dimension, place files directly under the feature directory. Each test file should cover a distinct
behavior — avoid redundant tests that overlap in what they verify.

It is important to make sure tests remain as exhaustive as possible (i.e. cover all kinds of cases
in each dimension).

## Compiler integration tests (`tests/lib/`)

Tests items exported by the `lib` crate.

Some tests rely on `.gpex` examples:

- `valid/` — Projects that compile successfully (asserts program structure and buffer fields)
- `error/` — Projects that produce errors
- `warning/` — Projects that produce warnings

## GPU execution tests (`tests/runner/`)

Compiles `.gpex` fixtures, runs them on the GPU, then verifies variable values. Test expectations
are embedded in the source files as comments:

```gpex
var _result = 2_147_483_647; // expected: 2147483647
```

The harness scans for `// expected: <value>` and asserts `runner.read_variable()` matches.

## Diagnostic snapshot tests (`tests/logs/`)

Compiles `.gpex` fixtures that trigger errors/warnings and compares the full log output against
`.expected` snapshot files. If no `.expected` file exists, it is auto-generated on first run.

Subdirectories are prefixed by category (e.g. `error_syntax/`, `warning_unused/`, ...).
