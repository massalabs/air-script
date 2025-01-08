use crate::ir::{BackLink, Builder, Child, Link, Node, NotSet, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Add {
    pub parents: Vec<BackLink<Owner>>,
    pub lhs: Link<Op>,
    pub rhs: Link<Op>,
}

impl Add {
    pub fn create(lhs: Link<Op>, rhs: Link<Op>) -> Link<Self> {
        Self {
            lhs,
            rhs,
            ..Default::default()
        }
        .into()
    }
}

impl Link<Add> {
    pub fn as_op(self) -> Link<Op> {
        Op::Add(self).into()
    }
    pub fn as_owner(self) -> Link<Owner> {
        Owner::Add(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::Add(self).into()
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

pub struct AddBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parents: Vec<BackLink<Owner>>,
    lhs: Option<Link<Op>>,
    rhs: Option<Link<Op>>,
}

type AddBuilderEmpty = AddBuilder<(BackLink<Owner>, NotSet, NotSet)>;
type AddBuilderA = AddBuilder<(BackLink<Owner>, Link<Op>, NotSet)>;
type AddBuilderB = AddBuilder<(BackLink<Owner>, NotSet, Link<Op>)>;
type AddBuilderFull = AddBuilder<(BackLink<Owner>, Link<Op>, Link<Op>)>;

impl Builder for Add {
    type Empty = AddBuilderEmpty;
    type Full = AddBuilderFull;
    fn builder() -> Self::Empty {
        AddBuilder::default()
    }
}

impl Default for AddBuilderEmpty {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parents: Vec::default(),
            lhs: None,
            rhs: None,
        }
    }
}

impl AddBuilderEmpty {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
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
        self.parents.push(parent.into());
        self
    }
    pub fn lhs(mut self, lhs: Link<Op>) -> Self {
        self.lhs = Some(lhs);
        self
    }
    pub fn rhs(mut self, rhs: Link<Op>) -> AddBuilderFull {
        self.rhs = Some(rhs);
        unsafe { std::mem::transmute(self) }
    }
}

impl AddBuilderB {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
        self
    }
    pub fn lhs(mut self, lhs: Link<Op>) -> AddBuilderFull {
        self.lhs = Some(lhs);
        unsafe { std::mem::transmute(self) }
    }
    pub fn rhs(mut self, rhs: Link<Op>) -> Self {
        self.rhs = Some(rhs);
        self
    }
}

impl AddBuilderFull {
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
    pub fn build(self) -> Link<Add> {
        Add {
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
    fn test_add_builder() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default().into()));
        let lhs = Link::new(Op::default());
        let rhs = Link::new(Op::default());
        let add = Add::builder()
            .parent(parent.clone())
            .lhs(lhs.clone())
            .rhs(rhs.clone())
            .build();
        assert_eq!(
            add.borrow().deref(),
            &Add {
                parents: vec![parent.into()],
                lhs,
                rhs,
            }
        );
    }
}
