export MAX_RUST_FILE_LINE_COUNT=250

# shellcheck disable=SC2034 # used by scripts sourcing this file
export EXCLUDED_FUNCTIONS=()
# shellcheck disable=SC2034 # used by scripts sourcing this file
export EXCLUDED_TYPE_PATHS=(
    "fmt::Result"
    "io::Error"
)

# General rule: if acronym is not idiomatic in Rust or plural doesn't seem natural, then don't use it.
export FORBIDDEN_WORDS=(
    "identifier"     # use "ident" or "id" instead
    "reference"      # use "ref" instead
    "argument"       # use "arg" instead
    "parameter"      # use "param" instead
    "expression"     # use "expr" instead
    "character"      # use "char" instead
    "initialize"     # use "init" instead
    "initialization" # use "init" instead
    "length"         # use "len" instead
    "file_system"    # use "fs" instead
    "configuration"  # use "config" instead
    "constant"       # use "const" instead
    "variable"       # use "var" instead
    "function"       # use "fn" instead
    "func"           # use "fn" instead
    "directory"      # use "dir" instead
    "folder"         # use "dir" instead
    "public"         # use "pub" instead
    "private"        # use "priv" instead
    "extension"      # use "ext" instead
    "properties"     # use "props" instead
    "property"       # use "prop" instead
    "message"        # use "msg" instead
    "err"            # use "error" instead
    "ctx"            # use "context" instead
    "semi"           # use "semicolon" instead
    "loc"            # use "location" instead
    "def"            # use "definition" instead
    "dep"            # use "dependency" instead
    "deps"           # use "dependencies" instead
    "sym"            # use "symbol" instead
    "req"            # use "required" or "request" instead
    "kw"             # use "keyword" instead
    "lag"            # use "language" instead
    "sig"            # use "signature" instead
    "stmt"           # use "statement" instead
    "lit"            # use "literal" instead
    "prev"           # use "previous" instead
    "indexation"     # use "indexing" instead
)
