use crate::ir_fix::{Builder, Link, Op, Owner, Parameter, Parent, Root};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder)]
pub struct Function {
    pub parameters: Vec<Link<Parameter>>,
    pub return_type: Link<Parameter>,
    pub body: Link<Vec<Link<Op>>>,
}

impl Function {
    pub fn create(
        parameters: Vec<Link<Parameter>>,
        return_type: Link<Parameter>,
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
