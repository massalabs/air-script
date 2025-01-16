use crate::ir::{BackLink, Builder, Child, Link, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder)]
#[enum_wrapper(Op)]
pub struct For {
    pub parents: Vec<BackLink<Owner>>,
    pub iterators: Link<Vec<Link<Op>>>,
    pub expr: Link<Op>,
    pub selector: Link<Op>,
}

impl For {
    pub fn create(iterators: Link<Vec<Link<Op>>>, expr: Link<Op>, selector: Link<Op>) -> Link<Op> {
        Op::For(Self {
            iterators,
            expr,
            selector,
            ..Default::default()
        })
        .into()
    }
}

impl Parent for For {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        let mut children = Vec::from(self.iterators.borrow().clone());
        children.push(self.expr.clone());
        children.push(self.selector.clone());
        Link::new(children)
    }
}

impl Child for For {
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
