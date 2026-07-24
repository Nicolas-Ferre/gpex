# Architecture

The compiler follows a multi-pass pipeline defined in `src/compiler/mod.rs`:

1. **Read**: Reads `.gpex` files from a folder recursively
2. **Parse**: Parse read files to produce an AST per file (called a "module")
   (`src/compiler/parsing/`)
3. **Index**: Builds symbol tables, e.g., to index imports and items for following stages
   (`src/compiler/indexing/`)
4. **Validate**: Semantic checks (type checking, circular dependency detection, naming
   conventions, ...) via `src/compiler/validation/`
5. **Transpile**: Converts ASTs to JSON file containing WGSL of each shader to execute
   (`src/compiler/transpilation/`)
6. **Run** (optional): Executes WGSL compute shaders on the GPU via wgpu (`src/runner/`)

Key directories and files:

- `src/compiler/`: Compilation pipeline orchestration and definition of each pipeline stage:
    - `state.rs`: Shared post-parse compiler state used by indexing, validation, value resolution,
      dependency analysis, and transpilation.
    - `item_ref.rs`: Shared item-reference representation used across compiler passes.
    - `parsing/`: AST definitions and parsing: modules, imports, items (functions, variables,
      structs, ...), expressions, statements. This layer also defines utility methods run on AST
      nodes.
    - `indexing/`: Symbol table construction (imports, item references, ...).
    - `validation/`: Semantic validation passes (type comparison, circular dependency
      detection, ...).
    - `validation/logs/`: Construction of user-facing compiler errors and warnings.
    - `transpilation/`: AST-to-WGSL conversion.
    - `dependencies.rs`: Item dependency resolution.
    - `key_rendering.rs`: Rendering of item keys for compiler logs.
    - `prelude.rs`: Built-in types and functions.
    - `refs.rs`: Reference checking (in this context, a reference is an expression that is permitted
      on the left-hand side of an assignment statement).
    - `consts/`: Constant value resolution.
    - `types.rs`: Type resolution.
- `src/runner/`: GPU execution using wgpu (device setup, shader dispatch, buffer readback).
- `src/utils/`: Reusable compilation utils, for file reading, parsing, logging, ...
- `res/prelude.gpex`: `GPEx` built-in types and functions.
