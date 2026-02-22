# Architecture

The compiler follows a multi-pass pipeline defined in `src/compiler/compilation.rs`:

1. **Read**: Reads `.gpex` files from a folder recursively
2. **Parse**: Parse read files to produce an AST per file (called a "module")
3. **Index**: Builds symbol tables, e.g., to index imports and items for following stages
4. **Validate**: Semantic checks (type checking, circular dependency detection, naming
   conventions, ...) via `src/validators/`
5. **Transpile**: Converts ASTs to JSON file containing WGSL of each shader to execute
   (`src/compiler/transpilation.rs`)
6. **Run** (optional): Executes WGSL compute shaders on the GPU via wgpu (`src/runner/`)

Key directories and files:

- `src/compiler/`: Compilation pipeline orchestration, indexing, constant evaluation, prelude
- `src/language/`: AST definitions: modules, imports, items (functions, variables, structs, ...),
  expressions, statements
- `src/validators/`: Semantic validation passes (type comparison, circular dependency
  detection, ...)
- `src/runner/`: GPU execution using wgpu (device setup, shader dispatch, buffer readback)
- `src/utils/`: Reusable compilation utils, for file reading, parsing, logging, ...
- `res/prelude.gpex`: `GPEx` built-in types and functions
