pub(crate) mod calls;
pub(crate) mod exprs;
pub(crate) mod idents;
pub(crate) mod imports;
pub(crate) mod items;
pub(crate) mod operators;
pub(crate) mod statements;

use crate::{LogInner, LogLevel};

pub(crate) fn replacement(replacement: &str) -> LogInner {
    LogInner {
        level: LogLevel::Hint,
        msg: format!("replace with `{replacement}`"),
        location: None,
    }
}
