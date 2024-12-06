use miden_diagnostics::SourceSpan;

use crate::{
    ir3::{BackLink, Child, Link, Op},
    MirValue, SpannedMirValue,
};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Value {
    parent: BackLink<Op>,
    value: SpannedMirValue,
}

impl Default for SpannedMirValue {
    fn default() -> Self {
        Self {
            value: MirValue::Constant(crate::ConstantValue::Felt(0)),
            span: Default::default(),
        }
    }
}

impl Value {
    pub fn new(value: SpannedMirValue) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}

impl Child for Value {
    type Parent = Op;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}
