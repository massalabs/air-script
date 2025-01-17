use crate::ir::{Builder, Link, Op, Parent, Root};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder)]
#[enum_wrapper(Root)]
pub struct Function {
    pub parameters: Vec<Link<Op>>, // Parameter
    pub return_type: Link<Op>,     // Parameter
    pub body: Link<Vec<Link<Op>>>,
}

impl Function {
    pub fn create(
        parameters: Vec<Link<Op>>,
        return_type: Link<Op>,
        body: Vec<Link<Op>>,
    ) -> Link<Root> {
        Root::Function(Self {
            parameters,
            return_type,
            body: Link::new(body),
        })
        .into()
    }
}

impl Parent for Function {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.body.clone()
    }
}
