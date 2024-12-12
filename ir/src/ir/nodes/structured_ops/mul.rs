use crate::ir::{BackLink, Builder, Child, Link, Node, NotSet, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Mul {
    pub parent: BackLink<Owner>,
    pub lhs: Link<Op>,
    pub rhs: Link<Op>,
}

impl Mul {
    pub fn new(lhs: Link<Op>, rhs: Link<Op>) -> Self {
        Self {
            lhs,
            rhs,
            ..Default::default()
        }
    }
    pub fn as_op(self) -> Op {
        Op::Mul(self)
    }
    pub fn as_owner(self) -> Owner {
        Owner::Mul(self)
    }
    pub fn as_node(self) -> Node {
        Node::Mul(self)
    }
}

impl Link<Mul> {
    pub fn as_op(self) -> Link<Op> {
        Link::new(Op::Mul(self.borrow().clone()))
    }
    pub fn as_owner(self) -> Link<Owner> {
        Link::new(Owner::Mul(self.borrow().clone()))
    }
    pub fn as_node(self) -> Link<Node> {
        Link::new(Node::Mul(self.borrow().clone()))
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
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}

pub struct MulBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parent: BackLink<Owner>,
    lhs: Option<Link<Op>>,
    rhs: Option<Link<Op>>,
}

type MulBuilderEmpty = MulBuilder<(BackLink<Owner>, NotSet, NotSet)>;
type MulBuilderA = MulBuilder<(BackLink<Owner>, Link<Op>, NotSet)>;
type MulBuilderB = MulBuilder<(BackLink<Owner>, NotSet, Link<Op>)>;
type MulBuilderFull = MulBuilder<(BackLink<Owner>, Link<Op>, Link<Op>)>;

impl Builder for Mul {
    type BuilderEmpty = MulBuilderEmpty;
    type BuilderFull = MulBuilderFull;
    fn builder() -> Self::BuilderEmpty {
        MulBuilder::default()
    }
    fn edit(self) -> Self::BuilderFull {
        Self::BuilderFull {
            _state: std::marker::PhantomData,
            parent: self.parent,
            lhs: Some(self.lhs),
            rhs: Some(self.rhs),
        }
    }
}

impl Default for MulBuilderEmpty {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parent: BackLink::default(),
            lhs: None,
            rhs: None,
        }
    }
}

impl MulBuilderEmpty {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
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
        self.parent = parent.into();
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
        self.parent = parent.into();
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
        self.parent = parent.into();
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
    pub fn build(self) -> Mul {
        Mul {
            parent: self.parent,
            lhs: self.lhs.unwrap(),
            rhs: self.rhs.unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Evaluator, Owner};

    #[test]
    fn test_mul_builder() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default()));
        let lhs = Link::new(Op::default());
        let rhs = Link::new(Op::default());
        let mul = Mul::builder()
            .parent(parent.clone())
            .lhs(lhs.clone())
            .rhs(rhs.clone())
            .build();
        assert_eq!(
            mul,
            Mul {
                parent: parent.into(),
                lhs,
                rhs,
            }
        );
    }
}
