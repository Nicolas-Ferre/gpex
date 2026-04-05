# General rule: if acronym is not idiomatic in Rust or plural doesn't seem natural, then don't use it.
FORBIDDEN_WORDS=(
    "identifier"     # "ident" or "id" is preferred
    "reference"      # "ref" is preferred
    "argument"       # "arg" is preferred
    "parameter"      # "param" is preferred
    "expression"     # "expr" is preferred
    "character"      # "char" is preferred
    "initialize"     # "init" is preferred
    "initialization" # "init" is preferred
    "length"         # "len" is preferred
    "file_system"    # "fs" is preferred
    "configuration"  # "config" is preferred
    "constant"       # "const" is preferred
    "variable"       # "var" is preferred
    "function"       # "fn" is preferred
    "func"           # "fn" is preferred
    "directory"      # "dir" is preferred
    "folder"         # "dir" is preferred
    "public"         # "pub" is preferred
    "private"        # "priv" is preferred
    "extension"      # "ext" is preferred
    "properties"     # "props" is preferred
    "property"       # "prop" is preferred
    "message"        # "msg" is preferred
    "err"            # "error" is preferred
    "ctx"            # "context" is preferred
    "semi"           # "semicolon" is preferred
    "loc"            # "location" is preferred
    "def"            # "definition" is preferred
    "dep"            # "dependency" is preferred
    "deps"           # "dependencies" is preferred
    "sym"            # "symbol" is preferred
    "req"            # "required" or "request" is preferred
    "kw"             # "keyword" is preferred
    "lag"            # "language" is preferred
    "sig"            # "signature" is preferred
    "stmt"           # "statement" is preferred
    "lit"            # "literal" is preferred
    "prev"           # "previous" is preferred
)
