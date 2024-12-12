use crate::ir2::{BackLink, IsChild, IsParent, Link, MiddleNode, Node, NodeType};

use std::fmt::Debug;

#[derive(Clone, Eq, PartialEq, Default)]
pub struct Vector {
    node: Node,
}

impl IsParent for Vector {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        self.node.get_children()
    }
}

impl IsChild for Vector {
    fn get_parent(&self) -> BackLink<NodeType> {
        self.node.get_parent()
    }
    fn set_parent(&mut self, parent: Link<NodeType>) {
        self.node.set_parent(parent)
    }
}

impl From<Vector> for Link<NodeType> {
    fn from(vector: Vector) -> Link<NodeType> {
        Link::new(NodeType::MiddleNode(MiddleNode::Vector(vector)))
    }
}

impl Debug for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.node)
    }
}

#[derive(Clone, Eq, PartialEq, Default)]
pub struct Matrix {
    node: Node,
}

impl IsParent for Matrix {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        self.node.get_children()
    }
}

impl IsChild for Matrix {
    fn get_parent(&self) -> BackLink<NodeType> {
        self.node.get_parent()
    }
    fn set_parent(&mut self, parent: Link<NodeType>) {
        self.node.set_parent(parent)
    }
}

impl From<Matrix> for Link<NodeType> {
    fn from(vector: Matrix) -> Link<NodeType> {
        Link::new(NodeType::MiddleNode(MiddleNode::Matrix(vector)))
    }
}

impl Debug for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.node)
    }
}
