use std::ops::Deref;

use crate::ir3::{BackLink, Builder, Child, Link, NotSet, Op, Owner, Parent, Root};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Call {
    parent: BackLink<Owner>,
    function: Link<Root>,
    /// Parent::children only contains the arguments
    arguments: Link<Vec<Link<Op>>>,
}

impl Call {
    pub fn new(function: Link<Root>, arguments: Vec<Link<Op>>) -> Self {
        Self {
            function,
            arguments: Link::new(arguments),
            ..Default::default()
        }
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

type CallBuilderStart = CallBuilder<(BackLink<Owner>, NotSet, Vec<Link<Op>>)>;
type CallBuilderFinish = CallBuilder<(BackLink<Owner>, Link<Root>, Vec<Link<Op>>)>;

impl Builder for Call {
    type BuilderType = CallBuilderStart;
    fn builder() -> Self::BuilderType {
        CallBuilder::default()
    }
}

impl Default for CallBuilderStart {
    fn default() -> Self {
        Self {
            _state: std::marker::PhantomData,
            parent: BackLink::default(),
            function: None,
            arguments: Vec::new(),
        }
    }
}

impl CallBuilderStart {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn function(mut self, function: Link<Root>) -> CallBuilderFinish {
        self.function = Some(function);
        unsafe { std::mem::transmute(self) }
    }
    pub fn arguments(mut self, arguments: Vec<Link<Op>>) -> Self {
        self.arguments = arguments;
        self
    }
}

impl CallBuilderFinish {
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
        if self.arguments.len()
            != match self.function.clone().unwrap().borrow().deref() {
                Root::Function(func) => func.parameters.len(),
                Root::Evaluator(ev) => ev.parameters.len(),
                _ => unreachable!(),
            }
        {
            panic!("wrong number of arguments");
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
    use crate::ir3::{Add, Function, Owner, Parameter};

    #[test]
    fn test_call_builder() {
        let parent = Link::new(Owner::Add(Add::default()));
        let function = Link::new(Root::Function(
            Function::builder()
                .parameters(Parameter::new(0))
                .return_type(Parameter::new(1))
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
