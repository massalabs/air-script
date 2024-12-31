use crate::ir::{BackLink, Builder, Child, Link, Node, NotSet, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct If {
    pub parent: BackLink<Owner>,
    pub condition: Link<Op>,
    pub then_branch: Link<Op>,
    pub else_branch: Link<Op>,
}

impl If {
    pub fn create(condition: Link<Op>, then_branch: Link<Op>, else_branch: Link<Op>) -> Link<Self> {
        Self {
            condition,
            then_branch,
            else_branch,
            ..Default::default()
        }
        .into()
    }
}

impl Link<If> {
    pub fn as_op(self) -> Link<Op> {
        Op::If(self).into()
    }
    pub fn as_owner(self) -> Link<Owner> {
        Owner::If(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::If(self).into()
    }
}

impl Parent for If {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![
            self.condition.clone(),
            self.then_branch.clone(),
            self.else_branch.clone(),
        ])
    }
}

impl Child for If {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}

pub struct IfBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parent: BackLink<Owner>,
    condition: Option<Link<Op>>,
    then_branch: Option<Link<Op>>,
    else_branch: Option<Link<Op>>,
}

type IfBuilderEmpty = IfBuilder<(BackLink<Owner>, NotSet, NotSet, NotSet)>;
type IfBuilderA = IfBuilder<(BackLink<Owner>, Link<Op>, NotSet, NotSet)>;
type IfBuilderB = IfBuilder<(BackLink<Owner>, NotSet, Link<Op>, NotSet)>;
type IfBuilderC = IfBuilder<(BackLink<Owner>, NotSet, NotSet, Link<Op>)>;
type IfBuilderAB = IfBuilder<(BackLink<Owner>, Link<Op>, Link<Op>, NotSet)>;
type IfBuilderAC = IfBuilder<(BackLink<Owner>, Link<Op>, NotSet, Link<Op>)>;
type IfBuilderBC = IfBuilder<(BackLink<Owner>, NotSet, Link<Op>, Link<Op>)>;
type IfBuilderFull = IfBuilder<(BackLink<Owner>, Link<Op>, Link<Op>, Link<Op>)>;

impl Builder for If {
    type BuilderEmpty = IfBuilderEmpty;
    type BuilderFull = IfBuilderFull;
    fn builder() -> Self::BuilderEmpty {
        IfBuilder::default()
    }
    fn edit(self) -> Self::BuilderFull {
        Self::BuilderFull {
            _state: std::marker::PhantomData,
            parent: self.parent,
            condition: Some(self.condition),
            then_branch: Some(self.then_branch),
            else_branch: Some(self.else_branch),
        }
    }
}

impl Default for IfBuilderEmpty {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parent: BackLink::default(),
            condition: None,
            then_branch: None,
            else_branch: None,
        }
    }
}

impl IfBuilderEmpty {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn condition(mut self, condition: Link<Op>) -> IfBuilderA {
        self.condition = Some(condition);
        unsafe { std::mem::transmute(self) }
    }
    pub fn then_branch(mut self, then_branch: Link<Op>) -> IfBuilderB {
        self.then_branch = Some(then_branch);
        unsafe { std::mem::transmute(self) }
    }
    pub fn else_branch(mut self, else_branch: Link<Op>) -> IfBuilderC {
        self.else_branch = Some(else_branch);
        unsafe { std::mem::transmute(self) }
    }
}

impl IfBuilderA {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn condition(mut self, condition: Link<Op>) -> Self {
        self.condition = Some(condition);
        self
    }
    pub fn then_branch(mut self, then_branch: Link<Op>) -> IfBuilderAB {
        self.then_branch = Some(then_branch);
        unsafe { std::mem::transmute(self) }
    }
    pub fn else_branch(mut self, else_branch: Link<Op>) -> IfBuilderAC {
        self.else_branch = Some(else_branch);
        unsafe { std::mem::transmute(self) }
    }
}

impl IfBuilderB {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn condition(mut self, condition: Link<Op>) -> IfBuilderAB {
        self.condition = Some(condition);
        unsafe { std::mem::transmute(self) }
    }
    pub fn then_branch(mut self, then_branch: Link<Op>) -> Self {
        self.then_branch = Some(then_branch);
        self
    }
    pub fn else_branch(mut self, else_branch: Link<Op>) -> IfBuilderBC {
        self.else_branch = Some(else_branch);
        unsafe { std::mem::transmute(self) }
    }
}

impl IfBuilderC {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn condition(mut self, condition: Link<Op>) -> IfBuilderAC {
        self.condition = Some(condition);
        unsafe { std::mem::transmute(self) }
    }
    pub fn then_branch(mut self, then_branch: Link<Op>) -> Self {
        self.then_branch = Some(then_branch);
        self
    }
    pub fn else_branch(mut self, else_branch: Link<Op>) -> IfBuilderBC {
        self.else_branch = Some(else_branch);
        unsafe { std::mem::transmute(self) }
    }
}

impl IfBuilderAB {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn condition(mut self, condition: Link<Op>) -> Self {
        self.condition = Some(condition);
        self
    }
    pub fn then_branch(mut self, then_branch: Link<Op>) -> Self {
        self.then_branch = Some(then_branch);
        self
    }
    pub fn else_branch(mut self, else_branch: Link<Op>) -> IfBuilderFull {
        self.else_branch = Some(else_branch);
        unsafe { std::mem::transmute(self) }
    }
}

impl IfBuilderAC {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn condition(mut self, condition: Link<Op>) -> Self {
        self.condition = Some(condition);
        self
    }
    pub fn then_branch(mut self, then_branch: Link<Op>) -> IfBuilderFull {
        self.then_branch = Some(then_branch);
        unsafe { std::mem::transmute(self) }
    }
    pub fn else_branch(mut self, else_branch: Link<Op>) -> Self {
        self.else_branch = Some(else_branch);
        self
    }
}

impl IfBuilderBC {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn condition(mut self, condition: Link<Op>) -> IfBuilderFull {
        self.condition = Some(condition);
        unsafe { std::mem::transmute(self) }
    }
    pub fn then_branch(mut self, then_branch: Link<Op>) -> Self {
        self.then_branch = Some(then_branch);
        self
    }
    pub fn else_branch(mut self, else_branch: Link<Op>) -> Self {
        self.else_branch = Some(else_branch);
        self
    }
}

impl IfBuilderFull {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn condition(mut self, condition: Link<Op>) -> Self {
        self.condition = Some(condition);
        self
    }
    pub fn then_branch(mut self, then_branch: Link<Op>) -> Self {
        self.then_branch = Some(then_branch);
        self
    }
    pub fn else_branch(mut self, else_branch: Link<Op>) -> Self {
        self.else_branch = Some(else_branch);
        self
    }
    pub fn build(self) -> Link<If> {
        If {
            parent: self.parent,
            condition: self.condition.unwrap(),
            then_branch: self.then_branch.unwrap(),
            else_branch: self.else_branch.unwrap(),
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;
    use crate::ir::{Add, Evaluator, Mul, Owner, Sub};

    #[test]
    fn test_if_builder() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default().into()));
        let condition = Link::new(Op::Sub(Sub::default().into()));
        let then_branch = Link::new(Op::Add(Add::default().into()));
        let else_branch = Link::new(Op::Mul(Mul::default().into()));
        let if_op = If::builder()
            .parent(parent.clone())
            .condition(condition.clone())
            .then_branch(then_branch.clone())
            .else_branch(else_branch.clone())
            .build();
        assert_eq!(
            if_op.borrow().deref(),
            &If {
                parent: parent.into(),
                condition,
                then_branch,
                else_branch,
            }
        );
    }
}
