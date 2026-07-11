use crate::compiler::indexing::indexes::Indexes;
use crate::compiler::indexing::item_ref::ItemRef;
use crate::compiler::key_rendering::KeyRenderer;
use crate::compiler::parsing::exprs::calls::UNARY_FN_NAMES;
use crate::compiler::parsing::exprs::{BINARY_FN_NAMES, OPERATOR_FN_NAME_PREFIX};
use crate::compiler::parsing::items::fns::FnDefinition;
use crate::compiler::parsing::items::params::Param;
use crate::compiler::prelude::PRELUDE_FILE_INDEX;
use crate::compiler::values::ValueResolver;
use crate::compiler::values::types::Type;
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
        .search_in_same_file(search_params, Visibility::Enforced)
        .next()
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

pub(crate) fn check_unique_fn_signature(
    fn_: &FnDefinition,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) {
    if let Some(previous_fn) = find_previous_same_fn_signature(fn_, indexes) {
        let fn_key = KeyRenderer::new(indexes)
            .fn_key(fn_)
            .unwrap_or_else(|_| unreachable!("function should be validated before"));
        context.logs.push(Log {
            level: LogLevel::Warning,
            msg: format!("`{fn_key}` function defined multiple times"),
            location: Some(context.location(fn_.signature_span_without_return)),
            inner: vec![LogInner {
                level: LogLevel::Info,
                msg: "function also defined here".into(),
                location: Some(context.location(previous_fn.signature_span_without_return)),
            }],
        });
    }
}

pub(crate) fn check_prelude_location(
    item: ItemRef<'_>,
    compilerimpl_keyword_span: Option<Span>,
    context: &mut ValidateContext<'_>,
) -> Result<(), ValidateError> {
    if let Some(compilerimpl_keyword_span) = compilerimpl_keyword_span
        && item.file_index() != PRELUDE_FILE_INDEX
    {
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: "forbidden `compilerimpl` item outside prelude".into(),
            location: Some(context.location(compilerimpl_keyword_span)),
            inner: vec![],
        });
        Err(ValidateError)
    } else {
        Ok(())
    }
}

pub(crate) fn check_usage(
    item: ItemRef<'_>,
    context: &mut ValidateContext<'_>,
    key_renderer: &mut KeyRenderer<'_, '_>,
    indexes: &Indexes<'_>,
) {
    let name_span = item.name_span();
    let name = context.slice(name_span);
    let ref_span = indexes.item_first_refs.get(&item.id());
    let is_unused_lint_ignored =
        name.starts_with('_') && !name.starts_with(OPERATOR_FN_NAME_PREFIX);
    if !item.is_pub() && ref_span.is_none() && !is_unused_lint_ignored {
        let displayed_key = item.displayed_key(key_renderer);
        context.logs.push(Log {
            level: LogLevel::Warning,
            msg: format!("`{displayed_key}` item unused"),
            location: Some(context.location(name_span)),
            inner: vec![],
        });
    } else if item.is_pub() && is_unused_lint_ignored {
        let displayed_key = item.displayed_key(key_renderer);
        context.logs.push(Log {
            level: LogLevel::Warning,
            msg: format!("`{displayed_key}` item public but name starting with `_`"),
            location: Some(context.location(name_span)),
            inner: vec![],
        });
    } else if let Some(&ref_span) = ref_span
        && is_unused_lint_ignored
    {
        let displayed_key = item.displayed_key(key_renderer);
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
    source: Option<ItemRef<'index>>,
    node: impl NodeRef,
    span: Span,
    key: &str,
    displayed_key: &str,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'index>,
) -> Result<ItemRef<'index>, ValidateError> {
    if let Some(source) = source {
        Ok(source)
    } else {
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{displayed_key}` item not found"),
            location: Some(context.location(span)),
            inner: if let Some(candidates) = indexes.candidate_sources.get(&node.id()) {
                candidates
                    .iter()
                    .map(|candidate| LogInner {
                        level: LogLevel::Info,
                        msg: "similar candidate".into(),
                        location: Some(context.location(candidate.signature_span_with_return())),
                    })
                    .collect()
            } else if let Some(priv_source) = indexes.priv_sources.get(&node.id()) {
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

pub(crate) fn check_unary_operator_fn(
    fn_: &FnDefinition,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) -> Result<(), ValidateError> {
    if !UNARY_FN_NAMES.contains(&fn_.name.as_str()) {
        Ok(())
    } else if fn_.params.params.len() != 1 {
        let fn_key = KeyRenderer::new(indexes).fn_key(fn_)?;
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{fn_key}` unary operator function must have exactly one parameter"),
            location: Some(context.location(fn_.signature_span_with_return)),
            inner: vec![],
        });
        Err(ValidateError)
    } else if fn_.return_type.is_none() {
        let fn_key = KeyRenderer::new(indexes).fn_key(fn_)?;
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{fn_key}` unary operator function without return type"),
            location: Some(context.location(fn_.signature_span_with_return)),
            inner: vec![],
        });
        Err(ValidateError)
    } else {
        Ok(())
    }
}

pub(crate) fn check_binary_operator_fn(
    fn_: &FnDefinition,
    context: &mut ValidateContext<'_>,
    indexes: &Indexes<'_>,
) -> Result<(), ValidateError> {
    if !BINARY_FN_NAMES.contains(&fn_.name.as_str()) {
        Ok(())
    } else if fn_.params.params.len() != 2 {
        let fn_key = KeyRenderer::new(indexes).fn_key(fn_)?;
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{fn_key}` binary operator function must have exactly two parameters"),
            location: Some(context.location(fn_.signature_span_with_return)),
            inner: vec![],
        });
        Err(ValidateError)
    } else if fn_.return_type.is_none() {
        let fn_key = KeyRenderer::new(indexes).fn_key(fn_)?;
        context.logs.push(Log {
            level: LogLevel::Error,
            msg: format!("`{fn_key}` binary operator function without return type"),
            location: Some(context.location(fn_.signature_span_with_return)),
            inner: vec![],
        });
        Err(ValidateError)
    } else {
        Ok(())
    }
}

fn find_previous_same_fn_signature<'index>(
    fn_: &FnDefinition,
    indexes: &'index Indexes<'_>,
) -> Option<&'index FnDefinition> {
    if fn_.has_requirement() {
        return None;
    }
    let search_params = SearchParams {
        key: &fn_.key(),
        location: ItemRef::Fn(fn_),
        imports: &indexes.imports,
        config: SearchConfig {
            can_be_after: false,
            can_be_parent_node: false,
        },
    };
    indexes
        .items
        .search_in_same_file(search_params, Visibility::Enforced)
        .map(|item| match item {
            ItemRef::Fn(previous_fn) => previous_fn,
            ItemRef::Var(_) | ItemRef::Const(_) | ItemRef::Struct(_) | ItemRef::Param(_) => {
                unreachable!("only functions are searched with parameter types")
            }
        })
        .filter(|previous_fn| !previous_fn.has_requirement())
        .find(|previous_fn| are_same_fn_signatures(fn_, previous_fn, indexes))
}

fn are_same_fn_signatures(
    fn_: &FnDefinition,
    other_fn: &FnDefinition,
    indexes: &Indexes<'_>,
) -> bool {
    debug_assert!(fn_.name == other_fn.name);
    debug_assert!(fn_.params.params.len() == other_fn.params.params.len());
    let mut value_resolver = ValueResolver::new(indexes);
    let mut other_value_resolver = ValueResolver::new(indexes);
    fn_.params
        .params
        .iter()
        .zip(&other_fn.params.params)
        .all(|(param, other_param)| {
            let type_ = value_resolver.param_type(param);
            let other_type = other_value_resolver.param_type(other_param);
            are_same_param_types(type_, other_type, fn_, other_fn)
        })
}

fn are_same_param_types(
    type_: Type<'_>,
    other_type: Type<'_>,
    fn_: &FnDefinition,
    other_fn: &FnDefinition,
) -> bool {
    match (type_, other_type) {
        (Type::Struct(struct_), Type::Struct(other_struct)) => struct_.id == other_struct.id,
        (Type::Param(param), Type::Param(other_param))
        | (Type::Wildcard(param), Type::Wildcard(other_param)) => {
            param_index(fn_, param) == param_index(other_fn, other_param)
        }
        _ => false,
    }
}

fn param_index(fn_: &FnDefinition, param: &Param) -> usize {
    fn_.params
        .params
        .iter()
        .position(|fn_param| fn_param.id == param.id)
        .unwrap_or_else(|| unreachable!("param should be found in the function"))
}
