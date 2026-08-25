---
name: gpex-review
description: Perform extensive code review of the current branch.
---

## Preconditions

- If in plan mode, stop and ask to exit plan mode.
- Do not run this skill if not explicitly requested by the user.

## Task

Review the changes of the current branch (compared to `main` branch).

## Workflow

Run these steps in order:

1. Run the sub-agents defined later.
2. Once all sub-agents are done, list only the found issues the following way:

```markdown
1. [<subagent name>] **<first finding>**
   <brief details and short snippet(s) of the proposed fix>
2. [<subagent name>] **<second finding>**
   <brief details and short snippet(s) of the proposed fix>
   ...
```

Don't hesitate to illustrate the findings with snippets to make it easier to understand.

## Sub-agents

Each subsection corresponds to a sub-agent to run during review.

The subagents must return the full list of findings.

### Bug detection

- Check all bugs related to the current branch changes.
- Do not miss any bug.
- If needed, bugs can be tested in `tmp/main.gpex`.

### Architecture review

Challenge the architecture and verify whether:

- the solution is future-proof and will not cause issues with future changes in the project
- the solution generalizes well and avoids uncovered edge cases
- the solution is not unnecessarily complicated

### Tests review

Check for missing tests, for example related to:

- An uncovered code branch
- An uncovered case specified in the GitHub issue and related to the current changes
- An existing test that doesn't follow general conventions defined in documentation and AGENTS.md
  files

### Code quality check

Check local code quality issues, such as:

- readability/maintainability issues
- inconsistent styling
- bad practices
- performance improvements
- not followed conventions defined in `AGENTS.md` files
- not up-to-date documentation
- typos or weird formulations
