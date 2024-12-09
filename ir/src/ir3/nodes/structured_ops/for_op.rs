use crate::ir3::{BackLink, Child, Link, Op, Owner, Parent, Vector};

pub enum ForChild {
    Iterators(Link<Vec<Link<Vector>>>),
    Expr(Link<Op>),
    Selector(Link<Op>),
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct For {
    parent: BackLink<Owner>,
    iterators: Link<Vec<Link<Vector>>>,
    expr: Link<Op>,
    selector: Link<Op>,
}

impl For {
    pub fn new(iterators: Link<Vec<Link<Vector>>>, expr: Link<Op>, selector: Link<Op>) -> Self {
        Self {
            iterators,
            expr,
            selector,
            ..Default::default()
        }
    }
}

impl Parent for For {
    type Child = ForChild;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(vec![
            ForChild::Iterators(self.iterators.clone()).into(),
            ForChild::Expr(self.expr.clone()).into(),
            ForChild::Selector(self.selector.clone()).into(),
        ])
    }
}

impl Child for For {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}
