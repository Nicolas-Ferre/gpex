use crate::language::items::struct_::StructDefinition;
use crate::utils::formatting;
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Debug, Default)]
pub(crate) struct ConstContext<'item> {
    scope_values: Vec<HashMap<u64, ConstValue<'item>>>,
}

impl<'item> ConstContext<'item> {
    pub(crate) fn run_scoped<O>(&mut self, mut callback: impl FnMut(&mut Self) -> O) -> O {
        self.scope_values.push(HashMap::new());
        let output = callback(self);
        self.scope_values.pop();
        output
    }

    pub(crate) fn value(&self, id: u64) -> ConstValue<'item> {
        self.scope_values
            .last()
            .and_then(|values| values.get(&id))
            .cloned()
            .unwrap_or(ConstValue::RuntimeValue)
    }

    pub(crate) fn add_value(&mut self, id: u64, value: ConstValue<'item>) {
        let current_scope_index = self.scope_values.len() - 1;
        self.scope_values[current_scope_index].insert(id, value);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ConstValue<'item> {
    TypeRef(&'item StructDefinition),
    I32(i32),
    U32(u32),
    F32(f32),
    Bool(bool),
    Unknown,
    RuntimeValue,
}

impl<'item> ConstValue<'item> {
    pub(crate) fn transpile(&self, shader: &mut String) {
        match self {
            Self::TypeRef(value) => value.transpile_ref(shader),
            Self::I32(value) => _ = write!(shader, "i32({value})"),
            Self::U32(value) => _ = write!(shader, "u32({value})"),
            Self::F32(value) => _ = write!(shader, "f32({})", formatting::f32_to_string(*value)),
            Self::Bool(value) => _ = write!(shader, "u32({})", u32::from(*value)),
            Self::Unknown | Self::RuntimeValue => unreachable!("non-constant cannot be transpiled"),
        }
    }

    pub(crate) fn type_ref(&self) -> Option<&'item StructDefinition> {
        if let Self::TypeRef(type_) = self {
            Some(type_)
        } else {
            None
        }
    }
}
