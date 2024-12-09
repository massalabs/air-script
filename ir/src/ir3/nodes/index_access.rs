use std::{any::Any, hash::Hash};

use crate::ir3::{BackLink, Child, Link, Op, Owner, Parent};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct IndexAccess {
    parent: BackLink<Owner>,
    indexable: Link<Op>,
    index: usize,
}

impl Default for IndexAccess {
    fn default() -> Self {
        Self {
            parent: BackLink::default(),
            indexable: Link::default(),
            index: 0,
        }
    }
}

impl Hash for IndexAccess {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.type_id().hash(state);
        self.index.hash(state);
        self.indexable.hash(state);
    }
}

impl IndexAccess {
    pub fn new(indexable: Link<Op>, index: usize) -> Self {
        Self {
            index,
            indexable,
            ..Default::default()
        }
    }
}

impl Parent for IndexAccess {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.indexable.clone()])
    }
}

impl Child for IndexAccess {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}
