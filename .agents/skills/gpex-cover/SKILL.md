---
name: gpex-cover
description: Increase test coverage for a specific code branch.
---

## Preconditions

- If in plan mode, stop and ask to exit plan mode.

## Task

Increase the test coverage for the provided code branch.

## Workflow

- Replace the branch by a panic.
- Ensure that tests are not panicking, else it means the branch is already covered.
- In `tmp/`, create an example that triggers the panic. You can run
  `cargo run --bin gpex -- compile tmp/` to verify if the panic is triggered.
- If the branch is unreachable, just say it. Never modify production code.
- Simplify the example at maximum.
- Create the same test in the most appropriate place in `tests/integration/`.
- Rollback the panic.
