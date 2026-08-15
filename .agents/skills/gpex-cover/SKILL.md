---
name: gpex-cover
description: Increase test coverage for a specific code branch.
---

## Preconditions

- If in plan mode, stop and ask to exit plan mode.

## Task

Increase the test coverage for the provided code branch.

## Workflow

Run these steps in order:

1. Replace the branch by a panic.
2. Run the existing tests:
    - If a test panics, then restore the branch and stop, the branch is already covered.
    - Else continue.
3. In `tmp/`, create an example that triggers the panic. You can run
   `cargo run --bin gpex -- compile tmp/ tmp/out.json` to verify if the panic is triggered.
    - If the branch is unreachable, then restore the branch and stop. Don't try to modify the
      production code to make the branch reachable.
    - Else continue.
4. Simplify the found example at maximum.
5. Create the same test in the most appropriate place in `tests/integration/`.
6. Run the tests to ensure panic is reached.
7. Rollback the panic.
8. Run the tests to ensure they are now passing.
