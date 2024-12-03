mod binary_op;
mod blocks;
mod felt;
mod scope;
mod structured_op;
mod unary_op;
use crate::ir2::{BackLink, Graph, IsChild, IsNode, IsParent, Leaf, Link};
pub use binary_op::{Add, Mul, Sub};
pub use blocks::{Evaluator, For, Function, If};
pub use felt::Felt;
pub use scope::Scope;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
pub use structured_op::{Call, Fold};
pub use unary_op::{Boundary, Enf};

#[derive(Clone, Eq, PartialEq)]
pub enum RootNode {
    Graph(Graph),
}

impl IsParent for RootNode {
    fn add_child(&mut self, child: Link<NodeType>) -> Link<NodeType> {
        match self {
            RootNode::Graph(graph) => graph.add_child(child),
        }
    }
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        match self {
            RootNode::Graph(graph) => graph.get_children(),
        }
    }
}

impl IsChild for RootNode {
    fn get_parent(&self) -> BackLink<NodeType> {
        unreachable!("RootNode has no parent: {:?}", self)
    }
    fn set_parent(&mut self, _parent: Link<NodeType>) {
        unreachable!("RootNode has no parent: {:?}", self)
    }
}

impl From<RootNode> for Link<NodeType> {
    fn from(root_node: RootNode) -> Link<NodeType> {
        match root_node {
            RootNode::Graph(graph) => Link::new(NodeType::RootNode(RootNode::Graph(graph))),
        }
    }
}

impl Debug for RootNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RootNode::Graph(graph) => write!(f, "{:?}", graph),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum LeafNode {
    Value(Leaf<Felt>),
}

impl IsParent for LeafNode {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        unreachable!("LeafNode has no children: {:?}", self)
    }
}

impl IsChild for LeafNode {
    fn get_parent(&self) -> BackLink<NodeType> {
        match self {
            LeafNode::Value(leaf) => leaf.get_parent(),
        }
    }
    fn set_parent(&mut self, parent: Link<NodeType>) {
        match self {
            LeafNode::Value(leaf) => leaf.set_parent(parent),
        }
    }
}

impl From<LeafNode> for Link<NodeType> {
    fn from(leaf_node: LeafNode) -> Link<NodeType> {
        match leaf_node {
            LeafNode::Value(leaf) => leaf.into(),
        }
    }
}

impl Debug for LeafNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeafNode::Value(leaf) => write!(f, "{:?}", leaf),
        }
    }
}

#[derive(Clone, Eq, PartialEq, IsNode)]
pub enum MiddleNode {
    Add(Add),
    Sub(Sub),
    Mul(Mul),
    Scope(Scope),
    Function(Function),
    Evaluator(Evaluator),
    If(If),
    For(For),
    Fold(Fold),
    Call(Call),
    Boundary(Boundary),
    Enf(Enf),
}

#[derive(Clone, Eq, PartialEq)]
pub enum NodeType {
    RootNode(RootNode),
    LeafNode(LeafNode),
    MiddleNode(MiddleNode),
}

impl IsParent for Link<NodeType> {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        match self.borrow().deref() {
            NodeType::LeafNode(leaf_node) => leaf_node.get_children(),
            NodeType::RootNode(root_node) => root_node.get_children(),
            NodeType::MiddleNode(parent_and_child) => parent_and_child.get_children(),
        }
    }
}

impl IsChild for Link<NodeType> {
    fn get_parent(&self) -> BackLink<NodeType> {
        match self.borrow().deref() {
            NodeType::LeafNode(leaf_node) => leaf_node.get_parent(),
            NodeType::RootNode(root_node) => root_node.get_parent(),
            NodeType::MiddleNode(parent_and_child) => parent_and_child.get_parent(),
        }
    }
    fn set_parent(&mut self, parent: Link<NodeType>) {
        match self.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.set_parent(parent),
            NodeType::RootNode(root_node) => root_node.set_parent(parent),
            NodeType::MiddleNode(parent_and_child) => parent_and_child.set_parent(parent),
        }
    }
}

impl Debug for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::LeafNode(leaf_node) => write!(f, "{:?}", leaf_node),
            NodeType::RootNode(root_node) => write!(f, "{:?}", root_node),
            NodeType::MiddleNode(parent_and_child) => write!(f, "{:?}", parent_and_child),
        }
    }
}
