pub(crate) mod exprs;
pub(crate) mod items;
pub(crate) mod modules;
pub(crate) mod statements;

use crate::Log;
use crate::compiler::transversal::ast::COMMENT_PREFIX;
use crate::compiler::transversal::ast::modules::Module;
use crate::compiler::transversal::ast::symbols::{COMMA_SYMBOL, PARENTHESIS_CLOSE_SYMBOL};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::Span;
use crate::utils::reading::ReadFile;
use std::path::Path;

pub(crate) fn parse(root_path: &Path, files: &[ReadFile]) -> Result<Vec<Module>, Vec<Log>> {
    let mut next_id = 0;
    let mut modules = vec![];
    let mut errors = vec![];
    for (file_index, file) in files.iter().enumerate() {
        let mut context =
            ParseContext::new(root_path, file, file_index, files, next_id, COMMENT_PREFIX);
        match modules::parse(&mut context) {
            Ok(module) => modules.push(module),
            Err(error) => errors.push(error.to_error()),
        }
        next_id = context.next_id();
    }
    if errors.is_empty() {
        Ok(modules)
    } else {
        Err(errors)
    }
}

fn arg_stop_excluded_parser<'context>(
    context: &mut ParseContext<'context>,
) -> Result<(), ParseError<'context>> {
    context
        .parse_any(&[
            &|context| Span::parse_symbol(context, COMMA_SYMBOL),
            &|context| Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL),
        ])
        .map(|_| ())
}
