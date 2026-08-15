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

Directories group tests by domain, feature, and optionally more specific subfeatures. A test
directory is any directory whose name starts with `ok_`, `wgsl_` or `nok_`, and this prefix defines
the testing behavior.

### `ok_*` test directories

These directories are tested the following way:

- Compile the directory.
- Verify the compilation succeeded.
- Verify no compiler warning was produced, unless a `.allow_warnings` file exists in the test
  directory.
- Run the program.
- Verify that expected values of variables match actual values stored on GPU.
  Expected values can be indicated for variables and constants in `.gpex` files using the following
  type of comment:
  ```gpex
  var value = 0; // expected: 1, 2, 3
  const _RESULT = 2_147_483_647; // expected: 2147483647, 2147483647
  ```
  Comma-separated values are checked after successive frames. A variable is no longer checked after
  its final expected value. Each test directory runs as many frames as its longest expected-value
  list across all `.gpex` files, or one frame if it contains no expected values.

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
- Verify that the compiler messages match the expected messages in `.expected.stderr`.
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
- `tests/integration/prelude/`: tests are organized per built-in prelude item kind.
- `tests/integration/type_narrowing/`: tests are organized per narrowing form and concept, using the
  naming convention `{operator}_{location}{optional sub-category}`:
    - operator: type narrowing condition operator, e.g. `eq` (equal) and `ne` (not equal)
    - location: tested location in the type narrowing expression (e.g. `fact` for the type fact
      condition, and `usage` for the expression after the type fact)
    - optional sub-category:
        - `*_form_{left_operand}_{right_operand}` for tests related to a form of type fact
        - `*_operand_{type}` for tests related to a specific type of operand in a type fact

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

Each test case is defined by an entrypoint file named `test_*.gpex` inside a test directory. A test
directory may also contain supporting `.gpex` files, such as imported modules.

After the `test_` prefix, the remainder of the filename identifies the exact tested scenario using
hierarchical naming.
