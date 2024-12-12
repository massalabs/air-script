use std::ops::Deref;

use crate::ir::{Link, Op, Parameter, Value};

use super::Node;

/// The Final nodes of the MIR Graph.
/// Currently unused in the structure but will be used in the next visitor pattern implementation.
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Leaf {
    Parameter(Parameter),
    Value(Value),
    #[default]
    None,
}

impl Leaf {
    pub fn as_parameter(self) -> Option<Parameter> {
        match self {
            Leaf::Parameter(p) => Some(p),
            _ => None,
        }
    }
    pub fn as_value(self) -> Option<Value> {
        match self {
            Leaf::Value(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_op(self) -> Op {
        match self {
            Leaf::Parameter(p) => Op::Parameter(p),
            Leaf::Value(v) => Op::Value(v),
            Leaf::None => Op::None,
        }
    }
    pub fn as_node(self) -> Node {
        match self {
            Leaf::Parameter(p) => Node::Parameter(p),
            Leaf::Value(v) => Node::Value(v),
            Leaf::None => Node::None,
        }
    }
}

impl Link<Leaf> {
    pub fn as_parameter(self) -> Option<Link<Parameter>> {
        self.borrow()
            .deref()
            .clone()
            .as_parameter()
            .map(|p| p.into())
    }
    pub fn as_value(self) -> Option<Link<Value>> {
        self.borrow().deref().clone().as_value().map(|v| v.into())
    }
    pub fn as_op(self) -> Link<Op> {
        self.borrow().deref().clone().as_op().into()
    }
    pub fn as_node(self) -> Link<Node> {
        self.borrow().deref().clone().as_node().into()
    }
}
