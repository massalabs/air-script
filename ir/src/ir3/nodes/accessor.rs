use std::{any::Any, hash::Hash};

use air_parser::ast::{AccessType, RangeBound, Type};

use crate::ir3::{BackLink, Builder, Child, Link, NotSet, Op, Owner, Parent};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Accessor {
    parent: BackLink<Owner>,
    indexable: Link<Op>,
    access_type: AccessType,
}

impl Default for Accessor {
    fn default() -> Self {
        Self {
            parent: BackLink::default(),
            indexable: Link::default(),
            access_type: AccessType::Default,
        }
    }
}

impl Hash for Accessor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.type_id().hash(state);
        match &self.access_type {
            AccessType::Default => 0.hash(state),
            AccessType::Slice(range_expr) => {
                1.hash(state);
                match &range_expr.start {
                    RangeBound::Const(constant) => constant.hash(state),
                    RangeBound::SymbolAccess(symbol_access) => {
                        symbol_access.name.hash(state);
                        match symbol_access.ty {
                            Some(Type::Felt) => {
                                0.hash(state);
                            }
                            Some(Type::Vector(x)) => {
                                1.hash(state);
                                x.hash(state);
                            }
                            Some(Type::Matrix(x, y)) => {
                                2.hash(state);
                                x.hash(state);
                                y.hash(state);
                            }
                            None => {}
                        }
                    }
                }
            }
            AccessType::Index(index) => {
                2.hash(state);
                index.hash(state);
            }
            AccessType::Matrix(x, y) => {
                3.hash(state);
                x.hash(state);
                y.hash(state);
            }
        }
        self.indexable.hash(state);
    }
}

impl Accessor {
    pub fn new(indexable: Link<Op>, access_type: AccessType) -> Self {
        Self {
            access_type,
            indexable,
            ..Default::default()
        }
    }
}

impl Parent for Accessor {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.indexable.clone()])
    }
}

impl Child for Accessor {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}

pub struct AccessorBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parent: BackLink<Owner>,
    indexable: Option<Link<Op>>,
    access_type: Option<AccessType>,
}

type AccessorBuilderEmpty = AccessorBuilder<(BackLink<Owner>, NotSet, NotSet)>;
type AccessorBuilderA = AccessorBuilder<(BackLink<Owner>, Link<Op>, NotSet)>;
type AccessorBuilderB = AccessorBuilder<(BackLink<Owner>, NotSet, AccessType)>;
type AccessorBuilderFull = AccessorBuilder<(BackLink<Owner>, Link<Op>, AccessType)>;

impl Builder for Accessor {
    type BuilderEmpty = AccessorBuilderEmpty;
    type BuilderFull = AccessorBuilderFull;
    fn builder() -> Self::BuilderEmpty {
        AccessorBuilder::default()
    }
    fn edit(self) -> Self::BuilderFull {
        Self::BuilderFull {
            _state: std::marker::PhantomData,
            parent: self.parent,
            indexable: Some(self.indexable),
            access_type: Some(self.access_type),
        }
    }
}

impl Default for AccessorBuilderEmpty {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parent: BackLink::default(),
            indexable: None,
            access_type: None,
        }
    }
}

impl AccessorBuilderEmpty {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn indexable(mut self, indexable: Link<Op>) -> AccessorBuilderA {
        self.indexable = Some(indexable);
        unsafe { std::mem::transmute(self) }
    }
    pub fn access_type(mut self, access_type: AccessType) -> AccessorBuilderB {
        self.access_type = Some(access_type);
        unsafe { std::mem::transmute(self) }
    }
}

impl AccessorBuilderA {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn indexable(mut self, indexable: Link<Op>) -> Self {
        self.indexable = Some(indexable);
        self
    }
    pub fn access_type(mut self, access_type: AccessType) -> AccessorBuilderFull {
        self.access_type = Some(access_type);
        unsafe { std::mem::transmute(self) }
    }
}

impl AccessorBuilderB {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn indexable(mut self, indexable: Link<Op>) -> AccessorBuilderFull {
        self.indexable = Some(indexable);
        unsafe { std::mem::transmute(self) }
    }
    pub fn access_type(mut self, access_type: AccessType) -> Self {
        self.access_type = Some(access_type);
        self
    }
}

impl AccessorBuilderFull {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn indexable(mut self, indexable: Link<Op>) -> Self {
        self.indexable = Some(indexable);
        self
    }
    pub fn access_type(mut self, access_type: AccessType) -> Self {
        self.access_type = Some(access_type);
        self
    }
    pub fn build(self) -> Accessor {
        Accessor {
            parent: self.parent,
            indexable: self.indexable.unwrap(),
            access_type: self.access_type.unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accessor_builder() {
        let parent = Link::new(Owner::default());
        let indexable = Link::new(Op::default());
        let access_type = AccessType::Default;

        let accessor = Accessor::builder()
            .parent(parent.clone())
            .indexable(indexable.clone())
            .access_type(access_type.clone())
            .build();

        assert_eq!(
            accessor,
            Accessor {
                parent: parent.into(),
                indexable,
                access_type
            }
        );
    }
}
