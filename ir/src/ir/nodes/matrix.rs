use crate::ir::{BackLink, Builder, Child, Link, Node, Op, Owner, Parent, Vector};

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash, Builder)]
pub struct Matrix {
    pub parents: Vec<BackLink<Owner>>,
    pub size: usize,
    pub elements: Link<Vec<Link<Vector>>>,
}

impl Matrix {
    pub fn create(elements: Vec<Link<Vector>>) -> Link<Self> {
        let size = elements.len();
        Self {
            size,
            elements: Link::new(elements),
            ..Default::default()
        }
        .into()
    }
}

impl Link<Matrix> {
    pub fn as_op(self) -> Link<Op> {
        Op::Matrix(self).into()
    }
    pub fn as_owner(self) -> Link<Owner> {
        Owner::Matrix(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::Matrix(self).into()
    }
}

impl Parent for Matrix {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.elements
            .clone()
            .borrow_mut()
            .iter()
            .map(|element| element.clone().as_op())
            .collect::<Vec<_>>()
            .into()
    }
}

impl Child for Matrix {
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

    #[test]
    fn test_matrix_builder() {
        let parent = Link::new(Owner::default());
        let a = Link::new(Vector::default());
        let b = Link::new(Vector::default());
        let matrix = MatrixBuilder::default()
            .parents(parent.clone())
            .size(2)
            .elements(a.clone())
            .elements(b.clone())
            .build();
        assert_eq!(
            matrix.borrow().deref(),
            &Matrix {
                parents: vec![parent.into()],
                size: 2,
                elements: Link::new(vec![a, b]),
            }
        );
    }
}
