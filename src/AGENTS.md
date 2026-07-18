# Source guidelines

- Implemented solutions must be simple, pragmatic and straightforward to understand.
- Avoid large functions, keep them as small and readable as possible (< 20 lines, except when
  justified).
- Minimize function cognitive complexity
- Define helper-method `impl` blocks in the same file as their type instead of creating external
  implementations elsewhere.
- Don't qualify Rust type names with their parent modules (e.g. `Type` instead of
  `crate::module::Type`)
- Avoid wrapper types for light operations.
- Keep context-specific equality logic local to the operation that needs it.
