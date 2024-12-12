use crate::ir::{Builder, Link, Node, NotSet, Op, Owner, Parameter, Parent, Root};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Function {
    pub parameters: Vec<Link<Parameter>>,
    pub return_type: Link<Parameter>,
    pub body: Link<Vec<Link<Op>>>,
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
    pub fn as_root(self) -> Root {
        Root::Function(self)
    }
    pub fn as_owner(self) -> Owner {
        Owner::Function(self)
    }
    pub fn as_node(self) -> Node {
        Node::Function(self)
    }
}

impl Link<Function> {
    pub fn as_root(self) -> Link<Root> {
        Link::new(Root::Function(self.borrow().clone()))
    }
    pub fn as_owner(self) -> Link<Owner> {
        Link::new(Owner::Function(self.borrow().clone()))
    }
    pub fn as_node(self) -> Link<Node> {
        Link::new(Node::Function(self.borrow().clone()))
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
    pub fn build(self) -> Function {
        Function::new(self.parameters, self.return_type.unwrap(), self.body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{nodes::value::MirType, Add};

    #[test]
    fn test_function_builder() {
        let a = Link::new(Parameter::new(0, MirType::Felt));
        let b = Link::new(Parameter::new(1, MirType::Felt));
        let return_type = Link::new(Parameter::new(2, MirType::Felt));
        let func = Function::builder()
            .parameters(a.clone())
            .parameters(b.clone())
            .return_type(return_type.clone())
            .body(Op::Add(Add::default()).into())
            .build();
        assert_eq!(func.parameters.len(), 2);
        assert_eq!(func.parameters[0].clone(), a.clone().into());
        assert_eq!(func.parameters[1].clone(), b.clone().into());
        assert_eq!(func.return_type.clone(), return_type.clone().into());
        assert_eq!(func.body.borrow().len(), 1);
    }
}
