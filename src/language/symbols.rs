use crate::utils::parsing::Symbol;

pub(crate) const KEYWORDS: &[&str] =
    &[CONST_KEYWORD.slice, IMPORT_KEYWORD.slice, VAR_KEYWORD.slice];

pub(crate) const CONST_KEYWORD: Symbol = Symbol {
    name: "`const`",
    slice: "const",
};
pub(crate) const IMPORT_KEYWORD: Symbol = Symbol {
    name: "`import`",
    slice: "import",
};
pub(crate) const VAR_KEYWORD: Symbol = Symbol {
    name: "`var`",
    slice: "var",
};

pub(crate) const DOT_SYMBOL: Symbol = Symbol {
    name: "`.`",
    slice: ".",
};
pub(crate) const EQUAL_SYMBOL: Symbol = Symbol {
    name: "`=`",
    slice: "=",
};
pub(crate) const SEMICOLON_SYMBOL: Symbol = Symbol {
    name: "`;`",
    slice: ";",
};
