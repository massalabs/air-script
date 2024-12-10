use crate::ir3::{BackLink, Builder, Child, Link, NotSet, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Sub {
    parent: BackLink<Owner>,
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
    pub fn as_op(self) -> Op {
        Op::Sub(self)
    }
    pub fn as_owner(self) -> Owner {
        Owner::Sub(self)
    }
}

impl Link<Sub> {
    pub fn as_op(self) -> Link<Op> {
        Link::new(Op::Sub(self.borrow().clone()))
    }
    pub fn as_owner(self) -> Link<Owner> {
        Link::new(Owner::Sub(self.borrow().clone()))
    }
}

impl Parent for Sub {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.lhs.clone(), self.rhs.clone()])
    }
}

impl Child for Sub {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}

pub struct SubBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parent: BackLink<Owner>,
    lhs: Option<Link<Op>>,
    rhs: Option<Link<Op>>,
}

type SubBuilderEmpty = SubBuilder<(BackLink<Owner>, NotSet, NotSet)>;
type SubBuilderA = SubBuilder<(BackLink<Owner>, Link<Op>, NotSet)>;
type SubBuilderB = SubBuilder<(BackLink<Owner>, NotSet, Link<Op>)>;
type SubBuilderFull = SubBuilder<(BackLink<Owner>, Link<Op>, Link<Op>)>;

impl Builder for Sub {
    type BuilderEmpty = SubBuilderEmpty;
    type BuilderFull = SubBuilderFull;
    fn builder() -> Self::BuilderEmpty {
        SubBuilder::default()
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

impl Default for SubBuilderEmpty {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parent: BackLink::default(),
            lhs: None,
            rhs: None,
        }
    }
}

impl SubBuilderEmpty {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn lhs(mut self, lhs: Link<Op>) -> SubBuilderA {
        self.lhs = Some(lhs);
        unsafe { std::mem::transmute(self) }
    }
    pub fn rhs(mut self, rhs: Link<Op>) -> SubBuilderB {
        self.rhs = Some(rhs);
        unsafe { std::mem::transmute(self) }
    }
}

impl SubBuilderA {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn lhs(mut self, lhs: Link<Op>) -> Self {
        self.lhs = Some(lhs);
        self
    }
    pub fn rhs(mut self, rhs: Link<Op>) -> SubBuilderFull {
        self.rhs = Some(rhs);
        unsafe { std::mem::transmute(self) }
    }
}

impl SubBuilderB {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn lhs(mut self, lhs: Link<Op>) -> SubBuilderFull {
        self.lhs = Some(lhs);
        unsafe { std::mem::transmute(self) }
    }
    pub fn rhs(mut self, rhs: Link<Op>) -> Self {
        self.rhs = Some(rhs);
        self
    }
}

impl SubBuilderFull {
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
    pub fn build(self) -> Sub {
        Sub {
            parent: self.parent,
            lhs: self.lhs.unwrap(),
            rhs: self.rhs.unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir3::{Evaluator, Owner};

    #[test]
    fn test_sub_builder() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default()));
        let lhs = Link::new(Op::default());
        let rhs = Link::new(Op::default());
        let sub = Sub::builder()
            .parent(parent.clone())
            .lhs(lhs.clone())
            .rhs(rhs.clone())
            .build();
        assert_eq!(
            sub,
            Sub {
                parent: parent.into(),
                lhs,
                rhs,
            }
        );
    }
}
