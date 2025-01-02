use std::marker::PhantomData;

use air_parser::ast::{self, Identifier, QualifiedIdentifier, TraceSegmentId};
use miden_diagnostics::SourceSpan;

use crate::ir::{BackLink, Builder, Child, Leaf, Link, Node, NotSet, Op, Owner, TraceAccess};

/// Represents a scalar value in the [MIR]
///
/// Values are either constant, or evaluated at runtime using the context
/// provided to an AirScript program (i.e. random values, public inputs, etc.).
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
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

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum ConstantValue {
    Felt(u64),
    Vector(Vec<u64>),
    Matrix(Vec<Vec<u64>>),
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct TraceAccessBinding {
    pub segment: TraceSegmentId,
    /// The offset to the first column of the segment which is bound by this binding
    pub offset: usize,
    /// The number of columns which are bound
    pub size: usize,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct RandomValueBinding {
    /// The offset in the random values array where this binding begins
    pub offset: usize,
    /// The number of elements which are bound
    pub size: usize,
}

/// Represents a typed value in the [MIR]
///
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct SpannedMirValue {
    pub span: SourceSpan,
    pub value: MirValue,
}

#[derive(Debug, Default, Eq, PartialEq, Clone, Hash)]
pub enum MirType {
    #[default]
    Felt,
    Vector(usize),
    Matrix(usize, usize),
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

impl Default for SpannedMirValue {
    fn default() -> Self {
        Self {
            value: MirValue::Constant(ConstantValue::Felt(0)),
            span: Default::default(),
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Value {
    pub parents: Vec<BackLink<Owner>>,
    pub value: SpannedMirValue,
}

impl Value {
    pub fn create(value: SpannedMirValue) -> Link<Self> {
        Self {
            value,
            ..Default::default()
        }
        .into()
    }
}

impl Link<Value> {
    pub fn as_leaf(self) -> Link<Leaf> {
        Leaf::Value(self).into()
    }
    pub fn as_op(self) -> Link<Op> {
        Op::Value(self).into()
    }
    pub fn as_node(self) -> Link<Node> {
        Node::Value(self).into()
    }
}

impl Child for Value {
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

pub struct ValueBuilder<State> {
    _state: PhantomData<State>,
    parents: Vec<BackLink<Owner>>,
    value: Option<SpannedMirValue>,
}

type ValueBuilderEmpty = ValueBuilder<(BackLink<Owner>, NotSet)>;
type ValueBuilderFull = ValueBuilder<(BackLink<Owner>, SpannedMirValue)>;

impl Builder for Value {
    type BuilderEmpty = ValueBuilderEmpty;
    type BuilderFull = ValueBuilderFull;
    fn builder() -> Self::BuilderEmpty {
        ValueBuilder::default()
    }
}

impl Default for ValueBuilderEmpty {
    fn default() -> Self {
        Self {
            _state: PhantomData,
            parents: Vec::default(),
            value: None,
        }
    }
}

impl ValueBuilderEmpty {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
        self
    }
    pub fn value(mut self, value: SpannedMirValue) -> ValueBuilderFull {
        self.value = Some(value);
        unsafe { std::mem::transmute(self) }
    }
}

impl ValueBuilderFull {
    pub fn value(mut self, value: SpannedMirValue) -> Self {
        self.value = Some(value);
        self
    }
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parents.push(parent.into());
        self
    }
    pub fn build(self) -> Link<Value> {
        Value {
            parents: self.parents,
            value: self.value.expect("value not set"),
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;
    use crate::ir::{Add, Owner};

    #[test]
    fn test_value_builder() {
        let parent = Link::new(Owner::Add(Add::default().into()));
        let value = Value::builder()
            .parent(parent.clone())
            .value(SpannedMirValue::default())
            .build();
        assert_eq!(
            value.borrow().deref(),
            &Value {
                parents: vec![parent.into()],
                value: SpannedMirValue::default()
            }
        );
    }
}
