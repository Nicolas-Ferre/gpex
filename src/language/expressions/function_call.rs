use crate::compiler::constants::Constant;
use crate::compiler::indexes::Indexes;
use crate::compiler::types::Type;
use crate::language::DependencyType;
use crate::language::expressions::Expression;
use crate::language::items::ItemRef;
use crate::language::patterns::IDENTIFIER_PATTERN;
use crate::language::symbols::{COMMA_SYMBOL, PARENTHESIS_CLOSE_SYMBOL, PARENTHESIS_OPEN_SYMBOL};
use crate::utils::dependencies::Dependencies;
use crate::utils::indexing::{NodeRef, SearchConfig, Visibility};
use crate::utils::parsing::{ParseContext, ParseError, Span, SpanProperties};
use crate::utils::validation::{ValidateContext, ValidateError};
use crate::validators;

const SEARCH_CONFIG: SearchConfig = SearchConfig {
    can_be_after: true,
    can_be_parent_node: true,
};

#[derive(Debug)]
pub(crate) struct FunctionCall {
    id: u64,
    scope: Vec<u64>,
    pub(crate) span: Span,
    pub(crate) name: String,
    args: Vec<Expression>,
}

impl NodeRef for &FunctionCall {
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

impl FunctionCall {
    pub(crate) fn parse<'context>(
        context: &mut ParseContext<'context>,
    ) -> Result<Self, ParseError<'context>> {
        let name_span = Span::parse_pattern(context, IDENTIFIER_PATTERN)?;
        Span::parse_symbol(context, PARENTHESIS_OPEN_SYMBOL)?;
        context.force_parse_any_error();
        let (args, _) = context
            .parse_many(
                0,
                Expression::parse,
                Some(|context| Span::parse_symbol(context, COMMA_SYMBOL).map(|_| ())),
            )
            .unwrap_or_else(|_| unreachable!("argument parsing should not fail as it is optional"));
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

    pub(crate) fn index(&self, indexes: &mut Indexes<'_>) {
        for arg in &self.args {
            arg.index(indexes);
        }
        let imports = &mut indexes.imports;
        if let Some(source) = indexes.items.search(
            &self.key(),
            self,
            imports,
            Visibility::Ignored,
            SEARCH_CONFIG,
        ) {
            indexes.private_sources.insert(self.id, source);
        }
        if let Some(source) = indexes.items.search(
            &self.key(),
            self,
            imports,
            Visibility::Enforced,
            SEARCH_CONFIG,
        ) {
            imports.mark_as_used(self.file_index(), source.file_index());
            indexes.sources.insert(self.id, source);
            indexes
                .item_first_refs
                .entry(source.id())
                .or_insert_with(|| self.span);
        }
    }

    pub(crate) fn dependencies<'index>(
        &self,
        type_: DependencyType,
        mut dependencies: Dependencies<ItemRef<'index>>,
        indexes: &Indexes<'index>,
    ) -> Result<Dependencies<ItemRef<'index>>, Vec<Span>> {
        for arg in &self.args {
            dependencies = arg.dependencies(type_, dependencies, indexes)?;
        }
        if let Some(&source) = indexes.sources.get(&self.id) {
            dependencies.register(self.span, source, |dependencies| {
                source.dependencies(type_, dependencies, indexes)
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

    pub(crate) fn constant<'index>(&self, indexes: &Indexes<'index>) -> Constant<'index> {
        match indexes.sources.get(&self.id) {
            Some(ItemRef::Function(node)) => node.constant(indexes),
            Some(ItemRef::Variable(_) | ItemRef::Constant(_) | ItemRef::Struct(_)) => {
                unreachable!("identifier should not refer to a value")
            }
            None => Constant::Unknown,
        }
    }

    pub(crate) fn validate(
        &self,
        constant_mark_span: Option<Span>,
        context: &mut ValidateContext<'_>,
        indexes: &Indexes<'_>,
    ) -> Result<(), ValidateError> {
        for arg in &self.args {
            arg.validate(constant_mark_span, context, indexes)?;
        }
        validators::item::check_found(self, self.span, &self.key(), context, indexes)?;
        if let Some(constant_mark_span) = constant_mark_span {
            validators::expression::check_constant(
                self.constant(indexes),
                self.span,
                constant_mark_span,
                context,
            )?;
        }
        Ok(())
    }

    pub(crate) fn transpile(&self, shader: &mut String, indexes: &Indexes<'_>) {
        match indexes.sources[&self.id] {
            ItemRef::Function(node) => node.transpile_ref(shader),
            ItemRef::Variable(_) | ItemRef::Constant(_) | ItemRef::Struct(_) => {
                unreachable!("function calls cannot reference values")
            }
        }
    }
}
