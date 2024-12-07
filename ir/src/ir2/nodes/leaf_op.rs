use air_parser::ast::{self, Identifier, QualifiedIdentifier, TraceSegmentId};
use miden_diagnostics::SourceSpan;

use crate::ir2::{Leaf, LeafNode, Link, NodeType, TraceAccess};
use std::fmt::Debug;

/// Represents a scalar value in the [MIR]
///
/// Values are either constant, or evaluated at runtime using the context
/// provided to an AirScript program (i.e. random values, public inputs, etc.).
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MirValue {
    /// A constant value.
    Constant(ConstantValue),
    /// Following to update from the ast::BindingType enum
    /// Goal: To represent the different types of values that can be stored in the MIR
    /// (Scalar, vectors and matrices)
    ///
    /// A reference to a specific column in the trace segment, with an optional offset.
    ///
    TraceAccess(TraceAccess),
    /// A reference to a periodic column
    ///
    /// The value this corresponds to is determined by the current row of the trace.
    PeriodicColumn(PeriodicColumnAccess),
    /// A reference to a specific element of a given public input
    PublicInput(PublicInputAccess),
    /// A reference to the `random_values` array, specifically the element at the given index
    RandomValue(usize),

    /// Vector version of the above, if needed
    /// (basically the same but with a size argument to allow for continuous access)
    /// We should delete the <TraceAccess> and <RandomValue> variants if we decide to use only the most generic variants below
    TraceAccessBinding(TraceAccessBinding),
    ///RandomValueBinding is a binding to a range of random values
    RandomValueBinding(RandomValueBinding),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ConstantValue {
    Felt(u64),
    Vector(Vec<u64>),
    Matrix(Vec<Vec<u64>>),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TraceAccessBinding {
    pub segment: TraceSegmentId,
    /// The offset to the first column of the segment which is bound by this binding
    pub offset: usize,
    /// The number of columns which are bound
    pub size: usize,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RandomValueBinding {
    /// The offset in the random values array where this binding begins
    pub offset: usize,
    /// The number of elements which are bound
    pub size: usize,
}

/// Represents a typed value in the [MIR]
///
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct SpannedMirValue {
    pub span: SourceSpan,
    pub value: MirValue,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MirType {
    Felt,
    Vector(usize),
    Matrix(usize, usize),
    Definition(Vec<usize>, usize),
}

impl From<ast::Type> for MirType {
    fn from(value: ast::Type) -> Self {
        match value {
            ast::Type::Felt => MirType::Felt,
            ast::Type::Vector(n) => MirType::Vector(n),
            ast::Type::Matrix(cols, rows) => MirType::Matrix(cols, rows),
        }
    }
}

/// Represents an access of a [PeriodicColumn], similar in nature to [TraceAccess]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PeriodicColumnAccess {
    pub name: QualifiedIdentifier,
    pub cycle: usize,
}
impl PeriodicColumnAccess {
    pub const fn new(name: QualifiedIdentifier, cycle: usize) -> Self {
        Self { name, cycle }
    }
}

/// Represents an access of a [PublicInput], similar in nature to [TraceAccess]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PublicInputAccess {
    /// The name of the public input to access
    pub name: Identifier,
    /// The index of the element in the public input to access
    pub index: usize,
}
impl PublicInputAccess {
    pub const fn new(name: Identifier, index: usize) -> Self {
        Self { name, index }
    }
}

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
