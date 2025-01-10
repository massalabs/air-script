use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder)]
pub struct Enf {
    pub parents: Vec<BackLink<Owner>>,
    pub expr: Link<Op>,
}

impl Enf {
    pub fn create(expr: Link<Op>) -> Link<Self> {
        Self {
            expr,
            ..Default::default()
        }
        .into()
    }
}

impl Link<Enf> {
    pub fn as_op(self) -> Link<Op> {
        Op::Enf(self).into()
    }
    pub fn as_owner(self) -> Link<Owner> {
        Owner::Enf(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::Enf(self).into()
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

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;
    use crate::ir::{Add, Evaluator};

    #[test]
    fn test_enf_builder() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default().into()));
        let expr = Link::new(Add::default()).as_op();
        let enf = Enf::builder()
            .parents(parent.clone())
            .expr(expr.clone())
            .build();
        assert_eq!(
            enf.borrow().deref(),
            &Enf {
                parents: vec![parent.into()],
                expr
            }
        );
    }
}
