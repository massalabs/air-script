use crate::ir::{BackLink, Builder, Child, Link, Node, NotSet, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Enf {
    pub parents: Vec<BackLink<Owner>>,
    pub expr: Link<Op>,
}

impl Enf {
    pub fn create(expr: Link<Op>) -> Link<Self> {
        Self {
            expr,
            ..Default::default()
        }
        .into()
    }
}

impl Link<Enf> {
    pub fn as_op(self) -> Link<Op> {
        Op::Enf(self).into()
    }
    pub fn as_owner(self) -> Link<Owner> {
        Owner::Enf(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::Enf(self).into()
    }
}

impl Parent for Enf {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.expr.clone()])
    }
}

impl Child for Enf {
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

pub struct EnfBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parents: Vec<BackLink<Owner>>,
    expr: Option<Link<Op>>,
}

type EnfBuilderEmpty = EnfBuilder<(BackLink<Owner>, NotSet)>;
type EnfBuilderFull = EnfBuilder<(BackLink<Owner>, Link<Op>)>;

impl Builder for Enf {
    type BuilderEmpty = EnfBuilderEmpty;
    type BuilderFull = EnfBuilderFull;
    fn builder() -> Self::BuilderEmpty {
        EnfBuilder::default()
    }
}

impl Default for EnfBuilderEmpty {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parents: Vec::default(),
            expr: None,
        }
    }
}

impl EnfBuilderEmpty {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
        self
    }
    pub fn expr(mut self, expr: Link<Op>) -> EnfBuilderFull {
        self.expr = Some(expr);
        unsafe { std::mem::transmute(self) }
    }
}

impl EnfBuilderFull {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
        self
    }
    pub fn expr(mut self, expr: Link<Op>) -> Self {
        self.expr = Some(expr);
        self
    }
    pub fn build(self) -> Link<Enf> {
        Enf {
            parents: self.parents,
            expr: self.expr.unwrap(),
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;
    use crate::ir::{Add, Evaluator};

    #[test]
    fn test_enf_builder() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default().into()));
        let expr = Link::new(Add::default()).as_op();
        let enf = Enf::builder()
            .parent(parent.clone())
            .expr(expr.clone())
            .build();
        assert_eq!(
            enf.borrow().deref(),
            &Enf {
                parents: vec![parent.into()],
                expr
            }
        );
    }
}
