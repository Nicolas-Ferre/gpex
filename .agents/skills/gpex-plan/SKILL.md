---
name: gpex-plan
description: Make plans more reliable.
---

## Plan improvements

The plan must include the following explicit final steps:

- Ensure conventions defined in all `AGENTS.md` files are respected, and adapt changes if this is
  not the case.
- Ensure tests are passing: `cargo test --no-fail-fast`
- Ensure Clippy is passing: `cargo clippy --all-targets --no-deps -- -D warnings`
- Ensure linters are passing: `bash .github/scripts/run_all_lints.sh`
