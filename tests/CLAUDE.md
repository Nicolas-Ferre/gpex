# Testing

Three integration test suites, each with its own `main.rs` harness.

## Test organization: forms × locations

Both `runner/` and `logs/` use a two-axis convention for combinatorial coverage:

- **`locations/`** — Tests the same construct in every valid syntactic position. Each file is named
  after the position: `variable_global.gpex`, `constant_global.gpex`, `statement_return.gpex`,
  `statement_assignment_right.gpex`, `function_return_type.gpex`, etc.

- **`forms/`** — Tests different variants or shapes of the same construct. For literals this means
  different values (`zero.gpex`, `max.gpex`, `min.gpex`). For items that can be imported, this
  includes multi-file scenarios (`imported_direct/`, `imported_indirect/` with their own
  `main.gpex` + `imported.gpex`).

A typical test directory looks like (but avoid unique nested folder when unnecessary):

```
expressions/literals_i32/
├── forms/          # zero.gpex, max.gpex, min.gpex
└── locations/      # variable_global.gpex, constant_global.gpex, statement_return.gpex, ...
```

When adding a new language feature, follow this pattern: create a `locations/` subdirectory covering
every syntactic position the feature can appear in, and a `forms/` subdirectory covering its value
variants and import scenarios.
