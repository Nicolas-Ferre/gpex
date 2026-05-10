# Testing

Three integration test suites, each with its own `main.rs` harness.

## Compiler integration tests (`tests/lib/`)

Tests items exported by the `lib` crate, so they are normally rarely updated.

Some tests rely on `.gpex` examples:

- `valid/`: Projects that compile successfully (asserts program structure and buffer fields)
- `error/`: Projects that produce errors
- `warning/`: Projects that produce warnings

## GPU execution tests (`tests/runner/`)

Compiles `.gpex` fixtures, runs them on the GPU, then verifies variable values. Test expectations
are embedded in the source files as comments:

```gpex
var _result = 2_147_483_647; // expected: 2147483647
```

The harness scans for `// expected: <value>` and asserts that the value stored on GPU side matches.

## Diagnostic snapshot tests (`tests/logs/`)

Compiles `.gpex` fixtures that trigger errors/warnings and compares the full log output against
`.expected` snapshot files. If no `.expected` file exists, it is auto-generated on first run.

To be noted that when using parametrized tests, it is possible to specify a `.expected__$$` file
instead of `.expected`, which contains the template of the expected logs for each case. The file
supports the same placeholders as `.gpex` files.

Subdirectories follow the naming convention `<log level>_<subcategory>` (e.g. `error_syntax/`,
`warning_unused/`, ...).

## Parametrized tests

Tests in `tests/lib/` and `tests/runner/` are organized in folders, each containing either:

- Simple tests: a static test folder with `.gpex` files (e.g., `syntax/`).
- Parametric tests: a `cases.yaml` file defining test dimensions, plus `.gpex` template files.

The parametric tests support the following placeholders:

- `$$` in filenames and `.gpex`file content: replaced by the unique name of the generated test.
- `{{dimension.key}}` in `.gpex` file content: replaced by a property of the generated case defined
  in `cases.yaml`.

The test harness generates concrete test cases by taking the Cartesian product of all dimension
cases and substituting placeholders.

`cases.yaml` schema is the following:

```yaml
dimensions: # dimensions are applied in order for the replacement of placeholders 
  - id: <dimension name>
    cases:
      <case name>:
        <key1>: <value1> # {{<dimension name>.<key1>}} will be replaced by <value1> in .gpex files
        ...
  - ...
```

It is possible to disable a test by inserting `<EXCLUDE>` in a generated `.gpex` file.
To optimize tests, it is recommended to exclude all generated files of a given test case, not only
the main file.
