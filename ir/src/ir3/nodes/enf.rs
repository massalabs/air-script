use crate::ir3::{BackLink, Child, Link, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Enf {
    parent: BackLink<Owner>,
    expr: Link<Op>,
}

impl Enf {
    pub fn new(expr: Link<Op>) -> Self {
        Self {
            expr,
            ..Default::default()
        }
    }
}

impl Parent for Enf {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.expr.clone()])
    }
}

impl Child for Enf {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}
