use crate::ir::{Builder, Link, Node, Op, Owner, Parameter, Parent, Root};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Evaluator {
    pub parameters: Vec<Link<Parameter>>,
    pub body: Link<Vec<Link<Op>>>,
}

impl Evaluator {
    pub fn create(parameters: Vec<Link<Parameter>>, body: Vec<Link<Op>>) -> Link<Self> {
        Self {
            parameters,
            body: Link::new(body),
        }
        .into()
    }
}

impl Link<Evaluator> {
    pub fn as_root(self) -> Link<Root> {
        Root::Evaluator(self).into()
    }
    pub fn as_owner(self) -> Link<Owner> {
        Owner::Evaluator(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::Evaluator(self).into()
    }
}

impl Parent for Evaluator {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.body.clone()
    }
}

pub struct EvaluatorBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parameters: Vec<Link<Parameter>>,
    body: Vec<Link<Op>>,
}

type EvaluatorBuilderState = EvaluatorBuilder<(Vec<Link<Parameter>>, Vec<Link<Op>>)>;

impl Builder for Evaluator {
    type BuilderEmpty = EvaluatorBuilderState;
    type BuilderFull = EvaluatorBuilderState;
    fn builder() -> Self::BuilderEmpty {
        EvaluatorBuilder::default()
    }
}

impl Default for EvaluatorBuilderState {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parameters: Vec::new(),
            body: Vec::new(),
        }
    }
}

impl EvaluatorBuilderState {
    pub fn parameters(mut self, parameter: Link<Parameter>) -> Self {
        self.parameters.push(parameter);
        self
    }
    pub fn body(mut self, op: Link<Op>) -> Self {
        self.body.push(op);
        self
    }
    pub fn build(self) -> Link<Evaluator> {
        Evaluator::create(self.parameters, self.body)
    }
}

#[cfg(test)]
mod tests {

    use std::ops::Deref;

    use super::*;
    use crate::ir::{Add, MirType};

    #[test]
    fn test_evaluator_builder() {
        let a = Parameter::create(0, MirType::Felt);
        let b = Parameter::create(1, MirType::Felt);
        let ev = Evaluator::builder()
            .parameters(a.clone())
            .parameters(b.clone())
            .body(Op::Add(Add::default().into()).into())
            .build();
        assert_eq!(
            ev.borrow().deref(),
            &Evaluator {
                parameters: vec![a.into(), b.into()],
                body: Link::new(vec![Op::Add(Add::default().into()).into()]),
            }
        );
    }
}
