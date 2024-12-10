use crate::ir3::{BackLink, Builder, Child, Link, NotSet, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Mul {
    parent: BackLink<Owner>,
    lhs: Link<Op>,
    rhs: Link<Op>,
}

impl Mul {
    pub fn new(lhs: Link<Op>, rhs: Link<Op>) -> Self {
        Self {
            lhs,
            rhs,
            ..Default::default()
        }
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

type MulBuilderStart = MulBuilder<(BackLink<Owner>, NotSet, NotSet)>;
type MulBuilderA = MulBuilder<(BackLink<Owner>, Link<Op>, NotSet)>;
type MulBuilderB = MulBuilder<(BackLink<Owner>, NotSet, Link<Op>)>;
type MulBuilderFinish = MulBuilder<(BackLink<Owner>, Link<Op>, Link<Op>)>;

impl Builder for Mul {
    type BuilderType = MulBuilderStart;
    fn builder() -> Self::BuilderType {
        MulBuilder::default()
    }
}

impl Default for MulBuilderStart {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parent: BackLink::default(),
            lhs: None,
            rhs: None,
        }
    }
}

impl MulBuilderStart {
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
    pub fn rhs(mut self, rhs: Link<Op>) -> MulBuilderFinish {
        self.rhs = Some(rhs);
        unsafe { std::mem::transmute(self) }
    }
}

impl MulBuilderB {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn lhs(mut self, lhs: Link<Op>) -> MulBuilderFinish {
        self.lhs = Some(lhs);
        unsafe { std::mem::transmute(self) }
    }
    pub fn rhs(mut self, rhs: Link<Op>) -> Self {
        self.rhs = Some(rhs);
        self
    }
}

impl MulBuilderFinish {
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
    use crate::ir3::{Evaluator, Owner};

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
