use crate::compiletools::parsing::Symbol;

pub(crate) const KEYWORDS: &[&str] = &[CONST_SYM.slice, IMPORT_SYM.slice, VAR_SYM.slice];

pub(crate) const CONST_SYM: Symbol = Symbol {
    name: "`const`",
    slice: "const",
};
pub(crate) const IMPORT_SYM: Symbol = Symbol {
    name: "`import`",
    slice: "import",
};
pub(crate) const VAR_SYM: Symbol = Symbol {
    name: "`var`",
    slice: "var",
};

pub(crate) const DOT_SYM: Symbol = Symbol {
    name: "`.`",
    slice: ".",
};
pub(crate) const EQ_SYM: Symbol = Symbol {
    name: "`=`",
    slice: "=",
};
pub(crate) const SEMI_SYM: Symbol = Symbol {
    name: "`;`",
    slice: ";",
};
