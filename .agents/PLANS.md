## Planning format

When asked to plan, use this format exactly.

Rules:

- Keep the plan technical, verbose and easy to read.
- Do not use narrative prose.
- Do not include generic discovery steps unless needed to identify unknown files.
- Use `TBD after inspection` instead of guessing paths.
- Keep each bullet to one line when possible.
- Use only these action labels, except if irrelevant:
    - `add`
    - `modify`
    - `delete`
    - `move`
    - `rename`
    - `refactor`
- During plan creation, don't hesitate to ask architectural questions when an architectural decision
  is unclear.

### Plan

#### Scope

Short description of the scope of the task.

#### Technical changes

- `<action label>` `<area/component/module>` in `<repo relative path>`:
    - `<direct implementation change>`

#### Test changes

- `<action label>` test in `<repository relative path>`:
    - Description: short description of the test
    - Reason: reason why test is important

Be complete, but avoid duplicated/unnecessary tests.
Remain aligned with existing tests.
Prioritize adding straightforward tests.

#### Steps

- Implement previously described changes.
- Update all necessary Markdown files.
- Adapt changes to respect all root and nested AGENTS.md files.
- Ensure tests are passing.
- Ensure fast linters are passing.

#### Assumptions

- `<any useful assumption>`

#### Risks / limitations / unknowns

- `<concrete technical risk>` or `none`
