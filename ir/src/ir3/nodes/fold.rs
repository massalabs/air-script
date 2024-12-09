use crate::ir3::{BackLink, Child, Link, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Fold {
    parent: BackLink<Owner>,
    iterator: Link<Op>,
    operator: Link<FoldOperator>,
    initial_value: Link<Op>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum FoldOperator {
    Add,
    Mul,
    #[default]
    None,
}

impl Fold {
    pub fn new(iterator: Link<Op>, operator: Link<FoldOperator>, initial_value: Link<Op>) -> Self {
        Self {
            iterator,
            operator,
            initial_value,
            ..Default::default()
        }
    }
}

impl Parent for Fold {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.iterator.clone(), self.initial_value.clone()])
    }
}

impl Child for Fold {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}
