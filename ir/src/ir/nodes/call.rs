use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner, Parent, Root};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder)]
pub struct Call {
    pub parents: Vec<BackLink<Owner>>,
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
    fn get_parents(&self) -> Vec<BackLink<Self::Parent>> {
        self.parents.clone()
    }
    fn add_parent(&mut self, parent: Link<Self::Parent>) {
        self.parents.push(parent.into());
    }
    fn remove_parent(&mut self, parent: Link<Self::Parent>) {
        self.parents.retain(|p| *p != parent.clone().into());
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
            .parents(parent.clone())
            .function(function.clone())
            .arguments(arg.clone())
            .build();
        assert_eq!(
            call.borrow().deref(),
            &Call {
                parents: vec![parent.into()],
                function: function.clone(),
                arguments: Link::new(vec![arg.clone()])
            }
        );
    }
}
