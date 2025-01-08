use crate::ir::{Builder, Link, Node, NotSet, Op, Owner, Parameter, Parent, Root};

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
    ) -> Link<Self> {
        Self {
            parameters,
            return_type,
            body: Link::new(body),
        }
        .into()
    }
}

impl Link<Function> {
    pub fn as_root(self) -> Link<Root> {
        Root::Function(self).into()
    }
    pub fn as_owner(self) -> Link<Owner> {
        Owner::Function(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::Function(self).into()
    }
}

impl Parent for Function {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.body.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;
    use crate::ir::{nodes::value::MirType, Add};

    #[test]
    fn test_function_builder() {
        let a = Parameter::create(0, MirType::Felt);
        let b = Parameter::create(1, MirType::Felt);
        let return_type = Parameter::create(2, MirType::Felt);
        let func = Function::builder()
            .parameters(a.clone())
            .parameters(b.clone())
            .return_type(return_type.clone())
            .body(Op::Add(Add::default().into()).into())
            .build();
        assert_eq!(
            func.borrow().deref(),
            &Function {
                parameters: vec![a.into(), b.into()],
                return_type: return_type.into(),
                body: vec![Op::Add(Add::default().into()).into()].into(),
            }
        );
    }
}
