# Integration tests guidelines

- Add tests in priority in an existing test folder/file.
- In case no existing test folder/file correspond to the test to add, then create a new test
  folder/file.
- Follow existing conventions (file naming, GPEx function/variable naming, ...).
- When a literal value doesn't matter in a `nok_` test, it is set to zero.
- Example literals within a test file are generally ordered (first expect value `1`, then `2`, ...)
- Function, variable, type, ... names should be as short as possible and aligned across tests (e.g.
  `fn_`, `used`, `called`, ...). Most generally, they should be aligned with other similar tests.
