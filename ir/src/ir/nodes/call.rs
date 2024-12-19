use std::ops::Deref;

use crate::ir::{BackLink, Builder, Child, Link, Node, NotSet, Op, Owner, Parent, Root};

use super::{MirValue, SpannedMirValue, TraceAccessBinding};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Call {
    pub parent: BackLink<Owner>,
    pub function: Link<Root>,
    /// Parent::children only contains the arguments
    pub arguments: Link<Vec<Link<Op>>>,
}

impl Call {
    pub fn new(function: Link<Root>, arguments: Vec<Link<Op>>) -> Self {
        Self {
            function,
            arguments: Link::new(arguments),
            ..Default::default()
        }
    }
    pub fn as_op(self) -> Op {
        Op::Call(self)
    }
    pub fn as_owner(self) -> Owner {
        Owner::Call(self)
    }
    pub fn as_node(self) -> Node {
        Node::Call(self)
    }
}

impl Link<Call> {
    pub fn as_op(self) -> Link<Op> {
        Link::new(Op::Call(self.borrow().clone()))
    }
    pub fn as_owner(self) -> Link<Owner> {
        Link::new(Owner::Call(self.borrow().clone()))
    }
    pub fn as_node(self) -> Link<Node> {
        Link::new(Node::Call(self.borrow().clone()))
    }
}

impl Parent for Call {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.arguments.clone()
    }
}

impl Child for Call {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}

pub struct CallBuilder<State> {
    _state: std::marker::PhantomData<State>,
    parent: BackLink<Owner>,
    function: Option<Link<Root>>,
    arguments: Vec<Link<Op>>,
}

type CallBuilderEmpty = CallBuilder<(BackLink<Owner>, NotSet, Vec<Link<Op>>)>;
type CallBuilderFull = CallBuilder<(BackLink<Owner>, Link<Root>, Vec<Link<Op>>)>;

impl Builder for Call {
    type BuilderEmpty = CallBuilderEmpty;
    type BuilderFull = CallBuilderFull;
    fn builder() -> Self::BuilderEmpty {
        CallBuilder::default()
    }
    fn edit(self) -> Self::BuilderFull {
        Self::BuilderFull {
            _state: std::marker::PhantomData,
            parent: self.parent,
            function: Some(self.function),
            arguments: self.arguments.borrow().clone(),
        }
    }
}

impl Default for CallBuilderEmpty {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parent: BackLink::default(),
            function: None,
            arguments: Vec::new(),
        }
    }
}

impl CallBuilderEmpty {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn function(mut self, function: Link<Root>) -> CallBuilderFull {
        self.function = Some(function);
        unsafe { std::mem::transmute(self) }
    }
    pub fn arguments(mut self, arguments: Vec<Link<Op>>) -> Self {
        self.arguments = arguments;
        self
    }
}

impl CallBuilderFull {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn function(mut self, function: Link<Root>) -> Self {
        self.function = Some(function);
        self
    }
    pub fn argument(mut self, argument: Link<Op>) -> Self {
        self.arguments.push(argument);
        self
    }
    pub fn build(self) -> Call {
        let num_args = match self.function.clone().unwrap().borrow().deref() {
            Root::Function(func) => self.arguments.len(),
            Root::Evaluator(ev) => ev
                .parameters
                .iter()
                .map(|p| match p.borrow().deref().value {
                    SpannedMirValue {
                        span: _,
                        value:
                            MirValue::TraceAccessBinding(TraceAccessBinding {
                                segment: _,
                                offset: _,
                                size,
                            }),
                    } => size,
                    _ => unreachable!(),
                })
                .sum(),
            Root::None => unreachable!(),
        };
        if num_args
            != match self.function.clone().unwrap().borrow().deref() {
                Root::Function(func) => func.parameters.len(),
                Root::Evaluator(ev) => ev
                    .parameters
                    .iter()
                    .map(|p| match p.borrow().deref().value {
                        SpannedMirValue {
                            span: _,
                            value:
                                MirValue::TraceAccessBinding(TraceAccessBinding {
                                    segment: _,
                                    offset: _,
                                    size,
                                }),
                        } => size,
                        _ => unreachable!(),
                    })
                    .sum(),
                Root::None => unreachable!(),
            }
        {
            panic!(
                "wrong number of arguments {:#?}\n    for function {:#?}",
                self.arguments,
                self.function.unwrap()
            );
        }
        Call {
            parent: self.parent,
            function: self.function.unwrap(),
            arguments: Link::new(self.arguments),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{nodes::value::MirType, Add, Function, Owner, Parameter};

    #[test]
    fn test_call_builder() {
        let parent = Link::new(Owner::Add(Add::default()));
        let function = Link::new(Root::Function(
            Function::builder()
                .parameters(Parameter::new(0, MirType::Felt).into())
                .return_type(Parameter::new(1, MirType::Felt).into())
                .build(),
        ));
        let arg = Link::new(Op::Add(Add::default()));
        let call = Call::builder()
            .parent(parent.clone())
            .function(function.clone())
            .argument(arg.clone())
            .build();
        assert_eq!(
            call,
            Call {
                parent: parent.into(),
                function: function.clone(),
                arguments: Link::new(vec![arg.clone()])
            }
        );
    }
}
