---
name: gpex-plan
description: Define all necessary open points to plan a change.
---

## Preconditions

- If not in plan mode, stop and ask to go in plan mode.

## Task

Perform technical design of the solution by asking all necessary questions to ensure alignment
on the following:

- Scope of the task
- Chosen architecture
- Technical implementation choices
- Code organization
- Test coverage and location

Follow instructions from `AGENTS.md` for how questions are displayed and using which method.

The plan must include the following explicit final steps:

- Ensure conventions defined in all `AGENTS.md` files are respected, and adapt changes if this is
  not the case.
- Ensure tests are passing: `cargo test --no-fail-fast`
- Ensure linters are passing: `bash .github/scripts/run_all_lints.sh`
