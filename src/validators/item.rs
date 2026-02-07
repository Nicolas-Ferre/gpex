use crate::compiler::indexes::Indexes;
use crate::compiler::prelude::PRELUDE_FILE_INDEX;
use crate::language::items::ItemRef;
use crate::utils::dependencies::Dependencies;
use crate::utils::indexing::{ItemNodeRef, NodeRef, SearchConfig, Visibility};
use crate::utils::parsing::{Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogInner, LogLevel};

pub(crate) fn check_circular_dependencies(
    item: ItemRef<'_>,
    dependencies: Result<Dependencies<ItemRef<'_>>, Vec<Span>>,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    let name_span = item.name_span();
    let name = context.slice(name_span);
    if let Err(stack) = dependencies {
        if stack.iter().min() != Some(&stack[0]) {
            // avoid repeating the same error for each item of the stack
            return Err(ValidateError);
        }
        context.logs.push(Log {
            level: LogLevel::Error,
            message: format!("`{name}` item has circular dependencies"),
            location: Some(context.location(name_span)),
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
                    location: Some(context.location(*ref_)),
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
    let name_span = item.name_span();
    let key = item.key();
    let search_config = SearchConfig {
        can_be_after: false,
        can_be_parent_node: false,
    };
    if let Some(duplicated_item) = indexes.items.search(
        &key,
        item,
        &indexes.imports,
        Visibility::Enforced,
        search_config,
    ) && duplicated_item.file_index() == item.file_index()
    {
        context.logs.push(Log {
            level: LogLevel::Error,
            message: format!("`{key}` item defined multiple times"),
            location: Some(context.location(name_span)),
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

pub(crate) fn check_prelude_location(
    item: ItemRef<'_>,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    if item.file_index() == PRELUDE_FILE_INDEX {
        Ok(())
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            message: "forbidden `compilerimpl` item outside prelude".into(),
            location: Some(context.location(item.name_span())),
            inner: vec![],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_usage(
    item: ItemRef<'_>,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) {
    if item.is_public() {
        return;
    }
    let name_span = item.name_span();
    let name = context.slice(name_span);
    let ref_span = indexes.item_first_refs.get(&item.id());
    if ref_span.is_none() && !name.starts_with('_') {
        context.logs.push(Log {
            level: LogLevel::Warning,
            message: format!("`{}` item unused", item.key()),
            location: Some(context.location(name_span)),
            inner: vec![],
        });
    } else if let Some(&ref_span) = ref_span
        && name.starts_with('_')
    {
        context.logs.push(Log {
            level: LogLevel::Warning,
            message: format!("`{}` item used but name starting with `_`", item.key()),
            location: Some(context.location(name_span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                message: "item used here".into(),
                location: Some(context.location(ref_span)),
            }],
        });
    }
}

pub(crate) fn check_found(
    node: impl NodeRef,
    span: Span,
    key: &str,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) -> Result<(), ValidateError> {
    if indexes.sources.contains_key(&node.id()) {
        Ok(())
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            message: format!("`{key}` item not found"),
            location: Some(context.location(span)),
            inner: if let Some(private_source) = indexes.private_sources.get(&node.id()) {
                vec![LogInner {
                    level: LogLevel::Info,
                    message: "item not qualified with `pub`".into(),
                    location: Some(context.location(private_source.name_span())),
                }]
            } else {
                indexes
                    .items
                    .iter_by_key(key)
                    .filter(ItemNodeRef::is_public)
                    .map(|item| LogInner {
                        level: LogLevel::Info,
                        message: format!(
                            "item can be imported from `{}`",
                            context.dot_path(item.file_index())
                        ),
                        location: Some(context.location(item.name_span())),
                    })
                    .collect()
            },
        });
        Err(ValidateError)
    }
}
