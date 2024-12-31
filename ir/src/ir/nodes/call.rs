use crate::ir::{BackLink, Builder, Child, Link, Node, NotSet, Op, Owner, Parent, Root};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Call {
    pub parent: BackLink<Owner>,
    pub function: Link<Root>,
    /// Parent::children only contains the arguments
    pub arguments: Link<Vec<Link<Op>>>,
}

impl Call {
    pub fn create(function: Link<Root>, arguments: Vec<Link<Op>>) -> Link<Self> {
        Self {
            function,
            arguments: Link::new(arguments),
            ..Default::default()
        }
        .into()
    }
}

impl Link<Call> {
    pub fn as_op(self) -> Link<Op> {
        Op::Call(self).into()
    }
    pub fn as_owner(self) -> Link<Owner> {
        Owner::Call(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::Call(self).into()
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
    pub fn argument(mut self, argument: Link<Op>) -> Self {
        self.arguments.push(argument);
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
    pub fn build(self) -> Link<Call> {
        Call {
            parent: self.parent,
            function: self.function.unwrap(),
            arguments: Link::new(self.arguments),
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;
    use crate::ir::{nodes::value::MirType, Add, Function, Owner, Parameter};

    #[test]
    fn test_call_builder() {
        let parent = Link::new(Owner::Add(Add::default().into()));
        let function = Link::new(Root::Function(
            Function::builder()
                .parameters(Parameter::create(0, MirType::Felt).into())
                .return_type(Parameter::create(1, MirType::Felt).into())
                .build(),
        ));
        let arg = Link::new(Op::Add(Add::default().into()));
        let call = Call::builder()
            .parent(parent.clone())
            .function(function.clone())
            .argument(arg.clone())
            .build();
        assert_eq!(
            call.borrow().deref(),
            &Call {
                parent: parent.into(),
                function: function.clone(),
                arguments: Link::new(vec![arg.clone()])
            }
        );
    }
}
