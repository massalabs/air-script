use crate::ir3::{BackLink, Child, Link, Op, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Sub {
    parent: BackLink<Op>,
    lhs: Link<Op>,
    rhs: Link<Op>,
}

impl Sub {
    pub fn new(lhs: Link<Op>, rhs: Link<Op>) -> Self {
        Self {
            lhs,
            rhs,
            ..Default::default()
        }
    }
}

impl Parent for Sub {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.lhs.clone(), self.rhs.clone()])
    }
}

impl Child for Sub {
    type Parent = Op;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}
