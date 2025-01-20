use crate::ir::{Builder, Link, Op, Parent, Root};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder)]
#[enum_wrapper(Root)]
pub struct Evaluator {
    pub parameters: Vec<Vec<Link<Op>>>,
    pub body: Link<Vec<Link<Op>>>,
}

impl Evaluator {
    pub fn create(parameters: Vec<Vec<Link<Op>>>, body: Vec<Link<Op>>) -> Link<Root> {
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
