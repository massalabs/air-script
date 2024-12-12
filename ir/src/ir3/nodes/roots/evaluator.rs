use crate::ir3::{Builder, Link, Node, NotSet, Op, Owner, Parameter, Parent, Root};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Evaluator {
    pub parameters: Vec<Link<Parameter>>,
    pub body: Link<Vec<Link<Op>>>,
}

impl Evaluator {
    pub fn new(parameters: Vec<Link<Parameter>>, body: Vec<Link<Op>>) -> Self {
        Self {
            parameters,
            body: Link::new(body),
        }
    }
    pub fn as_root(self) -> Root {
        Root::Evaluator(self)
    }
    pub fn as_owner(self) -> Owner {
        Owner::Evaluator(self)
    }
    pub fn as_node(self) -> Node {
        Node::Evaluator(self)
    }
}

impl Link<Evaluator> {
    pub fn as_root(self) -> Link<Root> {
        Link::new(Root::Evaluator(self.borrow().clone()))
    }
    pub fn as_owner(self) -> Link<Owner> {
        Link::new(Owner::Evaluator(self.borrow().clone()))
    }
    pub fn as_node(self) -> Link<Node> {
        Link::new(Node::Evaluator(self.borrow().clone()))
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
    fn edit(self) -> Self::BuilderFull {
        Self::BuilderFull {
            _state: std::marker::PhantomData,
            parameters: self.parameters,
            body: self.body.borrow().clone(),
        }
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
    pub fn parameters(mut self, parameter: Parameter) -> Self {
        self.parameters.push(Link::from(parameter));
        self
    }
    pub fn body(mut self, op: Op) -> Self {
        self.body.push(Link::from(op));
        self
    }
    pub fn build(self) -> Evaluator {
        Evaluator::new(self.parameters, self.body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir3::{nodes::value::MirType, Add, Owner};

    #[test]
    fn test_evaluator_builder() {
        let a = Parameter::new(0, MirType::Felt);
        let b = Parameter::new(1, MirType::Felt);
        let ev = Evaluator::builder()
            .parameters(a.clone())
            .parameters(b.clone())
            .body(Op::Add(Add::default()))
            .build();
        assert_eq!(ev.parameters.len(), 2);
        assert_eq!(ev.parameters[0].clone(), a.clone().into());
        assert_eq!(ev.parameters[1].clone(), b.clone().into());
        assert_eq!(ev.body.borrow().len(), 1);
    }
}
