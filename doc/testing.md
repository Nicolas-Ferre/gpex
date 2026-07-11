# Testing

Two test suites are defined, each with its own `main.rs` harness.

## Library tests (`tests/lib/`)

Tests items exported by the `lib` crate, so they are normally rarely updated.

Some tests rely on `.gpex` examples:

- `valid/`: program that compiles successfully.
- `error/`: program that produces errors.
- `warning/`: program that produces warnings.

## Integration tests (`tests/integration/`)

Tests examples of GPEx programs located in `tests/integration/`.

Directories group tests by domain, feature, and optionally more specific subfeatures. A test case is
any directory whose name starts with `ok_`, `wgsl_` or `nok_`, and this prefix defines the testing
behavior.

### `ok_*` test directories

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

### `wgsl_*` test directories

These directories are tested the following way:

- Same as `ok_*` tests.
- Verify that the formatted generated WGSL code matches the expected WGSL code in
  `.expected.wgsl`.
  If this file doesn't exist, it is auto-generated during the first test run.

### `nok_*` test directories

These directories are tested the following way:

- Compile the directory.
- Verify the compilation returned errors or warnings.
- Verify that the compilation error messages match the expected messages in `.expected.stderr`.
  If this file doesn't exist, it is auto-generated during the first test run.

### Test domains

The following test domains are defined:

- `tests/integration/expr_locations/`: tests are organized per syntactical expression location (
  returned value, variable default value, binary left operand, ...).
- `tests/integration/expr_location_scopes/`: tests are organized per scope containing an expression
  (global scope, function body, ...).
- `tests/integration/expr_forms/`: tests are organized per expression form (literal, variable
  reference, function call, ...)
- `tests/integration/items/`: tests are organized per item (variable, constant, function, ...)
- `tests/integration/logs/`: tests are organized per compiler log rendering behavior.
- `tests/integration/prelude/`: tests are organized per built-in prelude item.

### Test cases

The following test cases are commonly defined:

- `ok_syntax`: test valid syntax forms
- `ok_semantic`: test valid feature semantic
- `nok_semantic`: test invalid feature semantic
- `ok_search`: test items that are accessible from a given location
- `nok_search`: test items that are not accessible from a given location
- `ok_naming`: test valid item names
- `nok_naming`: test invalid or not recommended item names
- `wgsl_inlining`: test that items are inlined during transpilation
- `wgsl_no_inlining`: test that items are not inlined during transpilation
- `wgsl_transpilation`: test that items are correctly transpiled

### Test files

All test cases are defined in `test_*.gpex` files inside test case directories.

The filename starts with `test_`, and the rest of the filename defined the exact tested case using a
hierarchical naming.

### GPEx conventions

- When a literal value doesn't matter in a `nok_` test, it is set to zero.
- Example literals within a test file are generally ordered (first expect value `1`, then `2`, ...)
- Function, variable, type, ... names should be as short as possible and aligned across tests (e.g.
  `fn_`, `used`, `called`, ...). Most generally, they should be aligned with other similar tests.
