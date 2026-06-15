# Coding conventions

Here are the conventions that are not enforced by CI workflow.

## General

- For boolean variables and functions returning booleans:
    - Name should start with `is_`, `are_`, `has_` or `have_`.
    - Noun is placed before the adjective (e.g. `is_user_connected` instead of
      `is_connected_user`).
    - Avoid including a negation in the name (e.g. avoid `is_not_active`,
      `is_disconnected`, ...).

## Tests

- When a literal value doesn't matter in a `nok_` test, it is set to zero.
