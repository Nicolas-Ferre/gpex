use std::fmt::Write;

#[derive(Debug, Clone)]
pub(crate) enum ConstValue {
    I32(i32),
}

impl ConstValue {
    pub(crate) fn transpile(&self, shader: &mut String) {
        match self {
            Self::I32(value) => {
                let _ = write!(shader, "i32({value})");
            }
        }
    }
}
