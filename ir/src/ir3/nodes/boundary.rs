use std::hash::Hash;

use crate::ir3::{BackLink, Child, Link, Op, Owner, Parent};

use air_parser::ast::Boundary as BoundaryKind;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Boundary {
    parent: BackLink<Owner>,
    kind: BoundaryKind,
    expr: Link<Op>,
}

impl Default for Boundary {
    fn default() -> Self {
        Self {
            parent: BackLink::default(),
            kind: BoundaryKind::First,
            expr: Link::default(),
        }
    }
}

impl Hash for Boundary {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match &self.kind {
            BoundaryKind::First => 0.hash(state),
            BoundaryKind::Last => 1.hash(state),
        }
        self.expr.hash(state);
    }
}

impl Boundary {
    pub fn new(expr: Link<Op>) -> Self {
        Self {
            expr,
            ..Default::default()
        }
    }
}

impl Parent for Boundary {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.expr.clone()])
    }
}

impl Child for Boundary {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}
