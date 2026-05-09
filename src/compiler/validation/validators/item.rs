use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::key_rendering::KeyRenderer;
use crate::compiler::parsing::exprs::calls::{OPERATOR_FN_NAME_PREFIX, UNARY_FN_NAMES};
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::prelude::PRELUDE_FILE_INDEX;
use crate::utils::indexing::{ItemNodeRef, NodeRef, SearchConfig, SearchParams, Visibility};
use crate::utils::parsing::span::{Span, SpanProps};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::{Log, LogInner, LogLevel};

pub(crate) fn check_circular_dependencies(
    item: ItemRef<'_>,
    dependency_result: Result<(), Vec<Span>>,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    let name_span = item.name_span();
    let name = context.slice(name_span);
    if let Err(stack) = dependency_result {
        if stack.iter().min() != Some(&stack[0]) {
            // avoid repeating the same error for each item of the stack
            return Err(ValidateError);
        }
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{name}` item has circular dependencies"),
            location: Some(context.location(name_span)),
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
    let search_params = SearchParams {
        key: &key,
        location: item,
        imports: &indexes.imports,
        config: SearchConfig {
            can_be_after: false,
            can_be_parent_node: false,
        },
    };
    if let Some(duplicated_item) = indexes
        .items
        .search(search_params, Visibility::Enforced)
        .next()
        && duplicated_item.file_index() == item.file_index()
    {
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{key}` item defined multiple times"),
            location: Some(context.location(name_span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: "item also defined here".into(),
                location: Some(context.location(duplicated_item.name_span())),
            }],
        });
        Err(ValidateError)
    } else {
        Ok(())
    }
}

pub(crate) fn check_unique_params(
    params: &[Param],
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    let mut is_error = false;
    for (param_index, param) in params.iter().enumerate() {
        let duplicated_param = params[..param_index]
            .iter()
            .find(|other_param| other_param.name == param.name);
        if let Some(duplicated_param) = duplicated_param {
            context.logs.push(Log {
                level: LogLevel::Error,
                msg: format!("`{}` parameter defined multiple times", param.name),
                location: Some(context.location(param.name_span)),
                inner: vec![LogInner {
                    level: LogLevel::Info,
                    msg: "parameter also defined here".into(),
                    location: Some(context.location(duplicated_param.name_span)),
                }],
            });
            is_error = true;
        }
    }
    if is_error { Err(ValidateError) } else { Ok(()) }
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
            msg: "forbidden `compilerimpl` item outside prelude".into(),
            location: Some(context.location(item.name_span())),
            inner: vec![],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_usage(
    item: ItemRef<'_>,
    displayed_key: &str,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) {
    let name_span = item.name_span();
    let name = context.slice(name_span);
    let ref_span = indexes.item_first_refs.get(&item.id());
    if !item.is_pub()
        && ref_span.is_none()
        && (!name.starts_with('_') || name.starts_with(OPERATOR_FN_NAME_PREFIX))
    {
        context.logs.push(Log {
            level: LogLevel::Warning,
            msg: format!("`{displayed_key}` item unused"),
            location: Some(context.location(name_span)),
            inner: vec![],
        });
    } else if item.is_pub() && name.starts_with('_') && !name.starts_with(OPERATOR_FN_NAME_PREFIX) {
        context.logs.push(Log {
            level: LogLevel::Warning,
            msg: format!("`{displayed_key}` item public but name starting with `_`"),
            location: Some(context.location(name_span)),
            inner: vec![],
        });
    } else if let Some(&ref_span) = ref_span
        && name.starts_with('_')
        && !name.starts_with(OPERATOR_FN_NAME_PREFIX)
    {
        context.logs.push(Log {
            level: LogLevel::Warning,
            msg: format!("`{displayed_key}` item used but name starting with `_`"),
            location: Some(context.location(name_span)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: "item used here".into(),
                location: Some(context.location(ref_span)),
            }],
        });
    }
}

pub(crate) fn check_found<'index>(
    node: impl NodeRef,
    span: Span,
    key: &str,
    displayed_key: &str,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'index>,
) -> Result<ItemRef<'index>, ValidateError> {
    if let Some(source) = indexes.sources.get(&node.id()) {
        Ok(*source)
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{displayed_key}` item not found"),
            location: Some(context.location(span)),
            inner: if let Some(priv_source) = indexes.priv_sources.get(&node.id()) {
                vec![LogInner {
                    level: LogLevel::Info,
                    msg: "item not qualified with `pub`".into(),
                    location: Some(context.location(priv_source.name_span())),
                }]
            } else {
                indexes
                    .items
                    .iter_by_key(key)
                    .filter(ItemNodeRef::is_pub)
                    .map(|item| LogInner {
                        level: LogLevel::Info,
                        msg: format!(
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

pub(crate) fn check_unary_operator_fn_params(
    fn_: &FnDefinition,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) -> Result<(), ValidateError> {
    if !UNARY_FN_NAMES.contains(&fn_.name.as_str()) || fn_.params.params.len() == 1 {
        Ok(())
    } else {
        let fn_key = KeyRenderer::new(indexes).fn_key(fn_)?;
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{fn_key}` unary operator function must have exactly one parameter"),
            location: Some(context.location(fn_.signature_span)),
            inner: vec![],
        });
        Err(ValidateError)
    }
}

pub(crate) fn check_unary_operator_fn_return_type(
    fn_: &FnDefinition,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) -> Result<(), ValidateError> {
    if !UNARY_FN_NAMES.contains(&fn_.name.as_str()) || fn_.return_type.is_some() {
        Ok(())
    } else {
        let fn_key = KeyRenderer::new(indexes).fn_key(fn_)?;
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{fn_key}` unary operator function without return type"),
            location: Some(context.location(fn_.signature_span)),
            inner: vec![],
        });
        Err(ValidateError)
    }
}
