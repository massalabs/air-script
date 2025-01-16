use crate::ir_fix::{Builder, Link, Op, Owner, Parameter, Parent, Root};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder)]
pub struct Evaluator {
    pub parameters: Vec<Link<Parameter>>,
    pub body: Link<Vec<Link<Op>>>,
}

impl Evaluator {
    pub fn create(parameters: Vec<Link<Parameter>>, body: Vec<Link<Op>>) -> Link<Root> {
        Root::Evaluator(Self {
            parameters,
            body: Link::new(body),
        })
        .into()
    }
}

impl Parent for Evaluator {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.body.clone()
    }
}
