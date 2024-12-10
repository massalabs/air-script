use crate::ir3::{BackLink, Builder, Child, Link, NotSet, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Add {
    parent: BackLink<Owner>,
    lhs: Link<Op>,
    rhs: Link<Op>,
}

impl Add {
    pub fn new(lhs: Link<Op>, rhs: Link<Op>) -> Self {
        Self {
            lhs,
            rhs,
            ..Default::default()
        }
    }
}

impl Parent for Add {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.lhs.clone(), self.rhs.clone()])
    }
}

impl Child for Add {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}

pub struct AddBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parent: BackLink<Owner>,
    lhs: Option<Link<Op>>,
    rhs: Option<Link<Op>>,
}

type AddBuilderStart = AddBuilder<(BackLink<Owner>, NotSet, NotSet)>;
type AddBuilderA = AddBuilder<(BackLink<Owner>, Link<Op>, NotSet)>;
type AddBuilderB = AddBuilder<(BackLink<Owner>, NotSet, Link<Op>)>;
type AddBuilderFinish = AddBuilder<(BackLink<Owner>, Link<Op>, Link<Op>)>;

impl Builder for Add {
    type BuilderType = AddBuilderStart;
    fn builder() -> Self::BuilderType {
        AddBuilder::default()
    }
}

impl Default for AddBuilderStart {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parent: BackLink::default(),
            lhs: None,
            rhs: None,
        }
    }
}

impl AddBuilderStart {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn lhs(mut self, lhs: Link<Op>) -> AddBuilderA {
        self.lhs = Some(lhs);
        unsafe { std::mem::transmute(self) }
    }
    pub fn rhs(mut self, rhs: Link<Op>) -> AddBuilderB {
        self.rhs = Some(rhs);
        unsafe { std::mem::transmute(self) }
    }
}

impl AddBuilderA {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn lhs(mut self, lhs: Link<Op>) -> Self {
        self.lhs = Some(lhs);
        self
    }
    pub fn rhs(mut self, rhs: Link<Op>) -> AddBuilderFinish {
        self.rhs = Some(rhs);
        unsafe { std::mem::transmute(self) }
    }
}

impl AddBuilderB {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn lhs(mut self, lhs: Link<Op>) -> AddBuilderFinish {
        self.lhs = Some(lhs);
        unsafe { std::mem::transmute(self) }
    }
    pub fn rhs(mut self, rhs: Link<Op>) -> Self {
        self.rhs = Some(rhs);
        self
    }
}

impl AddBuilderFinish {
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
    pub fn build(self) -> Add {
        Add {
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
    fn test_add_builder() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default()));
        let lhs = Link::new(Op::default());
        let rhs = Link::new(Op::default());
        let add = Add::builder()
            .parent(parent.clone())
            .lhs(lhs.clone())
            .rhs(rhs.clone())
            .build();
        assert_eq!(
            add,
            Add {
                parent: parent.into(),
                lhs,
                rhs,
            }
        );
    }
}
