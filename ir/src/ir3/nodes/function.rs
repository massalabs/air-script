use crate::ir3::{BackLink, Child, Link, Op, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Function {
    parameters: Vec<Link<Parameter>>,
    return_type: Link<Parameter>,
    body: Link<Vec<Link<Op>>>,
}

impl Function {
    pub fn new(
        parameters: Vec<Link<Parameter>>,
        return_type: Link<Parameter>,
        body: Vec<Link<Op>>,
    ) -> Self {
        Self {
            parameters,
            return_type,
            body: Link::new(body),
        }
    }
}

impl Parent for Function {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.body.clone()
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Parameter {
    position: usize,
}

impl Parameter {
    pub fn new(position: usize) -> Self {
        Self { position }
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Evaluator {
    parameters: Vec<Link<Parameter>>,
    body: Link<Vec<Link<Op>>>,
}

impl Parent for Evaluator {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.body.clone()
    }
}
