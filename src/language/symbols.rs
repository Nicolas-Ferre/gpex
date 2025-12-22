use crate::compiletools::parsing::Symbol;

pub(crate) const KEYWORDS: &[&str] = &[VAR_SYM.slice];

pub(crate) const VAR_SYM: Symbol = Symbol {
    name: "`var`",
    slice: "var",
};

pub(crate) const EQ_SYM: Symbol = Symbol {
    name: "`=`",
    slice: "=",
};
pub(crate) const SEMI_SYM: Symbol = Symbol {
    name: "`;`",
    slice: ";",
};
