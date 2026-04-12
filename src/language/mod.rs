pub(crate) mod exprs;
pub(crate) mod items;
pub(crate) mod module;
pub(crate) mod patterns;
pub(crate) mod statements;
pub(crate) mod symbols;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DependencyType {
    CycleDetection,
    Transpilation,
}
