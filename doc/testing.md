# Testing

Three integration test suites, each with its own `main.rs` harness.

## Test organization: multiple dimensions

Some tests are defined in multiple dimensions. Here is an example.

Both `runner/` and `logs/` use a two-axis convention for combinatorial coverage:

- **`locations/`** — Tests the same construct in every valid syntactic position. Each file is named
  after the position (e.g. `variable_global.gpex`, `statement_return.gpex`, ...).

- **`forms/`** — Tests different variants or shapes of the same construct. For literals this means
  different values (e.g. `zero.gpex`, `max.gpex`, ...). For items that can be imported, this
  includes
  multi-file scenarios (subdirectories with their own `main.gpex` + `imported.gpex`).

When adding a new language feature, follow this pattern: create a `locations/` subdirectory covering
every syntactic position the feature can appear in, and a `forms/` subdirectory covering its value
variants and import scenarios.

Depending on the case (e.g. when there is one item only for a dimension), it might not
be needed to create subdirectories.

## Compiler integration tests (`tests/lib/`)

Tests items exported by the `lib` crate.

Some tests rely on `.gpex` examples:

- `valid/` — Projects that compile successfully (asserts program structure and buffer fields)
- `error/` — Projects that produce errors
- `warning/` — Projects that produce warnings

## GPU execution tests (`tests/runner/`)

Compiles `.gpex` fixtures, runs them on the GPU, then verifies variable values. Test expectations
are embedded in the source files as comments:

```
var _result = 2_147_483_647; // expected: 2147483647
```

The harness scans for `// expected: <value>` and asserts `runner.read_variable()` matches.

## Diagnostic snapshot tests (`tests/logs/`)

Compiles `.gpex` fixtures that trigger errors/warnings and compares the full log output against
`.expected` snapshot files. If no `.expected` file exists, it is auto-generated on first run.

Subdirectories are prefixed by category (e.g. `error_syntax/`, `warning_unused/`, ...).
