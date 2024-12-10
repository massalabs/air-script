use std::marker::PhantomData;

use air_parser::ast::{self, Identifier, QualifiedIdentifier, TraceSegmentId};
use miden_diagnostics::SourceSpan;

use crate::ir3::{BackLink, Builder, Child, Link, NotSet, Op, Owner, TraceAccess};

use super::*;

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

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
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

impl MirValue {
    /*fn ty(&self) -> MirType {
        match &self {
            MirValue::Constant(c) => match c {
                ConstantValue::Felt(_) => MirType::Felt,
                ConstantValue::Vector(v) => MirType::Vector(v.len()),
                ConstantValue::Matrix(m) => MirType::Matrix(m.len(), m[0].len()),
            },
            MirValue::TraceAccess(_) => MirType::Felt,
            MirValue::PeriodicColumn(_) => MirType::Felt,
            MirValue::PublicInput(_) => MirType::Felt,
            MirValue::RandomValue(_) => MirType::Felt,
            MirValue::TraceAccessBinding(trace_access_binding) => {
                let size = trace_access_binding.size;
                match size {
                    1 => MirType::Felt,
                    _ => MirType::Vector(size),
                }
            },
            MirValue::RandomValueBinding(random_value_binding) =>  {
                let size = random_value_binding.size;
                match size {
                    1 => MirType::Felt,
                    _ => MirType::Vector(size),
                }
            },
            MirValue::Vector(vec) => {
                let size = vec.len();
                let inner_ty = vec[0].ty();
                match inner_ty {
                    MirType::Felt => MirType::Vector(size),
                    MirType::Vector(inner_size) => MirType::Matrix(size, inner_size),
                    MirType::Matrix(_, _) => unreachable!(),
                }
            },
            MirValue::Matrix(vec) => {
                let size = vec.len();
                let inner_size = vec[0].len();
                MirType::Matrix(size, inner_size)
            },
        }
    }*/
}

impl SpannedMirValue {
    /*fn ty(&self) -> MirType {
        self.value.ty()
    }*/
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

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Value {
    parent: BackLink<Owner>,
    value: SpannedMirValue,
}

impl Default for SpannedMirValue {
    fn default() -> Self {
        Self {
            value: MirValue::Constant(crate::ConstantValue::Felt(0)),
            span: Default::default(),
        }
    }
}

impl Value {
    pub fn new(value: SpannedMirValue) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
    pub fn as_leaf(self) -> Leaf {
        Leaf::Value(self)
    }
    pub fn as_op(self) -> Op {
        Op::Value(self)
    }
}

impl Link<Value> {
    pub fn as_leaf(self) -> Link<Leaf> {
        Link::new(Leaf::Value(self.borrow().clone()))
    }
    pub fn as_op(self) -> Link<Op> {
        Link::new(Op::Value(self.borrow().clone()))
    }
}

impl Child for Value {
    type Parent = Owner;
    fn get_parent(&self) -> BackLink<Self::Parent> {
        self.parent.clone()
    }
    fn set_parent(&mut self, parent: Link<Self::Parent>) {
        self.parent = parent.into();
    }
}

pub struct ValueBuilder<State> {
    _state: PhantomData<State>,
    parent: BackLink<Owner>,
    value: Option<SpannedMirValue>,
}

impl Builder for Value {
    type BuilderEmpty = ValueBuilder<(BackLink<Owner>, NotSet)>;
    type BuilderFull = ValueBuilder<(BackLink<Owner>, NotSet)>;
    fn builder() -> Self::BuilderEmpty {
        ValueBuilder::default()
    }
    fn edit(self) -> Self::BuilderFull {
        Self::BuilderFull {
            _state: PhantomData,
            parent: self.parent,
            value: Some(self.value),
        }
    }
}

impl Default for ValueBuilder<(BackLink<Owner>, NotSet)> {
    fn default() -> Self {
        Self {
            _state: PhantomData,
            parent: BackLink::default(),
            value: None,
        }
    }
}

impl ValueBuilder<(BackLink<Owner>, NotSet)> {
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn value(
        mut self,
        value: SpannedMirValue,
    ) -> ValueBuilder<(BackLink<Owner>, SpannedMirValue)> {
        self.value = Some(value);
        unsafe { std::mem::transmute(self) }
    }
}

impl ValueBuilder<(BackLink<Owner>, SpannedMirValue)> {
    pub fn value(mut self, value: SpannedMirValue) -> Self {
        self.value = Some(value);
        self
    }
    pub fn parent(mut self, parent: Link<Owner>) -> Self {
        self.parent = parent.into();
        self
    }
    pub fn build(self) -> Value {
        Value {
            parent: self.parent,
            value: self.value.expect("value not set"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir3::{Add, Owner};

    #[test]
    fn test_value_builder() {
        let parent = Link::new(Owner::Add(Add::default()));
        let value = Value::builder()
            .parent(parent.clone())
            .value(SpannedMirValue::default())
            .build();
        assert_eq!(Link::from(value.parent), parent.clone());
        assert_eq!(value.value, SpannedMirValue::default());
    }
}
