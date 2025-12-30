use std::fmt::Write;

#[derive(Debug, Clone)]
pub(crate) enum Constant {
    I32(i32),
}

impl Constant {
    pub(crate) fn transpile(&self, shader: &mut String) {
        match self {
            Self::I32(value) => {
                _ = write!(shader, "i32({value})");
            }
        }
    }
}
