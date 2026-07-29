# Integration tests guidelines

- Prioritize adding tests to an existing test folder/file.
- If no existing test folder or file corresponds to the test being added, create one.
- Follow existing conventions (file naming, GPEx function/variable naming, ...).
- When a literal value doesn't matter in a `nok_` test, it is set to zero.
- Example literals within a test file are generally ordered (first expected value `1`, then
  `2`, ...).
- Function, variable, type, ... names should be as short as possible and aligned across tests (e.g.
  `fn_`, `used`, `called`, ...). Most generally, they should be aligned with other similar tests.
- It is preferred to have multiple simple `test_*.gpex` files instead of one big file.
