use crate::ir::{BackLink, Builder, Child, Link, Node, NotSet, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
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

pub struct MulBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parents: Vec<BackLink<Owner>>,
    lhs: Option<Link<Op>>,
    rhs: Option<Link<Op>>,
}

type MulBuilderEmpty = MulBuilder<(BackLink<Owner>, NotSet, NotSet)>;
type MulBuilderA = MulBuilder<(BackLink<Owner>, Link<Op>, NotSet)>;
type MulBuilderB = MulBuilder<(BackLink<Owner>, NotSet, Link<Op>)>;
type MulBuilderFull = MulBuilder<(BackLink<Owner>, Link<Op>, Link<Op>)>;

impl Builder for Mul {
    type Empty = MulBuilderEmpty;
    type Full = MulBuilderFull;
    fn builder() -> Self::Empty {
        MulBuilder::default()
    }
}

impl Default for MulBuilderEmpty {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parents: Vec::default(),
            lhs: None,
            rhs: None,
        }
    }
}

impl MulBuilderEmpty {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
        self
    }
    pub fn lhs(mut self, lhs: Link<Op>) -> MulBuilderA {
        self.lhs = Some(lhs);
        unsafe { std::mem::transmute(self) }
    }
    pub fn rhs(mut self, rhs: Link<Op>) -> MulBuilderB {
        self.rhs = Some(rhs);
        unsafe { std::mem::transmute(self) }
    }
}

impl MulBuilderA {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
        self
    }
    pub fn lhs(mut self, lhs: Link<Op>) -> Self {
        self.lhs = Some(lhs);
        self
    }
    pub fn rhs(mut self, rhs: Link<Op>) -> MulBuilderFull {
        self.rhs = Some(rhs);
        unsafe { std::mem::transmute(self) }
    }
}

impl MulBuilderB {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
        self
    }
    pub fn lhs(mut self, lhs: Link<Op>) -> MulBuilderFull {
        self.lhs = Some(lhs);
        unsafe { std::mem::transmute(self) }
    }
    pub fn rhs(mut self, rhs: Link<Op>) -> Self {
        self.rhs = Some(rhs);
        self
    }
}

impl MulBuilderFull {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
        self
    }
    pub fn lhs(mut self, lhs: Link<Op>) -> Self {
        self.lhs = Some(lhs);
        self
    }
    pub fn rhs(mut self, rhs: Link<Op>) -> Self {
        self.rhs = Some(rhs);
        self
    }
    pub fn build(self) -> Link<Mul> {
        Mul {
            parents: self.parents,
            lhs: self.lhs.unwrap(),
            rhs: self.rhs.unwrap(),
        }
        .into()
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
            .parent(parent.clone())
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
