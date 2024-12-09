use std::hash::Hash;

use crate::ir3::{BackLink, Builder, Child, Link, NotSet, Op, Owner, Parent};

use air_parser::ast::Boundary as BoundaryKind;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Boundary {
    parent: BackLink<Owner>,
    kind: BoundaryKind,
    expr: Link<Op>,
}

impl Default for Boundary {
    fn default() -> Self {
        Self {
            parent: BackLink::default(),
            kind: BoundaryKind::First,
            expr: Link::default(),
        }
    }
}

impl Hash for Boundary {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match &self.kind {
            BoundaryKind::First => 0.hash(state),
            BoundaryKind::Last => 1.hash(state),
        }
        self.expr.hash(state);
    }
}

impl Boundary {
    pub fn new(expr: Link<Op>) -> Self {
        Self {
            expr,
            ..Default::default()
        }
    }
}

impl Parent for Boundary {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![self.expr.clone()])
    }
}

impl Child for Boundary {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}

pub struct BoundaryBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parent: BackLink<Owner>,
    kind: Option<BoundaryKind>,
    expr: Option<Link<Op>>,
}

type BoundaryBuilderStart = BoundaryBuilder<(BackLink<Owner>, NotSet, NotSet)>;
type BoundaryBuilderA = BoundaryBuilder<(BackLink<Owner>, BoundaryKind, NotSet)>;
type BoundaryBuilderB = BoundaryBuilder<(BackLink<Owner>, NotSet, Link<Op>)>;
type BoundaryBuilderFinish = BoundaryBuilder<(BackLink<Owner>, BoundaryKind, Link<Op>)>;

impl Builder for Boundary {
    type BuilderType = BoundaryBuilderStart;
    fn builder() -> Self::BuilderType {
        BoundaryBuilder::default()
    }
}

impl Default for BoundaryBuilderStart {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parent: BackLink::default(),
            kind: None,
            expr: None,
        }
    }
}

impl BoundaryBuilderStart {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn kind(mut self, kind: BoundaryKind) -> BoundaryBuilderA {
        self.kind = Some(kind);
        unsafe { std::mem::transmute(self) }
    }
    pub fn expr(mut self, expr: Link<Op>) -> BoundaryBuilderB {
        self.expr = Some(expr);
        unsafe { std::mem::transmute(self) }
    }
}

impl BoundaryBuilderA {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn kind(mut self, kind: BoundaryKind) -> Self {
        self.kind = Some(kind);
        self
    }
    pub fn expr(mut self, expr: Link<Op>) -> BoundaryBuilderFinish {
        self.expr = Some(expr);
        unsafe { std::mem::transmute(self) }
    }
}

impl BoundaryBuilderB {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn kind(mut self, kind: BoundaryKind) -> BoundaryBuilderFinish {
        self.kind = Some(kind);
        unsafe { std::mem::transmute(self) }
    }
    pub fn expr(mut self, expr: Link<Op>) -> Self {
        self.expr = Some(expr);
        self
    }
}

impl BoundaryBuilderFinish {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn kind(mut self, kind: BoundaryKind) -> Self {
        self.kind = Some(kind);
        self
    }
    pub fn expr(mut self, expr: Link<Op>) -> Self {
        self.expr = Some(expr);
        self
    }
    pub fn build(self) -> Boundary {
        Boundary {
            parent: self.parent,
            kind: self.kind.unwrap(),
            expr: self.expr.unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ir3::Evaluator;

    use super::*;

    #[test]
    fn test_boundary() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default()));
        let expr = Link::new(Op::default());
        let boundary = Boundary::builder()
            .parent(parent.clone())
            .kind(BoundaryKind::Last)
            .expr(expr.clone())
            .build();
        assert_eq!(
            boundary,
            Boundary {
                parent: parent.into(),
                kind: BoundaryKind::Last,
                expr: expr.clone()
            }
        );
    }
}
