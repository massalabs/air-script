use crate::ir::{
    get_inner, get_inner_mut, Accessor, Add, BackLink, Boundary, BusOp, Call, Child, ConstantValue,
    Enf, Exp, Fold, For, If, Link, Matrix, MirValue, Mul, Node, Owner, Parameter, Parent,
    Singleton, SpannedMirValue, Sub, Value, Vector,
};
use miden_diagnostics::{SourceSpan, Spanned};

use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut},
};

/// The combined [Op]s and leaves of the MIR Graph
/// These represent the operations that can be present in [Root] bodies
/// The [Op] enum owns it's inner struct to allow conversion between variants
#[derive(Clone, PartialEq, Eq, Debug, Hash, Spanned)]
pub enum Op {
    Enf(Enf),
    Boundary(Boundary),
    Add(Add),
    Sub(Sub),
    Mul(Mul),
    Exp(Exp),
    If(If),
    For(For),
    Call(Call),
    Fold(Fold),
    Vector(Vector),
    Matrix(Matrix),
    Accessor(Accessor),
    BusOp(BusOp),
    Parameter(Parameter),
    Value(Value),
    None(SourceSpan),
}

impl Op {
    pub fn is_parent(&self) -> bool {
        !matches!(self, Op::Value(_) | Op::Parameter(_) | Op::None(_))
    }

    pub fn is_child(&self) -> bool {
        !matches!(self, Op::None(_))
    }
}

impl Default for Op {
    fn default() -> Self {
        Op::None(Default::default())
    }
}

impl Parent for Op {
    type Child = Op;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        match self {
            Op::Enf(e) => e.children(),
            Op::Boundary(b) => b.children(),
            Op::Add(a) => a.children(),
            Op::Sub(s) => s.children(),
            Op::Mul(m) => m.children(),
            Op::Exp(e) => e.children(),
            Op::If(i) => i.children(),
            Op::For(f) => f.children(),
            Op::Call(c) => c.children(),
            Op::Fold(f) => f.children(),
            Op::Vector(v) => v.children(),
            Op::Matrix(m) => m.children(),
            Op::Accessor(a) => a.children(),
            Op::BusOp(b) => b.children(),
            Op::Parameter(_) => Link::default(),
            Op::Value(_) => Link::default(),
            Op::None(_) => Link::default(),
        }
    }
}

impl Child for Op {
    type Parent = Owner;
    fn get_parents(&self) -> Vec<BackLink<Self::Parent>> {
        match self {
            Op::Enf(e) => e.get_parents(),
            Op::Boundary(b) => b.get_parents(),
            Op::Add(a) => a.get_parents(),
            Op::Sub(s) => s.get_parents(),
            Op::Mul(m) => m.get_parents(),
            Op::Exp(e) => e.get_parents(),
            Op::If(i) => i.get_parents(),
            Op::For(f) => f.get_parents(),
            Op::Call(c) => c.get_parents(),
            Op::Fold(f) => f.get_parents(),
            Op::Vector(v) => v.get_parents(),
            Op::Matrix(m) => m.get_parents(),
            Op::Accessor(a) => a.get_parents(),
            Op::BusOp(b) => b.get_parents(),
            Op::Parameter(p) => p.get_parents(),
            Op::Value(v) => v.get_parents(),
            Op::None(_) => Default::default(),
        }
    }
    fn add_parent(&mut self, parent: Link<Self::Parent>) {
        match self {
            Op::Enf(e) => e.add_parent(parent),
            Op::Boundary(b) => b.add_parent(parent),
            Op::Add(a) => a.add_parent(parent),
            Op::Sub(s) => s.add_parent(parent),
            Op::Mul(m) => m.add_parent(parent),
            Op::Exp(e) => e.add_parent(parent),
            Op::If(i) => i.add_parent(parent),
            Op::For(f) => f.add_parent(parent),
            Op::Call(c) => c.add_parent(parent),
            Op::Fold(f) => f.add_parent(parent),
            Op::Vector(v) => v.add_parent(parent),
            Op::Matrix(m) => m.add_parent(parent),
            Op::Accessor(a) => a.add_parent(parent),
            Op::BusOp(b) => b.add_parent(parent),
            Op::Parameter(p) => p.add_parent(parent),
            Op::Value(v) => v.add_parent(parent),
            Op::None(_) => {}
        }
    }
    fn remove_parent(&mut self, parent: Link<Self::Parent>) {
        match self {
            Op::Enf(e) => e.remove_parent(parent),
            Op::Boundary(b) => b.remove_parent(parent),
            Op::Add(a) => a.remove_parent(parent),
            Op::Sub(s) => s.remove_parent(parent),
            Op::Mul(m) => m.remove_parent(parent),
            Op::Exp(e) => e.remove_parent(parent),
            Op::If(i) => i.remove_parent(parent),
            Op::For(f) => f.remove_parent(parent),
            Op::Call(c) => c.remove_parent(parent),
            Op::Fold(f) => f.remove_parent(parent),
            Op::Vector(v) => v.remove_parent(parent),
            Op::Matrix(m) => m.remove_parent(parent),
            Op::Accessor(a) => a.remove_parent(parent),
            Op::BusOp(b) => b.remove_parent(parent),
            Op::Parameter(p) => p.remove_parent(parent),
            Op::Value(v) => v.remove_parent(parent),
            Op::None(_) => {}
        }
    }
}

impl Link<Op> {
    /// Debug the current [Op], showing [std::cell::RefCell]'s `@{pointer}` and inner struct.
    /// This is useful to debug shared mutability issues.
    pub fn debug(&self) -> String {
        match self.borrow().deref() {
            Op::Enf(e) => format!("Op::Enf@{}({:#?})", self.get_ptr(), e),
            Op::Boundary(b) => format!("Op::Boundary@{}({:#?})", self.get_ptr(), b),
            Op::Add(a) => format!("Op::Add@{}({:#?})", self.get_ptr(), a),
            Op::Sub(s) => format!("Op::Sub@{}({:#?})", self.get_ptr(), s),
            Op::Mul(m) => format!("Op::Mul@{}({:#?})", self.get_ptr(), m),
            Op::Exp(e) => format!("Op::Exp@{}({:#?})", self.get_ptr(), e),
            Op::If(i) => format!("Op::If@{}({:#?})", self.get_ptr(), i),
            Op::For(f) => format!("Op::For@{}({:#?})", self.get_ptr(), f),
            Op::Call(c) => format!("Op::Call@{}({:#?})", self.get_ptr(), c),
            Op::Fold(f) => format!("Op::Fold@{}({:#?})", self.get_ptr(), f),
            Op::Vector(v) => format!("Op::Vector@{}({:#?})", self.get_ptr(), v),
            Op::Matrix(m) => format!("Op::Matrix@{}({:#?})", self.get_ptr(), m),
            Op::Accessor(a) => format!("Op::Accessor@{}({:#?})", self.get_ptr(), a),
            Op::BusOp(b) => format!("Op::BusOp@{}({:#?})", self.get_ptr(), b),
            Op::Parameter(p) => format!("Op::Parameter@{}({:#?})", self.get_ptr(), p),
            Op::Value(v) => format!("Op::Value@{}({:#?})", self.get_ptr(), v),
            Op::None(_) => "Op::None".to_string(),
        }
    }
    /// Update the current [Op] with the other [Op].
    /// Also updates all instances of [Node] and [Owner] wrappers,
    /// setting them to the new variant.
    #[track_caller]
    pub fn set(&self, other: &Link<Op>) {
        #[cfg(feature = "debug_link")]
        eprintln!("Updating Op: {} -> {}", self.debug(), other.debug());
        self.as_node().update(&other.as_node());
        if let (Some(self_owner), Some(other_owner)) = (self.as_owner(), other.as_owner()) {
            self_owner.update(&other_owner);
        }
        self.update(other);
    }

    /// Get the current [Op]'s [Node] variant,
    /// creating a new [Node] if it doesn't exist, re-using it as a singleton otherwise.
    pub fn as_node(&self) -> Link<Node> {
        match self.clone().borrow().deref() {
            Op::Enf(Enf {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::Boundary(Boundary {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::Add(Add {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::Sub(Sub {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::Mul(Mul {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::Exp(Exp {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::If(If {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::For(For {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::Call(Call {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::Fold(Fold {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::Vector(Vector {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::Matrix(Matrix {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::Accessor(Accessor {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::BusOp(BusOp {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::Parameter(Parameter {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::Value(Value {
                _node: Singleton(Some(link)),
                ..
            }) => link.clone(),
            Op::None(span) => Node::None(*span).into(),
            _ => {
                unreachable!("Singleton node not found for Op: {}", self.debug())
            }
        }
    }
    /// Try getting the current [Op]'s [Owner] variant,
    /// creating a new [Owner] if it doesn't exist, re-using it as a singleton otherwise.
    pub fn as_owner(&self) -> Option<Link<Owner>> {
        match self.clone().borrow().deref() {
            Op::Enf(Enf {
                _owner: Singleton(Some(link)),
                ..
            }) => Some(link.clone()),
            Op::Boundary(Boundary {
                _owner: Singleton(Some(link)),
                ..
            }) => Some(link.clone()),
            Op::Add(Add {
                _owner: Singleton(Some(link)),
                ..
            }) => Some(link.clone()),
            Op::Sub(Sub {
                _owner: Singleton(Some(link)),
                ..
            }) => Some(link.clone()),
            Op::Mul(Mul {
                _owner: Singleton(Some(link)),
                ..
            }) => Some(link.clone()),
            Op::Exp(Exp {
                _owner: Singleton(Some(link)),
                ..
            }) => Some(link.clone()),
            Op::If(If {
                _owner: Singleton(Some(link)),
                ..
            }) => Some(link.clone()),
            Op::For(For {
                _owner: Singleton(Some(link)),
                ..
            }) => Some(link.clone()),
            Op::Call(Call {
                _owner: Singleton(Some(link)),
                ..
            }) => Some(link.clone()),
            Op::Fold(Fold {
                _owner: Singleton(Some(link)),
                ..
            }) => Some(link.clone()),
            Op::Vector(Vector {
                _owner: Singleton(Some(link)),
                ..
            }) => Some(link.clone()),
            Op::Matrix(Matrix {
                _owner: Singleton(Some(link)),
                ..
            }) => Some(link.clone()),
            Op::Accessor(Accessor {
                _owner: Singleton(Some(link)),
                ..
            }) => Some(link.clone()),
            Op::BusOp(BusOp {
                _owner: Singleton(Some(link)),
                ..
            }) => Some(link.clone()),
            Op::Parameter(_) => None,
            Op::Value(_) => None,
            Op::None(_) => None,
            _ => {
                unreachable!("Singleton owner not found for Op: {}", self.debug())
            }
        }
    }
    /// Try getting the current [Op]'s inner [Enf].
    /// Returns None if the current [Op] is not an [Enf] or the Rc count is zero.
    pub fn as_enf(&self) -> Option<Ref<Enf>> {
        get_inner(self.borrow(), |op| match op {
            Op::Enf(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Enf], borrowing mutably.
    /// Returns None if the current [Op] is not an [Enf] or the Rc count is zero.
    pub fn as_enf_mut(&self) -> Option<RefMut<Enf>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Enf(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Boundary].
    /// Returns None if the current [Op] is not a [Boundary] or the Rc count is zero.
    pub fn as_boundary(&self) -> Option<Ref<Boundary>> {
        get_inner(self.borrow(), |op| match op {
            Op::Boundary(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Boundary], borrowing mutably.
    /// Returns None if the current [Op] is not a [Boundary] or the Rc count is zero.
    pub fn as_boundary_mut(&self) -> Option<RefMut<Boundary>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Boundary(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Add].
    /// Returns None if the current [Op] is not an [Add] or the Rc count is zero.
    pub fn as_add(&self) -> Option<Ref<Add>> {
        get_inner(self.borrow(), |op| match op {
            Op::Add(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Add], borrowing mutably.
    /// Returns None if the current [Op] is not an [Add] or the Rc count is zero.
    pub fn as_add_mut(&self) -> Option<RefMut<Add>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Add(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Sub].
    /// Returns None if the current [Op] is not a [Sub] or the Rc count is zero.
    pub fn as_sub(&self) -> Option<Ref<Sub>> {
        get_inner(self.borrow(), |op| match op {
            Op::Sub(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Sub], borrowing mutably.
    /// Returns None if the current [Op] is not a [Sub] or the Rc count is zero.
    pub fn as_sub_mut(&self) -> Option<RefMut<Sub>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Sub(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Mul].
    /// Returns None if the current [Op] is not a [Mul] or the Rc count is zero.
    pub fn as_mul(&self) -> Option<Ref<Mul>> {
        get_inner(self.borrow(), |op| match op {
            Op::Mul(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Mul], borrowing mutably.
    /// Returns None if the current [Op] is not a [Mul] or the Rc count is zero.
    pub fn as_mul_mut(&self) -> Option<RefMut<Mul>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Mul(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Exp].
    /// Returns None if the current [Op] is not an [Exp] or the Rc count is zero.
    pub fn as_exp(&self) -> Option<Ref<Exp>> {
        get_inner(self.borrow(), |op| match op {
            Op::Exp(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Exp], borrowing mutably.
    /// Returns None if the current [Op] is not an [Exp] or the Rc count is zero.
    pub fn as_exp_mut(&self) -> Option<RefMut<Exp>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Exp(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [If].
    /// Returns None if the current [Op] is not an [If] or the Rc count is zero.
    pub fn as_if(&self) -> Option<Ref<If>> {
        get_inner(self.borrow(), |op| match op {
            Op::If(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [If], borrowing mutably.
    /// Returns None if the current [Op] is not an [If] or the Rc count is zero.
    pub fn as_if_mut(&self) -> Option<RefMut<If>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::If(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [For].
    /// Returns None if the current [Op] is not a [For] or the Rc count is zero.
    pub fn as_for(&self) -> Option<Ref<For>> {
        get_inner(self.borrow(), |op| match op {
            Op::For(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [For], borrowing mutably.
    /// Returns None if the current [Op] is not a [For] or the Rc count is zero.
    pub fn as_for_mut(&self) -> Option<RefMut<For>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::For(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Call].
    /// Returns None if the current [Op] is not a [Call] or the Rc count is zero.
    pub fn as_call(&self) -> Option<Ref<Call>> {
        get_inner(self.borrow(), |op| match op {
            Op::Call(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Call], borrowing mutably.
    /// Returns None if the current [Op] is not a [Call] or the Rc count is zero.
    pub fn as_call_mut(&self) -> Option<RefMut<Call>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Call(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Fold].
    /// Returns None if the current [Op] is not a [Fold] or the Rc count is zero.
    pub fn as_fold(&self) -> Option<Ref<Fold>> {
        get_inner(self.borrow(), |op| match op {
            Op::Fold(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Fold], borrowing mutably.
    /// Returns None if the current [Op] is not a [Fold] or the Rc count is zero.
    pub fn as_fold_mut(&self) -> Option<RefMut<Fold>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Fold(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Vector].
    /// Returns None if the current [Op] is not a [Vector] or the Rc count is zero.
    pub fn as_vector(&self) -> Option<Ref<Vector>> {
        get_inner(self.borrow(), |op| match op {
            Op::Vector(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Vector], borrowing mutably.
    /// Returns None if the current [Op] is not a [Vector] or the Rc count is zero.
    pub fn as_vector_mut(&self) -> Option<RefMut<Vector>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Vector(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Matrix].
    /// Returns None if the current [Op] is not a [Matrix] or the Rc count is zero.
    pub fn as_matrix(&self) -> Option<Ref<Matrix>> {
        get_inner(self.borrow(), |op| match op {
            Op::Matrix(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Matrix], borrowing mutably.
    /// Returns None if the current [Op] is not a [Matrix] or the Rc count is zero.
    pub fn as_matrix_mut(&self) -> Option<RefMut<Matrix>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Matrix(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Accessor].
    /// Returns None if the current [Op] is not an [Accessor] or the Rc count is zero.
    pub fn as_accessor(&self) -> Option<Ref<Accessor>> {
        get_inner(self.borrow(), |op| match op {
            Op::Accessor(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Accessor], borrowing mutably.
    /// Returns None if the current [Op] is not an [Accessor] or the Rc count is zero.
    pub fn as_accessor_mut(&self) -> Option<RefMut<Accessor>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Accessor(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [BusOp].
    /// Returns None if the current [Op] is not a [BusOp] or the Rc count is zero.
    pub fn as_bus_op(&self) -> Option<Ref<BusOp>> {
        get_inner(self.borrow(), |op| match op {
            Op::BusOp(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [BusOp], borrowing mutably.
    /// Returns None if the current [Op] is not a [BusOp] or the Rc count is zero.
    pub fn as_bus_op_mut(&self) -> Option<RefMut<BusOp>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::BusOp(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Parameter].
    /// Returns None if the current [Op] is not a [Parameter] or the Rc count is zero.
    pub fn as_parameter(&self) -> Option<Ref<Parameter>> {
        get_inner(self.borrow(), |op| match op {
            Op::Parameter(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Parameter], borrowing mutably.
    /// Returns None if the current [Op] is not a [Parameter] or the Rc count is zero.
    pub fn as_parameter_mut(&self) -> Option<RefMut<Parameter>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Parameter(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Value].
    /// Returns None if the current [Op] is not a [Value] or the Rc count is zero.
    pub fn as_value(&self) -> Option<Ref<Value>> {
        get_inner(self.borrow(), |op| match op {
            Op::Value(inner) => Some(inner),
            _ => None,
        })
    }
    /// Try getting the current [Op]'s inner [Value], borrowing mutably.
    /// Returns None if the current [Op] is not a [Value] or the Rc count is zero.
    pub fn as_value_mut(&self) -> Option<RefMut<Value>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Value(inner) => Some(inner),
            _ => None,
        })
    }

    pub fn is_parent(&self) -> bool {
        self.borrow().is_parent()
    }
    pub fn is_child(&self) -> bool {
        self.borrow().is_child()
    }
}

impl From<u64> for Link<Op> {
    fn from(value: u64) -> Self {
        Value::create(SpannedMirValue {
            value: MirValue::Constant(ConstantValue::Felt(value)),
            ..Default::default()
        })
    }
}

impl From<i64> for Link<Op> {
    fn from(value: i64) -> Self {
        Self::from(value as u64)
    }
}

impl From<i32> for Link<Op> {
    fn from(value: i32) -> Self {
        Self::from(value as u64)
    }
}

impl From<Link<Node>> for Link<Op> {
    fn from(node: Link<Node>) -> Self {
        node.as_op()
            .unwrap_or_else(|| panic!("{:?} is not an op", node))
    }
}

impl From<Link<Owner>> for Link<Op> {
    fn from(owner: Link<Owner>) -> Self {
        owner
            .as_op()
            .unwrap_or_else(|| panic!("{:?} is not an op", owner))
    }
}
