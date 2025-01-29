use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner, Parent};
use air_parser::ast::{AccessType, RangeBound, Type};
use std::{any::Any, hash::Hash};

#[derive(Clone, PartialEq, Eq, Debug, Builder)]
#[enum_wrapper(Op)]
pub struct Accessor {
    pub parents: Vec<BackLink<Owner>>,
    pub indexable: Link<Op>,
    pub access_type: AccessType,
    pub _node: Option<Link<Node>>,
    pub _owner: Option<Link<Owner>>,
}

impl Default for Accessor {
    fn default() -> Self {
        Self {
            parents: Vec::default(),
            indexable: Link::default(),
            access_type: AccessType::Default,
            _node: None,
            _owner: None,
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
    pub fn create(indexable: Link<Op>, access_type: AccessType) -> Link<Op> {
        Op::Accessor(Self {
            access_type,
            indexable,
            ..Default::default()
        })
        .into()
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
    fn get_parents(&self) -> Vec<BackLink<Self::Parent>> {
        self.parents.clone()
    }
    fn add_parent(&mut self, parent: Link<Self::Parent>) {
        self.parents.push(parent.into());
    }
    fn remove_parent(&mut self, parent: Link<Self::Parent>) {
        self.parents.retain(|p| *p != parent.clone().into());
    }
}
