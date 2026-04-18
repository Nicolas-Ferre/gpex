use crate::compiler::consts::{ConstContext, ConstValue};
use crate::compiler::indexes::Indexes;
use crate::compiler::types::Type;
use crate::language::DependencyType;
use crate::language::exprs::Expr;
use crate::language::items::ItemRef;
use crate::language::patterns::IDENT_PATTERN;
use crate::language::symbols::{COMMA_SYMBOL, PARENTHESIS_CLOSE_SYMBOL, PARENTHESIS_OPEN_SYMBOL};
use crate::utils::dependencies::Dependencies;
use crate::utils::indexing::{NodeRef, SearchConfig, SearchParams, Visibility};
use crate::utils::parsing::context::ParseContext;
use crate::utils::parsing::error::ParseError;
use crate::utils::parsing::span::{Span, SpanProps};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;
use itertools::Itertools;

const SEARCH_CONFIG: SearchConfig = SearchConfig {
    can_be_after: true,
    can_be_parent_node: true,
};

#[derive(Debug)]
pub(crate) struct Call {
    id: u64,
    scope: Vec<u64>,
    pub(crate) span: Span,
    pub(crate) name: String,
    args: Vec<Expr>,
}

impl NodeRef for &Call {
    fn file_index(&self) -> usize {
        self.span.file_index
    }

    fn id(&self) -> u64 {
        self.id
    }

    // coverage: off (unused because function can be called in itself)
    fn scope(&self) -> &[u64] {
        &self.scope
    }
    // coverage: on
}

impl Call {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let name_span = Span::parse_pattern(context, IDENT_PATTERN)?;
        Span::parse_symbol(context, PARENTHESIS_OPEN_SYMBOL)?;
        context.force_parse_any_error();
        let args = context.parse_many(
            Expr::parse,
            Some(|context| Span::parse_symbol(context, COMMA_SYMBOL).map(|_| ())),
            |context| Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL).map(|_| ()),
        )?;
        let end_span = Span::parse_symbol(context, PARENTHESIS_CLOSE_SYMBOL)?;
        Ok(Self {
            id: context.next_id(),
            scope: context.scope().to_vec(),
            span: name_span.until(end_span),
            name: context.slice(name_span).into(),
            args,
        })
    }

    fn key(&self) -> String {
        format!("{}({})", self.name, self.args.len())
    }

    fn displayed_key(&self, indexes: &Indexes<'_>) -> String {
        let fn_name = &self.name;
        let arg_types = self
            .args
            .iter()
            .map(|arg| arg.type_(indexes).name())
            .join(", ");
        format!("{fn_name}({arg_types})")
    }

    pub(crate) fn index(&self, indexes: &mut Indexes<'_>) {
        for arg in &self.args {
            arg.index(indexes);
        }
        let search_params = SearchParams {
            key: &self.key(),
            location: self,
            imports: &indexes.imports,
            config: SEARCH_CONFIG,
        };
        let matching_fn = indexes
            .items
            .search(search_params, Visibility::Enforced)
            .find(|item| item.has_same_param_types_as(&self.args, indexes));
        if let Some(source) = matching_fn {
            indexes.sources.insert(self.id, source);
            indexes
                .imports
                .mark_as_used(self.file_index(), source.file_index());
            indexes
                .item_first_refs
                .entry(source.id())
                .or_insert_with(|| self.span);
        } else if let Some(source) = indexes
            .items
            .search(search_params, Visibility::Ignored)
            .find(|item| item.has_same_param_types_as(&self.args, indexes))
        {
            indexes.priv_sources.insert(self.id, source);
        }
    }

    fn are_args_const(&self, is_in_const_fn: bool, indexes: &Indexes<'_>) -> bool {
        self.args
            .iter()
            .all(|arg| arg.is_const(is_in_const_fn, indexes))
    }

    pub(crate) fn dependencies<'index>(
        &self,
        is_in_const_fn: bool,
        type_: DependencyType,
        mut dependencies: Dependencies<ItemRef<'index>>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<ItemRef<'index>>, Vec<Span>> {
        for arg in &self.args {
            dependencies = arg.dependencies(is_in_const_fn, type_, dependencies, indexes)?;
        }
        if let Some(&source) = indexes.sources.get(&self.id) {
            dependencies.register(self.span, source, |dependencies| {
                source.dependencies(
                    is_in_const_fn,
                    self.are_args_const(is_in_const_fn, indexes),
                    type_,
                    dependencies,
                    indexes,
                )
            })
        } else {
            Ok(dependencies)
        }
    }

    pub(crate) fn type_<'index>(&self, indexes: &Indexes<'index>) -> Type<'index> {
        match indexes.sources.get(&self.id) {
            Some(source) => source.type_(indexes),
            None => Type::Unknown,
        }
    }

    pub(crate) fn is_const(&self, is_in_const_fn: bool, indexes: &Indexes<'_>) -> bool {
        indexes
            .sources
            .get(&self.id)
            .is_some_and(|source| source.is_const(is_in_const_fn))
            && self.are_args_const(is_in_const_fn, indexes)
    }

    pub(crate) fn const_value<'index>(
        &self,
        indexes: &Indexes<'index>,
        context: &mut ConstContext<'index>,
    ) -> ConstValue<'index> {
        match indexes.sources.get(&self.id) {
            Some(ItemRef::Fn(source)) => {
                let param_args = self
                    .args
                    .iter()
                    .zip(&source.params.params)
                    .map(|(arg, param)| (param.id, arg.const_value(indexes, context)))
                    .collect::<Vec<_>>();
                context.run_scoped(|context| {
                    for (param_id, arg_value) in param_args {
                        match arg_value {
                            ConstValue::TypeRef(_)
                            | ConstValue::I32(_)
                            | ConstValue::U32(_)
                            | ConstValue::F32(_)
                            | ConstValue::Bool(_) => context.add_value(param_id, arg_value),
                            ConstValue::Unknown | ConstValue::RuntimeValue => return arg_value,
                        }
                    }
                    source.const_value(indexes, context)
                })
            }
            Some(
                ItemRef::Variable(_)
                | ItemRef::Constant(_)
                | ItemRef::Struct(_)
                | ItemRef::Param(_),
            ) => {
                unreachable!("identifier should not refer to a value")
            }
            None => ConstValue::Unknown,
        }
    }

    pub(crate) fn validate(
        &self,
        const_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        for arg in &self.args {
            arg.validate(const_mark_span, context, indexes)?;
        }
        let source = validators::item::check_found(
            self,
            self.span,
            &self.key(),
            &self.displayed_key(indexes),
            context,
            indexes,
        )?;
        if let Some(const_mark_span) = const_mark_span {
            validators::expr::check_const_value(source, self.span, const_mark_span, context)?;
        }
        Ok(())
    }

    pub(crate) fn transpile<'index>(
        &self,
        shader: &mut String,
        indexes: &Indexes<'index>,
        context: &mut ConstContext<'index>,
    ) {
        match indexes.sources[&self.id] {
            ItemRef::Fn(node) => {
                node.transpile_name(shader);
                *shader += "(";
                for arg in &self.args {
                    arg.transpile(shader, indexes, context);
                    *shader += ", ";
                }
                *shader += ")";
            }
            ItemRef::Variable(_)
            | ItemRef::Constant(_)
            | ItemRef::Struct(_)
            | ItemRef::Param(_) => {
                unreachable!("function calls cannot reference values")
            }
        }
    }
}
