use crate::ir3::{BackLink, Child, Link, Op, Parent, Root};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Call {
    parent: BackLink<Op>,
    function: Link<Root>,
    /// Parent::children only contains the arguments
    arguments: Vec<Link<Op>>,
}

impl Call {
    pub fn new(function: Link<Root>, arguments: Vec<Link<Op>>) -> Self {
        Self {
            function,
            arguments,
            ..Default::default()
        }
    }
}

impl Parent for Call {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        Link::new(self.arguments.clone())
    }
}

impl Child for Call {
    type Parent = Op;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}
