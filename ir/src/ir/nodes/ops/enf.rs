use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder)]
#[enum_wrapper(Op)]
pub struct Enf {
    pub parents: Vec<BackLink<Owner>>,
    pub expr: Link<Op>,
    pub _node: Option<Link<Node>>,
    pub _owner: Option<Link<Owner>>,
}

impl Enf {
    pub fn create(expr: Link<Op>) -> Link<Op> {
        Op::Enf(Self {
            expr,
            ..Default::default()
        })
        .into()
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
