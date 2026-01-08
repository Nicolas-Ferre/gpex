# Maintenance

Here are the code conventions applied in this repository:

- The CI workflow enforces the main conventions (e.g., by Clippy)
- Additional rules:
    - `impl Trait for Type` blocks should be placed before `impl Type` block of the same type.
    - Use full names instead of abbreviations (e.g. `VariableDefinition` instead of `VarDef`). Only
      exceptions are abbreviations widely used in Rust programming (e.g. `id` instead of
      `identifier`). `clippy.toml` file contains a list of known abbreviations and full names to
      enforce.
    - Lifetimes names should be explicit (instead of `'a`, `'b`, ...). Singular names are preferred.
    - Regarding boolean variables and functions returning booleans:
        - Name should preferably start with `is` or `has`.
        - Noun is placed before the adjective (e.g. `is_user_connected` instead of
          `is_connected_user`).
        - Avoid including a negation in the name (e.g. avoid `is_not_active`,
          `is_disconnected`, ...).
    - `unreachable` should be preferred to `unwrap()` and `expect()` for errors that should never
      occur in practice. If the error can occur, then a clean error should be returned to the user
      instead of panicking.
    - `use` statements should be placed after `mod` statements.
    - Favor `_ = value;` over `let _ = value;`.
