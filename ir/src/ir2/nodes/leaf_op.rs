use miden_diagnostics::SourceSpan;

use crate::ir::SpannedMirValue;
use crate::ir2::{Leaf, LeafNode, Link, NodeType};
use crate::MirType;
use std::fmt::Debug;

impl From<Leaf<SpannedMirValue>> for Link<NodeType> {
    fn from(value: Leaf<SpannedMirValue>) -> Link<NodeType> {
        Link::new(NodeType::LeafNode(LeafNode::Value(value)))
    }
}

impl From<SpannedMirValue> for Link<NodeType> {
    fn from(value: SpannedMirValue) -> Link<NodeType> {
        Leaf::new(value).into()
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Parameter {
    pub span: SourceSpan,
    pub ty: MirType,
    pub argument_position: usize,
}

impl Parameter {
    pub fn new(span: SourceSpan, ty: MirType, argument_position: usize) -> Self {
        Self {
            span,
            ty,
            argument_position,
        }
    }
}

impl From<Leaf<Parameter>> for Link<NodeType> {
    fn from(value: Leaf<Parameter>) -> Link<NodeType> {
        Link::new(NodeType::LeafNode(LeafNode::Parameter(value)))
    }
}

impl From<Parameter> for Link<NodeType> {
    fn from(value: Parameter) -> Link<NodeType> {
        Leaf::new(value).into()
    }
}
