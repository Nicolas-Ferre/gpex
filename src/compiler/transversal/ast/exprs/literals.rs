use crate::utils::parsing::span::Span;

#[derive(Debug)]
pub(crate) struct BoolLiteral {
    pub(crate) span: Span,
    pub(crate) value: bool,
}

#[derive(Debug)]
pub(crate) struct F32Literal {
    pub(crate) span: Span,
    pub(crate) value: Option<f32>,
}

#[derive(Debug)]
pub(crate) struct I32Literal {
    pub(crate) span: Span,
    pub(crate) value: Option<i32>,
}

#[derive(Debug)]
pub(crate) struct U32Literal {
    pub(crate) span: Span,
    pub(crate) value: Option<u32>,
}
