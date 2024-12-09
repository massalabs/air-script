use crate::ir3::{BackLink, Builder, Child, Link, NotSet, Op, Owner, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Enf {
    parent: BackLink<Owner>,
    expr: Link<Op>,
}

impl Enf {
    pub fn new(expr: Link<Op>) -> Self {
        Self {
            expr,
            ..Default::default()
        }
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

type EnfBuilderStart = EnfBuilder<(BackLink<Owner>, NotSet)>;
type EnfBuilderFinish = EnfBuilder<(BackLink<Owner>, Link<Op>)>;

impl Builder for Enf {
    type BuilderType = EnfBuilderStart;
    fn builder() -> Self::BuilderType {
        EnfBuilder::default()
    }
}

impl Default for EnfBuilderStart {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parent: BackLink::default(),
            expr: None,
        }
    }
}

impl EnfBuilderStart {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn expr(mut self, expr: Op) -> EnfBuilderFinish {
        self.expr = Some(Link::new(expr));
        unsafe { std::mem::transmute(self) }
    }
}

impl EnfBuilderFinish {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn expr(mut self, expr: Op) -> Self {
        self.expr = Some(Link::new(expr));
        self
    }
    pub fn build(self) -> Enf {
        Enf {
            parent: self.parent,
            expr: self.expr.unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir3::{Add, Evaluator};

    #[test]
    fn test_enf_builder() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default()));
        let expr = Op::Add(Add::default());
        let enf = Enf::builder()
            .parent(parent.clone())
            .expr(expr.clone())
            .build();
        assert_eq!(
            enf,
            Enf {
                parent: parent.into(),
                expr: expr.into()
            }
        );
    }
}
