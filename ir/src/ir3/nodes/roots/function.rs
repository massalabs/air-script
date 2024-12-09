use crate::ir3::{Builder, Link, NotSet, Op, Parameter, Parent};

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

type FunctionBuilderStart = FunctionBuilder<(Vec<Link<Parameter>>, NotSet, Vec<Link<Op>>)>;
type FunctionBuilderFinish =
    FunctionBuilder<(Vec<Link<Parameter>>, Link<Parameter>, Vec<Link<Op>>)>;

impl Builder for Function {
    type BuilderType = FunctionBuilderStart;
    fn builder() -> Self::BuilderType {
        FunctionBuilder::default()
    }
}

impl Default for FunctionBuilderStart {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parameters: Vec::new(),
            return_type: None,
            body: Vec::new(),
        }
    }
}

impl FunctionBuilderStart {
    pub fn parameters(mut self, parameter: Parameter) -> Self {
        self.parameters.push(Link::from(parameter));
        self
    }
    pub fn return_type(mut self, return_type: Parameter) -> FunctionBuilderFinish {
        self.return_type = Some(Link::from(return_type));
        unsafe { std::mem::transmute(self) }
    }
    pub fn body(&mut self, op: Op) -> &mut Self {
        self.body.push(Link::from(op));
        self
    }
}

impl FunctionBuilderFinish {
    pub fn parameters(mut self, parameter: Parameter) -> Self {
        self.parameters.push(Link::from(parameter));
        self
    }
    pub fn return_type(mut self, return_type: Parameter) -> Self {
        self.return_type = Some(Link::from(return_type));
        self
    }
    pub fn body(mut self, op: Op) -> Self {
        self.body.push(Link::from(op));
        self
    }
    pub fn build(self) -> Function {
        Function::new(self.parameters, self.return_type.unwrap(), self.body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir3::{Add, Owner};

    #[test]
    fn test_function_builder() {
        let a = Parameter::new(0);
        let b = Parameter::new(1);
        let return_type = Parameter::new(2);
        let func = Function::builder()
            .parameters(a.clone())
            .parameters(b.clone())
            .return_type(return_type.clone())
            .body(Op::Add(Add::default()))
            .build();
        assert_eq!(func.parameters.len(), 2);
        assert_eq!(func.parameters[0].clone(), a.clone().into());
        assert_eq!(func.parameters[1].clone(), b.clone().into());
        assert_eq!(func.return_type.clone(), return_type.clone().into());
        assert_eq!(func.body.borrow().len(), 1);
    }
}
