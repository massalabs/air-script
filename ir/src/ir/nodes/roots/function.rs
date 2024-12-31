use crate::ir::{Builder, Link, Node, NotSet, Op, Owner, Parameter, Parent, Root};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
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

pub struct FunctionBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parameters: Vec<Link<Parameter>>,
    return_type: Option<Link<Parameter>>,
    body: Vec<Link<Op>>,
}

type FunctionBuilderEmpty = FunctionBuilder<(Vec<Link<Parameter>>, NotSet, Vec<Link<Op>>)>;
type FunctionBuilderFull = FunctionBuilder<(Vec<Link<Parameter>>, Link<Parameter>, Vec<Link<Op>>)>;

impl Builder for Function {
    type BuilderEmpty = FunctionBuilderEmpty;
    type BuilderFull = FunctionBuilderFull;
    fn builder() -> Self::BuilderEmpty {
        FunctionBuilder::default()
    }
    fn edit(self) -> Self::BuilderFull {
        Self::BuilderFull {
            _state: std::marker::PhantomData,
            parameters: self.parameters,
            return_type: Some(self.return_type),
            body: self.body.borrow().clone(),
        }
    }
}

impl Default for FunctionBuilderEmpty {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parameters: Vec::new(),
            return_type: None,
            body: Vec::new(),
        }
    }
}

impl FunctionBuilderEmpty {
    pub fn parameters(mut self, parameter: Link<Parameter>) -> Self {
        self.parameters.push(parameter);
        self
    }
    pub fn return_type(mut self, return_type: Link<Parameter>) -> FunctionBuilderFull {
        self.return_type = Some(return_type);
        unsafe { std::mem::transmute(self) }
    }
    pub fn body(&mut self, op: Link<Op>) -> &mut Self {
        self.body.push(op);
        self
    }
}

impl FunctionBuilderFull {
    pub fn parameters(mut self, parameter: Link<Parameter>) -> Self {
        self.parameters.push(parameter);
        self
    }
    pub fn return_type(mut self, return_type: Link<Parameter>) -> Self {
        self.return_type = Some(return_type);
        self
    }
    pub fn body(mut self, op: Link<Op>) -> Self {
        self.body.push(op);
        self
    }
    pub fn build(self) -> Link<Function> {
        Function::create(self.parameters, self.return_type.unwrap(), self.body)
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
