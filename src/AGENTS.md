# Source guidelines

- Implemented solutions must be simple, pragmatic and straightforward to understand.
- Keep functions as small and readable as possible, avoid large functions.
- Minimize function cognitive complexity
- Define helper-method `impl` blocks in the same file as their type instead of creating external
  implementations elsewhere.
- Don't qualify Rust type names with their parent modules (e.g. `Type` instead of
  `crate::module::Type`)
- Functions must always be qualified with their parent module.
- Don't define structs in the middle of standalone functions.
- Favor mutable methods (`&mut self`) before their corresponding immutable ones (`&self`).
- Avoid wrapper types for light operations.
- Keep context-specific equality logic local to the operation that needs it.
