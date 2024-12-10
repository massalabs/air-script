use crate::ir3::{BackLink, Builder, Child, Link, NotSet, Op, Owner, Parent, Vector};

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

pub struct ForBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parent: BackLink<Owner>,
    iterators: Vec<Link<Vector>>,
    expr: Option<Link<Op>>,
    selector: Option<Link<Op>>,
}

type ForBuilderStart = ForBuilder<(BackLink<Owner>, Vec<Link<Vector>>, NotSet, NotSet)>;
type ForBuilderA = ForBuilder<(BackLink<Owner>, Vec<Link<Vector>>, Link<Op>, NotSet)>;
type ForBuilderB = ForBuilder<(BackLink<Owner>, Vec<Link<Vector>>, NotSet, Link<Op>)>;
type ForBuilderFinish = ForBuilder<(BackLink<Owner>, Vec<Link<Vector>>, Link<Op>, Link<Op>)>;

impl Builder for For {
    type BuilderType = ForBuilderStart;
    fn builder() -> Self::BuilderType {
        ForBuilder::default()
    }
}

impl Default for ForBuilderStart {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parent: BackLink::default(),
            iterators: Vec::new(),
            expr: None,
            selector: None,
        }
    }
}

impl ForBuilderStart {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn iterators(mut self, iterator: Link<Vector>) -> Self {
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
        self.parent = parent.into();
        self
    }
    pub fn iterators(mut self, iterator: Link<Vector>) -> Self {
        self.iterators.push(iterator);
        self
    }
    pub fn expr(mut self, expr: Link<Op>) -> Self {
        self.expr = Some(expr);
        self
    }
    pub fn selector(mut self, selector: Link<Op>) -> ForBuilderFinish {
        self.selector = Some(selector);
        unsafe { std::mem::transmute(self) }
    }
}

impl ForBuilderB {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn iterators(mut self, iterator: Link<Vector>) -> Self {
        self.iterators.push(iterator);
        self
    }
    pub fn expr(mut self, expr: Link<Op>) -> ForBuilderFinish {
        self.expr = Some(expr);
        unsafe { std::mem::transmute(self) }
    }
    pub fn selector(mut self, selector: Link<Op>) -> Self {
        self.selector = Some(selector);
        self
    }
}

impl ForBuilderFinish {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn iterators(mut self, iterator: Link<Vector>) -> Self {
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
    pub fn build(self) -> For {
        For {
            parent: self.parent,
            iterators: Link::new(self.iterators),
            expr: self.expr.expect("expr"),
            selector: self.selector.expect("selector"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ir3::{Add, Evaluator, Sub};

    use super::*;

    #[test]
    fn test_for_builder() {
        let parent = Link::new(Owner::Evaluator(Evaluator::default()));
        let i_a = Link::new(Vector::default());
        let i_b = Link::new(Vector::default());
        let expr = Link::new(Op::Add(Add::default()));
        let selector = Link::new(Op::Sub(Sub::default()));
        let for_op = For::builder()
            .parent(parent.clone())
            .iterators(i_a.clone())
            .iterators(i_b.clone())
            .expr(expr.clone())
            .selector(selector.clone())
            .build();
        assert_eq!(
            for_op,
            For {
                parent: parent.clone().into(),
                iterators: Link::new(vec![i_a.clone(), i_b.clone()]),
                expr: expr.clone(),
                selector: selector.clone(),
            }
        );
    }
}
