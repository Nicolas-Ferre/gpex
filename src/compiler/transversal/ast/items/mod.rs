pub(crate) mod actions;
pub(crate) mod fns;
pub(crate) mod imports;
pub(crate) mod params;
pub(crate) mod types;
pub(crate) mod vars;

use crate::compiler::transversal::ast::items::fns::FnDefinition;
use crate::compiler::transversal::ast::items::types::StructDefinition;
use crate::compiler::transversal::ast::items::vars::{ConstDefinition, VarDefinition};
use actions::RepeatDefinition;
use imports::Import;

#[derive(Debug)]
pub(crate) enum Item {
    Import(Import),
    Var(VarDefinition),
    Const(ConstDefinition),
    Struct(StructDefinition),
    Fn(Box<FnDefinition>),
    Repeat(RepeatDefinition),
}
