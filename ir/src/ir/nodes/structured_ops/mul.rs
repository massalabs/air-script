use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder)]
pub struct Mul {
    pub parents: Vec<BackLink<Owner>>,
    pub lhs: Link<Op>,
    pub rhs: Link<Op>,
}

impl Mul {
    pub fn create(lhs: Link<Op>, rhs: Link<Op>) -> Link<Self> {
        Self {
            lhs,
            rhs,
            ..Default::default()
        }
        .into()
    }
}

impl Link<Mul> {
    pub fn as_op(self) -> Link<Op> {
        Op::Mul(self).into()
    }
    pub fn as_owner(self) -> Link<Owner> {
        Owner::Mul(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::Mul(self).into()
    }
}

impl Parent for Mul {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.lhs.clone(), self.rhs.clone()])
    }
}

impl Child for Mul {
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
    use crate::ir::{Evaluator, Owner};

    #[test]
    fn test_mul_builder() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default().into()));
        let lhs = Link::new(Op::default());
        let rhs = Link::new(Op::default());
        let mul = Mul::builder()
            .parents(parent.clone())
            .lhs(lhs.clone())
            .rhs(rhs.clone())
            .build();
        assert_eq!(
            mul.borrow().deref(),
            &Mul {
                parents: vec![parent.into()],
                lhs,
                rhs,
            }
        );
    }
}
