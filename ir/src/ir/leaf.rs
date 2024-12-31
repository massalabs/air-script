use std::ops::Deref;

use crate::ir::{BackLink, Child, Link, Op, Owner, Parameter, Value};

use super::Node;

/// The Final nodes of the MIR Graph.
/// Currently unused in the structure but will be used in the next visitor pattern implementation.
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Leaf {
    Parameter(Link<Parameter>),
    Value(Link<Value>),
    #[default]
    None,
}

impl Child for Leaf {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        match self {
            Leaf::Parameter(p) => p.get_parent(),
            Leaf::Value(v) => v.get_parent(),
            Leaf::None => BackLink::default(),
        }
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        match self {
            Leaf::Parameter(p) => p.set_parent(parent),
            Leaf::Value(v) => v.set_parent(parent),
            Leaf::None => (),
        }
    }
}

impl Link<Leaf> {
    pub fn as_parameter(self) -> Option<Link<Parameter>> {
        match self.borrow().deref() {
            Leaf::Parameter(p) => Some(p.clone()),
            _ => None,
        }
    }
    pub fn as_value(self) -> Option<Link<Value>> {
        match self.borrow().deref() {
            Leaf::Value(v) => Some(v.clone()),
            _ => None,
        }
    }
    pub fn as_op(self) -> Link<Op> {
        match self.borrow().deref() {
            Leaf::Parameter(p) => Op::Parameter(p.clone()).into(),
            Leaf::Value(v) => Op::Value(v.clone()).into(),
            Leaf::None => Op::None.into(),
        }
    }
    pub fn as_node(self) -> Link<Node> {
        match self.borrow().deref() {
            Leaf::Parameter(p) => Node::Parameter(p.clone()).into(),
            Leaf::Value(v) => Node::Value(v.clone()).into(),
            Leaf::None => Node::None.into(),
        }
    }
}
