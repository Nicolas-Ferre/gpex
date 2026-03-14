# Coding conventions

- The CI workflow enforces the main conventions (e.g., by Clippy and rustfmt). See `clippy.toml`
  and `rustfmt.toml` for details.
- Additional rules:
    - Use full names instead of abbreviations (e.g. `VariableDefinition` instead of `VarDef`). Only
      exceptions are abbreviations widely used in Rust programming (e.g. `id` instead of
      `identifier`). `clippy.toml` maintains the lists of known abbreviations, full names and
      allowed short identifiers to enforce.
    - Lifetimes names should be explicit (instead of `'a`, `'b`, ...). Singular names are preferred.
    - Regarding boolean variables and functions returning booleans:
        - Name should preferably start with `is` or `has`.
        - Noun is placed before the adjective (e.g. `is_user_connected` instead of
          `is_connected_user`).
        - Avoid including a negation in the name (e.g. avoid `is_not_active`,
          `is_disconnected`, ...).
    - `unreachable!()` should be preferred to `unwrap()` and `expect()` for conditions that should
      never occur in practice. If an error can occur, then a clean error should be returned to the
      user instead of panicking.
    - `use` statements should be placed after `mod` statements.
    - Favor `_ = value;` over `let _ = value;`.
    - Within a same file, functions should be defined after their call sites, so that the file goes
      from high-level functions to lower-level functions.
