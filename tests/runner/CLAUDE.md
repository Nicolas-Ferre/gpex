# GPU execution tests

Compiles `.gpex` fixtures, runs them on the GPU, then verifies variable values. Test expectations
are embedded in the source files as comments:

```
var _result = 2_147_483_647; // expected: 2147483647
```

The harness scans for `// expected: <value>` and asserts `runner.read_variable()` matches.

Other subdirectories: `items/imports/` (import resolution), `syntax/` (comments), `prelude/`
(built-in types), `empty/` (empty program).
