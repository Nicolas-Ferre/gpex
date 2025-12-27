use crate::compiler::dependencies::Dependencies;
use crate::compiletools::parsing::Span;
use crate::compiletools::validation::{ValidateCtx, ValidateError};
use crate::{Log, LogInner, LogLevel};

pub(crate) fn check_circular_dependencies(
    name: &Span,
    dependencies: Result<Dependencies<'_>, Vec<Span>>,
    ctx: &mut ValidateCtx<'_>,
) -> Result<(), ValidateError> {
    if let Err(stack) = dependencies {
        if stack.iter().any(|ref_| &stack[0] > ref_) {
            // avoid repeating the same error for each item of the stack
            return Err(ValidateError);
        }
        ctx.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{}` item has circular dependencies", name.slice),
            loc: Some(ctx.loc(name)),
            inner: stack
                .iter()
                .enumerate()
                .map(|(index, ref_)| LogInner {
                    level: LogLevel::Info,
                    msg: if index == stack.len() - 1 {
                        "depends on itself".into()
                    } else {
                        "depends on this item".into()
                    },
                    loc: Some(ctx.loc(ref_)),
                })
                .collect(),
        });
        Err(ValidateError)
    } else {
        Ok(())
    }
}
