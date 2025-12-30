use crate::compiler::dependencies::Dependencies;
use crate::compiler::indexes::Indexes;
use crate::language::items::ItemRef;
use crate::utils::indexing::NodeRef;
use crate::utils::parsing::Span;
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogInner, LogLevel};

pub(crate) fn check_circular_dependencies(
    item: ItemRef<'_>,
    dependencies: Result<Dependencies<'_>, Vec<Span>>,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    let name = item.name_span();
    if let Err(stack) = dependencies {
        if stack.iter().any(|ref_| &stack[0] > ref_) {
            // avoid repeating the same error for each item of the stack
            return Err(ValidateError);
        }
        context.logs.push(Log {
            level: LogLevel::Error,
            message: format!("`{}` item has circular dependencies", name.slice),
            location: Some(context.location(name)),
            inner: stack
                .iter()
                .enumerate()
                .map(|(index, ref_)| LogInner {
                    level: LogLevel::Info,
                    message: if index == stack.len() - 1 {
                        "depends on itself".into()
                    } else {
                        "depends on this item".into()
                    },
                    location: Some(context.location(ref_)),
                })
                .collect(),
        });
        Err(ValidateError)
    } else {
        Ok(())
    }
}

pub(crate) fn check_unique_definition(
    item: ItemRef<'_>,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) -> Result<(), ValidateError> {
    let name = item.name_span();
    if let Some(duplicated_item) = indexes.items.search(&name.slice, item, &indexes.imports)
        && duplicated_item.file_index() == item.file_index()
    {
        context.logs.push(Log {
            level: LogLevel::Error,
            message: format!("`{}` item defined multiple times", name.slice),
            location: Some(context.location(name)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                message: "item also defined here".into(),
                location: Some(context.location(duplicated_item.name_span())),
            }],
        });
        Err(ValidateError)
    } else {
        Ok(())
    }
}

pub(crate) fn check_usage(
    item: ItemRef<'_>,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) {
    let name = item.name_span();
    let ref_span = indexes.item_first_refs.get(&item.id());
    if ref_span.is_none() && !name.slice.starts_with('_') {
        context.logs.push(Log {
            level: LogLevel::Warning,
            message: format!("`{}` value unused", name.slice),
            location: Some(context.location(name)),
            inner: vec![],
        });
    } else if let Some(ref_span) = ref_span
        && name.slice.starts_with('_')
    {
        context.logs.push(Log {
            level: LogLevel::Warning,
            message: format!("`{}` value used but name starting with `_`", name.slice),
            location: Some(context.location(name)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                message: "value used here".into(),
                location: Some(context.location(ref_span)),
            }],
        });
    }
}
