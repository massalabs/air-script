use std::hash::Hash;

use crate::ir::{BackLink, Builder, Child, Link, Node, NotSet, Op, Owner, Parent};

use air_parser::ast::Boundary as BoundaryKind;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Boundary {
    pub parents: Vec<BackLink<Owner>>,
    pub kind: BoundaryKind,
    pub expr: Link<Op>,
}

impl Default for Boundary {
    fn default() -> Self {
        Self {
            parents: Vec::default(),
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
    pub fn create(expr: Link<Op>, kind: BoundaryKind) -> Link<Self> {
        Self {
            expr,
            kind,
            ..Default::default()
        }
        .into()
    }
}

impl Link<Boundary> {
    pub fn as_op(self) -> Link<Op> {
        Op::Boundary(self).into()
    }
    pub fn as_owner(self) -> Link<Owner> {
        Owner::Boundary(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::Boundary(self).into()
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

pub struct BoundaryBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parents: Vec<BackLink<Owner>>,
    kind: Option<BoundaryKind>,
    expr: Option<Link<Op>>,
}

type BoundaryBuilderEmpty = BoundaryBuilder<(BackLink<Owner>, NotSet, NotSet)>;
type BoundaryBuilderA = BoundaryBuilder<(BackLink<Owner>, BoundaryKind, NotSet)>;
type BoundaryBuilderB = BoundaryBuilder<(BackLink<Owner>, NotSet, Link<Op>)>;
type BoundaryBuilderFull = BoundaryBuilder<(BackLink<Owner>, BoundaryKind, Link<Op>)>;

impl Builder for Boundary {
    type BuilderEmpty = BoundaryBuilderEmpty;
    type BuilderFull = BoundaryBuilderFull;
    fn builder() -> Self::BuilderEmpty {
        BoundaryBuilder::default()
    }
}

impl Default for BoundaryBuilderEmpty {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parents: Vec::default(),
            kind: None,
            expr: None,
        }
    }
}

impl BoundaryBuilderEmpty {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
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
        self.parents.push(parent.into());
        self
    }
    pub fn kind(mut self, kind: BoundaryKind) -> Self {
        self.kind = Some(kind);
        self
    }
    pub fn expr(mut self, expr: Link<Op>) -> BoundaryBuilderFull {
        self.expr = Some(expr);
        unsafe { std::mem::transmute(self) }
    }
}

impl BoundaryBuilderB {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
        self
    }
    pub fn kind(mut self, kind: BoundaryKind) -> BoundaryBuilderFull {
        self.kind = Some(kind);
        unsafe { std::mem::transmute(self) }
    }
    pub fn expr(mut self, expr: Link<Op>) -> Self {
        self.expr = Some(expr);
        self
    }
}

impl BoundaryBuilderFull {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
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
    pub fn build(self) -> Link<Boundary> {
        Boundary {
            parents: self.parents,
            kind: self.kind.unwrap(),
            expr: self.expr.unwrap(),
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use crate::ir::Evaluator;

    use super::*;

    #[test]
    fn test_boundary() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default().into()));
        let expr = Link::new(Op::default());
        let boundary = Boundary::builder()
            .parent(parent.clone())
            .kind(BoundaryKind::Last)
            .expr(expr.clone())
            .build();
        assert_eq!(
            boundary.borrow().deref(),
            &Boundary {
                parents: vec![parent.into()],
                kind: BoundaryKind::Last,
                expr: expr.clone()
            }
        );
    }
}
