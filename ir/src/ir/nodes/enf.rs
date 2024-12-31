use crate::ir::{BackLink, Builder, Child, Link, Node, NotSet, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Enf {
    pub parent: BackLink<Owner>,
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
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}

pub struct EnfBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parent: BackLink<Owner>,
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
    fn edit(self) -> Self::BuilderFull {
        Self::BuilderFull {
            _state: std::marker::PhantomData,
            parent: self.parent,
            expr: Some(self.expr),
        }
    }
}

impl Default for EnfBuilderEmpty {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parent: BackLink::default(),
            expr: None,
        }
    }
}

impl EnfBuilderEmpty {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn expr(mut self, expr: Link<Op>) -> EnfBuilderFull {
        self.expr = Some(expr);
        unsafe { std::mem::transmute(self) }
    }
}

impl EnfBuilderFull {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn expr(mut self, expr: Link<Op>) -> Self {
        self.expr = Some(expr);
        self
    }
    pub fn build(self) -> Link<Enf> {
        Enf {
            parent: self.parent,
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
                parent: parent.into(),
                expr
            }
        );
    }
}
