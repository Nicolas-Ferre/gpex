# Testing

Two test suites are defined, each with its own `main.rs` harness.

## Library tests (`tests/lib/`)

Tests items exported by the `lib` crate, so they are normally rarely updated.

Some tests rely on `.gpex` examples:

- `valid/`: program that compiles successfully.
- `error/`: program that produces errors.
- `warning/`: program that produces warnings.

## Integration tests (`tests/integration/`)

Tests examples of GPEx programs located in `tests/integration/*/*/*/`.

The first two directory levels group tests by domain and feature. The third directory level is the
test case itself and its name prefix defines the testing behavior.

### `tests/integration/*/*/*/ok_*` test directories

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

### `tests/integration/*/*/*/wgsl_*` test directories

These directories are tested the following way:

- Same as `ok_*` tests.
- Verify that the formatted generated WGSL code matches the expected WGSL code in
  `.expected.wgsl`.
  If this file doesn't exist, it is auto-generated during the first test run.

### `tests/integration/*/*/*/nok_*` test directories

These directories are tested the following way:

- Compile the directory.
- Verify the compilation returned errors or warnings.
- Verify that the compilation error messages match the expected messages in `.expected.stderr`.
  If this file doesn't exist, it is auto-generated during the first test run.

### Test domains

The following test domains are defined:

- `tests/integration/expr_locations/`: tests are organized per syntactical expression location (
  returned value, variable default value, binary left operand, ...).
- `tests/integration/expr_types/`: tests are organized per expression kind (literal, variable
  reference, function call, ...)
- `tests/integration/items/`: tests are organized per item (variable, constant, function, ...)

### Test cases

The following test cases at `tests/integration/*/*/*/` are commonly defined:

- `ok_syntax`: test valid syntax forms
- `ok_semantic`: test invalid feature semantic
- `nok_semantic`: test valid feature semantic
- `ok_search`: test items that are accessible from a given location
- `nok_search`: test items that are not accessible from a given location
- `ok_naming`: test valid item names
- `nok_naming`: test invalid or not recommended item names
- `wgsl_inlining`: test that items are inlined during transpilation
- `wgsl_no_inlining`: test that items are not inlined during transpilation
- `wgsl_transpiling`: test that items are correctly transpiled

### Test files

Each test case is defined in a file `tests/integration/*/*/*/test_*.gpex`.

The filename starts with `test_`, and the rest of the filename defined the exact tested case using a
hierarchical naming.

### GPEx conventions

- When a literal value doesn't matter in a `nok_` test, it is set to zero.
- Example literals within a test file are generally ordered (first expect value `1`, then `2`, ...)
- Generic functions primarily tested by the file are named `fn_`.
- Secondary helper functions called by the primary function are named `called`.
- Functions named `used` are reserved for tests where item usage or search visibility is part of the
  tested behavior.
