use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder)]
pub struct If {
    pub parents: Vec<BackLink<Owner>>,
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
    use crate::ir::{Add, Evaluator, Mul, Owner, Sub};

    #[test]
    fn test_if_builder() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default().into()));
        let condition = Link::new(Op::Sub(Sub::default().into()));
        let then_branch = Link::new(Op::Add(Add::default().into()));
        let else_branch = Link::new(Op::Mul(Mul::default().into()));
        let if_op = If::builder()
            .parents(parent.clone())
            .condition(condition.clone())
            .then_branch(then_branch.clone())
            .else_branch(else_branch.clone())
            .build();
        assert_eq!(
            if_op.borrow().deref(),
            &If {
                parents: vec![parent.into()],
                condition,
                then_branch,
                else_branch,
            }
        );
    }
}
