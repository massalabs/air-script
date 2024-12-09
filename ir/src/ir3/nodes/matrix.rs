use crate::ir3::{BackLink, Child, Link, Op, Owner, Parent, Vector};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Matrix {
    parent: BackLink<Owner>,
    size: usize,
    elements: Link<Vec<Link<Vector>>>,
}

impl Matrix {
    pub fn new(elements: Vec<Link<Vector>>) -> Self {
        let size = elements.len();
        Self {
            size,
            elements: Link::new(elements),
            ..Default::default()
        }
    }
}

impl Parent for Matrix {
    type Child = Vector;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.elements.clone()
    }
}

impl Child for Matrix {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}
