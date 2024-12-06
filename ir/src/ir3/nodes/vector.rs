use crate::ir3::{BackLink, Child, Link, Op, Parent};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Vector {
    parent: BackLink<Op>,
    size: usize,
    elements: Link<Vec<Link<Op>>>,
}

impl Vector {
    pub fn new(elements: Vec<Link<Op>>) -> Self {
        let size = elements.len();
        Self {
            size,
            elements: Link::new(elements),
            ..Default::default()
        }
    }
}

impl Parent for Vector {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.elements.clone()
    }
}

impl Child for Vector {
    type Parent = Op;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}
