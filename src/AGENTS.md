# Source guidelines

- Implemented solutions must be simple, pragmatic, straightforward and easy to read/understand.
- Enforce Single Responsibility Principle for functions, modules, ...
- Define helper-method `impl` blocks in the same file as their type instead of creating external
  implementations elsewhere.
- Avoid wrapper types for light operations.
- Keep context-specific equality logic local to the operation that needs it.
