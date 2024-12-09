use std::{any::Any, hash::Hash};

use air_parser::ast::{AccessType, RangeBound, Type};

use crate::ir3::{BackLink, Child, Link, Op, Owner, Parent};

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
