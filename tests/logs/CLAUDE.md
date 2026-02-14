# Diagnostic snapshot tests

Compiles `.gpex` fixtures that trigger errors/warnings and compares the full log output against
`.expected` snapshot files. If no `.expected` file exists, it is auto-generated on first run.

Error categories: `error_syntax`, `error_circular_items`, `error_disallowed_items`,
`error_not_found_items`, `error_out_of_bounds`, `error_multiple_definitions`,
`error_non_constant_expressions`, `error_type_comparison`.
Warning categories: `warning_unused`, `warning_naming`.
