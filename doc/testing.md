# Testing

Two test suites are defined, each with its own `main.rs` harness.

## Library tests (`tests/lib/`)

Tests items exported by the `lib` crate, so they are normally rarely updated.

Some tests rely on `.gpex` examples:

- `valid/`: program that compiles successfully.
- `error/`: program that produces errors.
- `warning/`: program that produces warnings.

## Integration tests (`tests/integration/`)

Tests examples of GPEx programs located in `tests/integration/*/`. The testing behavior depends on
the name prefix of the inner directory.

### `tests/integration/*/ok_*` test directories

These directories are tested the following way:

- Compile the directory.
- Verify the compilation succeeded.
- Verify no compiler warning was produced, unless a `.allow_warnings` file exists in the test
  directory.
- Run the program.
- Verify that expected values of variables match actual values stored on GPU.
  An expected value can be indicated in a `.gpex` file using the following type of comment:
  ```gpex
  var _result = 2_147_483_647; // expected: 2147483647
  const _RESULT = 2_147_483_647; // expected: 2147483647
  ```

### `tests/integration/*/wgsl_*` test directories

These directories are tested the following way:

- Same as `ok_*` tests.
- Verify that the generated WGSL code matches the expected WGSL code in `.expected.wgsl`.
  If this file doesn't exist, it is auto-generated during the first test run.

### `tests/integration/*/nok_*` test directories

These directories are tested the following way:

- Compile the directory.
- Verify the compilation returned errors or warnings.
- Verify that the compilation error messages match the expected messages in `.expected.stderr`.
  If this file doesn't exist, it is auto-generated during the first test run.

### Test dimensions

Some of the test folders inside `tests/integration/*/` can have a suffix that indicates the
dimension of the test. Here are the common ones:

- `*_exprs`: test all forms of expressions in a particular context.
- `*_locations`: test all expression locations in a particular context.
- `*_scopes`: test main semantic scopes in a particular context. For these tests, it is not needed
  to test all locations, only the main group of locations are enough.
- `*_forms`: test all forms of a concept or an item.

It is possible to use other dimensions when it makes sense, but they should be clearly extensible.
