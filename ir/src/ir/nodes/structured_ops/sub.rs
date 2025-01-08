use crate::ir::{BackLink, Builder, Child, Link, Node, NotSet, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Sub {
    pub parents: Vec<BackLink<Owner>>,
    pub lhs: Link<Op>,
    pub rhs: Link<Op>,
}

impl Sub {
    pub fn create(lhs: Link<Op>, rhs: Link<Op>) -> Link<Self> {
        Self {
            lhs,
            rhs,
            ..Default::default()
        }
        .into()
    }
}

impl Link<Sub> {
    pub fn as_op(self) -> Link<Op> {
        Op::Sub(self).into()
    }
    pub fn as_owner(self) -> Link<Owner> {
        Owner::Sub(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::Sub(self).into()
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

pub struct SubBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parents: Vec<BackLink<Owner>>,
    lhs: Option<Link<Op>>,
    rhs: Option<Link<Op>>,
}

type SubBuilderEmpty = SubBuilder<(BackLink<Owner>, NotSet, NotSet)>;
type SubBuilderA = SubBuilder<(BackLink<Owner>, Link<Op>, NotSet)>;
type SubBuilderB = SubBuilder<(BackLink<Owner>, NotSet, Link<Op>)>;
type SubBuilderFull = SubBuilder<(BackLink<Owner>, Link<Op>, Link<Op>)>;

impl Builder for Sub {
    type Empty = SubBuilderEmpty;
    type Full = SubBuilderFull;
    fn builder() -> Self::Empty {
        SubBuilder::default()
    }
}

impl Default for SubBuilderEmpty {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parents: Vec::default(),
            lhs: None,
            rhs: None,
        }
    }
}

impl SubBuilderEmpty {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
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
        self.parents.push(parent.into());
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
        self.parents.push(parent.into());
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
    pub fn build(self) -> Link<Sub> {
        Sub {
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
    fn test_sub_builder() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default().into()));
        let lhs = Link::new(Op::default());
        let rhs = Link::new(Op::default());
        let sub = Sub::builder()
            .parent(parent.clone())
            .lhs(lhs.clone())
            .rhs(rhs.clone())
            .build();
        assert_eq!(
            sub.borrow().deref(),
            &Sub {
                parents: vec![parent.into()],
                lhs,
                rhs,
            }
        );
    }
}
