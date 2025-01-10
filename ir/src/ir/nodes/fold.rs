use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder)]
pub struct Fold {
    pub parents: Vec<BackLink<Owner>>,
    pub iterator: Link<Op>,
    pub operator: FoldOperator,
    pub initial_value: Link<Op>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum FoldOperator {
    Add,
    Mul,
    #[default]
    None,
}

impl Fold {
    pub fn create(
        iterator: Link<Op>,
        operator: FoldOperator,
        initial_value: Link<Op>,
    ) -> Link<Self> {
        Self {
            iterator,
            operator,
            initial_value,
            ..Default::default()
        }
        .into()
    }
}

impl Link<Fold> {
    pub fn as_op(self) -> Link<Op> {
        Op::Fold(self).into()
    }
    pub fn as_owner(self) -> Link<Owner> {
        Owner::Fold(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::Fold(self).into()
    }
}

impl Parent for Fold {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.iterator.clone(), self.initial_value.clone()])
    }
}

impl Child for Fold {
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

    use crate::ir::{Add, Evaluator, Mul};

    use super::*;

    #[test]
    fn test_fold_builder() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default().into()));
        let fold = Fold::builder()
            .parents(parent.clone())
            .iterator(Link::new(Op::Add(Add::default().into())))
            .operator(FoldOperator::Add)
            .initial_value(Link::new(Op::Mul(Mul::default().into())))
            .build();
        assert_eq!(
            fold.borrow().deref(),
            &Fold {
                parents: vec![parent.clone().into()],
                iterator: Link::new(Op::Add(Add::default().into())),
                operator: FoldOperator::Add,
                initial_value: Link::new(Op::Mul(Mul::default().into())),
            }
        );
    }
}
