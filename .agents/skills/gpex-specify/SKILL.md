---
name: gpex-specify
description: Define all necessary open points to specify a new feature.
---

## Preconditions

- If not in plan mode, stop and ask to go in plan mode.

## Task

- Create the specifications of a feature by asking all necessary questions one by one. Follow
  instructions from `AGENTS.md` for how questions are displayed and using which method.
- Once validated by the user, save the specifications in `tmp/SPECS.md`.
- Never implement anything, only save the file.

## Specifications format

The final specifications should follow the following template:

````markdown
## Description

Short description of the issue with only the general idea of the issue.

## Dependencies

- List of dependency issues, only if already present.

## Details

### Short description of the aspect to specify

- A bullet list of important information about the aspect.

```zig,gpex
// One example that demonstrates the aspect. Keep it as minimal as possible.
```

### Other aspect

...

## Steps

- A bullet list of PR descriptions for each implementation step (make it as granular as possible,
  but each step must be end-to-end testable and isolated, so they will generally be transversal).
  Names should be short but contain enough context to be unique across the repository.
  The entire "Steps" section can be skipped if the issue has only one trivial step.
````

An example of description is available in `resources/example.md`.

Verbosity should be kept to a minimum to ease reading. Avoid AI marks such as em-dash (`—`).
Minimize the changes in case the issue is already well described, and avoid unnecessary rephrasing.
The number of bullets must be as small as possible: if multiple related bullets can be merged in a
simple bullet without losing information, please do it.
