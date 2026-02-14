# Architecture

The compiler follows a multi-pass pipeline defined in `compiler/compilation.rs`:

1. **Parse** — Reads `.gpex` files from a folder recursively, produces AST per module
2. **Index** — Builds symbol tables for imports, variables, constants, structs, functions
3. **Validate** — Semantic checks (type checking, circular dependency detection, naming conventions)
   via `validators/`
4. **Transpile** — Converts AST to WGSL output (`compiler/transpilation.rs`)
5. **Run** (optional) — Executes WGSL compute shaders on the GPU via wgpu (`runner/`)

Key directories:

- `compiler/` — Compilation pipeline orchestration, indexing, constant evaluation, prelude
- `language/` — AST definitions: modules, imports, items (functions, variables, constants,
  structs), expressions, statements
- `validators/` — Semantic validation passes (types, identifiers, imports, literals, circular deps)
- `runner/` — GPU execution using wgpu (device setup, shader dispatch, buffer readback)
- `utils/` — Parsing context/spans, logging, file reading, dependency graphs, validation context
- `../res/prelude.gpex` — Built-in types and functions
