use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder)]
pub struct For {
    pub parents: Vec<BackLink<Owner>>,
    pub iterators: Link<Vec<Link<Op>>>,
    pub expr: Link<Op>,
    pub selector: Link<Op>,
}

impl For {
    pub fn create(
        iterators: Link<Vec<Link<Op>>>,
        expr: Link<Op>,
        selector: Link<Op>,
    ) -> Link<Self> {
        Self {
            iterators,
            expr,
            selector,
            ..Default::default()
        }
        .into()
    }
}

impl Link<For> {
    pub fn as_op(self) -> Link<Op> {
        Op::For(self).into()
    }
    pub fn as_owner(self) -> Link<Owner> {
        Owner::For(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::For(self).into()
    }
}

impl Parent for For {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        let mut children = Vec::from(self.iterators.borrow().clone());
        children.push(self.expr.clone());
        if *self.selector.borrow() != Op::None {
            children.push(self.selector.clone());
        }
        Link::new(children)
    }
}

impl Child for For {
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

    use crate::ir::{Add, Evaluator, SpannedMirValue, Sub, Value};

    use super::*;

    #[test]
    fn test_for_builder() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default().into()));
        let i_a = Value::builder()
            .value(SpannedMirValue::default())
            .build()
            .as_op();
        let i_b = Value::builder()
            .value(SpannedMirValue::default())
            .build()
            .as_op();
        let expr = Link::new(Op::Add(Add::default().into()));
        let selector = Link::new(Op::Sub(Sub::default().into()));
        let for_op = For::builder()
            .parents(parent.clone())
            .iterators(i_a.clone())
            .iterators(i_b.clone())
            .expr(expr.clone())
            .selector(selector.clone())
            .build();
        assert_eq!(
            for_op.borrow().deref(),
            &For {
                parents: vec![parent.clone().into()],
                iterators: Link::new(vec![i_a.clone(), i_b.clone()]),
                expr: expr.clone(),
                selector: selector.clone(),
            }
        );
    }
}
