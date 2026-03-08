use crate::utils::parsing::Symbol;

pub(crate) const KEYWORDS: &[&str] = &[
    COMPILERIMPL_KEYWORD.slice,
    CONST_KEYWORD.slice,
    FALSE_KEYWORD.slice,
    FN_KEYWORD.slice,
    IMPORT_KEYWORD.slice,
    PUB_KEYWORD.slice,
    REPEAT_KEYWORD.slice,
    RETURN_KEYWORD.slice,
    STRUCT_KEYWORD.slice,
    TRUE_KEYWORD.slice,
    VAR_KEYWORD.slice,
];

pub(crate) const COMPILERIMPL_KEYWORD: Symbol = Symbol {
    name: "`compilerimpl`",
    slice: "compilerimpl",
};
pub(crate) const CONST_KEYWORD: Symbol = Symbol {
    name: "`const`",
    slice: "const",
};
pub(crate) const FALSE_KEYWORD: Symbol = Symbol {
    name: "`false`",
    slice: "false",
};
pub(crate) const FN_KEYWORD: Symbol = Symbol {
    name: "`fn`",
    slice: "fn",
};
pub(crate) const IMPORT_KEYWORD: Symbol = Symbol {
    name: "`import`",
    slice: "import",
};
pub(crate) const PUB_KEYWORD: Symbol = Symbol {
    name: "`pub`",
    slice: "pub",
};
pub(crate) const REPEAT_KEYWORD: Symbol = Symbol {
    name: "`repeat`",
    slice: "repeat",
};
pub(crate) const RETURN_KEYWORD: Symbol = Symbol {
    name: "`return`",
    slice: "return",
};
pub(crate) const STRUCT_KEYWORD: Symbol = Symbol {
    name: "`struct`",
    slice: "struct",
};
pub(crate) const TRUE_KEYWORD: Symbol = Symbol {
    name: "`true`",
    slice: "true",
};
pub(crate) const VAR_KEYWORD: Symbol = Symbol {
    name: "`var`",
    slice: "var",
};

pub(crate) const ARROW_SYMBOL: Symbol = Symbol {
    name: "`->`",
    slice: "->",
};
pub(crate) const BRACE_OPEN_SYMBOL: Symbol = Symbol {
    name: "`{`",
    slice: "{",
};
pub(crate) const BRACE_CLOSE_SYMBOL: Symbol = Symbol {
    name: "`}`",
    slice: "}",
};
pub(crate) const COLON_SYMBOL: Symbol = Symbol {
    name: "`:`",
    slice: ":",
};
pub(crate) const COMMA_SYMBOL: Symbol = Symbol {
    name: "`,`",
    slice: ",",
};
pub(crate) const DOT_SYMBOL: Symbol = Symbol {
    name: "`.`",
    slice: ".",
};
pub(crate) const EQUAL_SYMBOL: Symbol = Symbol {
    name: "`=`",
    slice: "=",
};
pub(crate) const PARENTHESIS_OPEN_SYMBOL: Symbol = Symbol {
    name: "`(`",
    slice: "(",
};
pub(crate) const PARENTHESIS_CLOSE_SYMBOL: Symbol = Symbol {
    name: "`)`",
    slice: ")",
};
pub(crate) const SEMICOLON_SYMBOL: Symbol = Symbol {
    name: "`;`",
    slice: ";",
};
pub(crate) const TILDE_SYMBOL: Symbol = Symbol {
    name: "`~`",
    slice: "~",
};
