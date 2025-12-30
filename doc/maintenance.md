# Maintenance

Here are the code conventions applied in this repository:

- Main conventions are enforced by the CI workflow (e.g. by Clippy)
- Additional rules:
    - `impl Trait for Type` blocks should be placed before `impl Type` block of the same type.
    - Use full names instead of abbreviations (e.g. `VariableDefinition` instead of `VarDef`). Only
      exceptions are abbreviations widely used in Rust programming (e.g. `id` instead of
      `identifier`). `clippy.toml` file contains a list of known abbreviations and full names to
      enforce.
    - Lifetimes names should be explicit (instead of `'a`, `'b`, ...).
    - Regarding boolean variables and functions returning booleans:
        - Name should preferably start with `is` or `has`.
        - Noun is placed before the adjective (e.g. `is_user_connected` instead of
          `is_connected_user`).
        - Avoid including a negation in the name (e.g. avoid `is_not_active`,
          `is_disconnected`, ...).
    - `unwrap()` and `expect()` should be avoided as much as possible. In case they cannot be
      avoided:
        - Use `unwrap()` in one of these cases:
            - There is a high confidence the code will never panic due to local logic.
            - A `debug_assert` guard is defined just before.
        - Use `expect()` in one of these cases:
            - It is not sure the code will never panic (e.g. because third party function is
              called).
            - The code should never panic due to another piece of code that is not local.
    - `use` statements should be placed after `mod` statements.
    - Favor `_ = value;` over `let _ = value;`.
