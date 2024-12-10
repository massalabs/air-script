use std::marker::PhantomData;

use crate::{
    ir3::{BackLink, Builder, Child, Link, NotSet, Op, Owner},
    MirValue, SpannedMirValue,
};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Value {
    parent: BackLink<Owner>,
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
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}

pub struct ValueBuilder<State> {
    _state: PhantomData<State>,
    parent: BackLink<Owner>,
    value: Option<SpannedMirValue>,
}

impl Builder for Value {
    type BuilderEmpty = ValueBuilder<(BackLink<Owner>, NotSet)>;
    type BuilderFull = ValueBuilder<(BackLink<Owner>, NotSet)>;
    fn builder() -> Self::BuilderEmpty {
        ValueBuilder::default()
    }
    fn edit(self) -> Self::BuilderFull {
        Self::BuilderFull {
            _state: PhantomData,
            parent: self.parent,
            value: Some(self.value),
        }
    }
}

impl Default for ValueBuilder<(BackLink<Owner>, NotSet)> {
    fn default() -> Self {
        Self {
            _state: PhantomData,
            parent: BackLink::default(),
            value: None,
        }
    }
}

impl ValueBuilder<(BackLink<Owner>, NotSet)> {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn value(
        mut self,
        value: SpannedMirValue,
    ) -> ValueBuilder<(BackLink<Owner>, SpannedMirValue)> {
        self.value = Some(value);
        unsafe { std::mem::transmute(self) }
    }
}

impl ValueBuilder<(BackLink<Owner>, SpannedMirValue)> {
    pub fn value(mut self, value: SpannedMirValue) -> Self {
        self.value = Some(value);
        self
    }
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn build(self) -> Value {
        Value {
            parent: self.parent,
            value: self.value.expect("value not set"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir3::{Add, Owner};

    #[test]
    fn test_value_builder() {
        let parent = Link::new(Owner::Add(Add::default()));
        let value = Value::builder()
            .parent(parent.clone())
            .value(SpannedMirValue::default())
            .build();
        assert_eq!(Link::from(value.parent), parent.clone());
        assert_eq!(value.value, SpannedMirValue::default());
    }
}
