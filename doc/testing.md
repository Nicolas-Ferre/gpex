# Testing

Two test suites are defined, each with its own `main.rs` harness.

## Library tests (`tests/lib/`)

Tests items exported by the `lib` crate, so they are normally rarely updated.

Some tests rely on `.gpex` examples:

- `valid/`: Projects that compile successfully (asserts program structure and buffer fields)
- `error/`: Projects that produce errors
- `warning/`: Projects that produce warnings

## Integration tests (`tests/integration/`)

Tests examples of GPEx programs located in `tests/integration/*/`. The testing behavior depends on
the name prefix of the inner directory.

### `tests/integration/ok_*` test directories

These directories are tested the following way:

- Compile the directory.
- Verify the compilation succeeded.
- Run the program.
- Verify that expected values of variables match actual values stored on GPU.
  An expected value can be indicated in a `.gpex` file using the following type of comment:
  ```gpex
  var _result = 2_147_483_647; // expected: 2147483647
  ```

### `tests/integration/wgsl_*` test directories

These directories are tested the following way:

- Same as `ok_*` tests.
- Verify that the generated WGSL code matches the expected WGSL code in `.expected`. If this file
  doesn't exist, it is auto-generated during the first test run.

### `tests/integration/nok_*` test directories

These directories are tested the following way:

- Compile the directory.
- Verify the compilation returned errors or warnings.
- Verify that the compilation error messages match the expected messages in `.expected`. If this
  file doesn't exist, it is auto-generated during the first test run.
