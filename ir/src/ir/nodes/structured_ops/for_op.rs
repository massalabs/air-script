use crate::ir::{BackLink, Builder, Child, Link, Node, NotSet, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
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
        children.push(self.selector.clone());
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

pub struct ForBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parents: Vec<BackLink<Owner>>,
    iterators: Vec<Link<Op>>,
    expr: Option<Link<Op>>,
    selector: Option<Link<Op>>,
}

type ForBuilderEmpty = ForBuilder<(BackLink<Owner>, Vec<Link<Op>>, NotSet, NotSet)>;
type ForBuilderA = ForBuilder<(BackLink<Owner>, Vec<Link<Op>>, Link<Op>, NotSet)>;
type ForBuilderB = ForBuilder<(BackLink<Owner>, Vec<Link<Op>>, NotSet, Link<Op>)>;
type ForBuilderFull = ForBuilder<(BackLink<Owner>, Vec<Link<Op>>, Link<Op>, Link<Op>)>;

impl Builder for For {
    type BuilderEmpty = ForBuilderEmpty;
    type BuilderFull = ForBuilderFull;
    fn builder() -> Self::BuilderEmpty {
        ForBuilder::default()
    }
}

impl Default for ForBuilderEmpty {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parents: Vec::default(),
            iterators: Vec::new(),
            expr: None,
            selector: None,
        }
    }
}

impl ForBuilderEmpty {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
        self
    }
    pub fn iterators(mut self, iterator: Link<Op>) -> Self {
        self.iterators.push(iterator);
        self
    }
    pub fn expr(mut self, expr: Link<Op>) -> ForBuilderA {
        self.expr = Some(expr);
        unsafe { std::mem::transmute(self) }
    }
    pub fn selector(mut self, selector: Link<Op>) -> ForBuilderB {
        self.selector = Some(selector);
        unsafe { std::mem::transmute(self) }
    }
}

impl ForBuilderA {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
        self
    }
    pub fn iterators(mut self, iterator: Link<Op>) -> Self {
        self.iterators.push(iterator);
        self
    }
    pub fn expr(mut self, expr: Link<Op>) -> Self {
        self.expr = Some(expr);
        self
    }
    pub fn selector(mut self, selector: Link<Op>) -> ForBuilderFull {
        self.selector = Some(selector);
        unsafe { std::mem::transmute(self) }
    }
}

impl ForBuilderB {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
        self
    }
    pub fn iterators(mut self, iterator: Link<Op>) -> Self {
        self.iterators.push(iterator);
        self
    }
    pub fn expr(mut self, expr: Link<Op>) -> ForBuilderFull {
        self.expr = Some(expr);
        unsafe { std::mem::transmute(self) }
    }
    pub fn selector(mut self, selector: Link<Op>) -> Self {
        self.selector = Some(selector);
        self
    }
}

impl ForBuilderFull {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
        self
    }
    pub fn iterators(mut self, iterator: Link<Op>) -> Self {
        self.iterators.push(iterator);
        self
    }
    pub fn expr(mut self, expr: Link<Op>) -> Self {
        self.expr = Some(expr);
        self
    }
    pub fn selector(mut self, selector: Link<Op>) -> Self {
        self.selector = Some(selector);
        self
    }
    pub fn build(self) -> Link<For> {
        For {
            parents: self.parents,
            iterators: Link::new(self.iterators),
            expr: self.expr.expect("expr"),
            selector: self.selector.expect("selector"),
        }
        .into()
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
            .parent(parent.clone())
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
