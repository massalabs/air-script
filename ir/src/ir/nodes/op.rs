use crate::ir::{
    get_inner, get_inner_mut, Accessor, Add, BackLink, Boundary, Call, Child, Enf, Fold, For, If,
    Link, Matrix, Mul, Node, Owner, Parameter, Parent, Sub, Value, Vector,
};

use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut},
};

/// The combined Operators and Leaves of the MIR Graph
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Op {
    Enf(Enf),
    Boundary(Boundary),
    Add(Add),
    Sub(Sub),
    Mul(Mul),
    If(If),
    For(For),
    Call(Call),
    Fold(Fold),
    Vector(Vector),
    Matrix(Matrix),
    Accessor(Accessor),
    Parameter(Parameter),
    Value(Value),
    #[default]
    None,
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
            Op::If(i) => i.children(),
            Op::For(f) => f.children(),
            Op::Call(c) => c.children(),
            Op::Fold(f) => f.children(),
            Op::Vector(v) => v.children(),
            Op::Matrix(m) => m.children(),
            Op::Accessor(a) => a.children(),
            Op::Parameter(_) => Link::default(),
            Op::Value(_) => Link::default(),
            Op::None => Link::default(),
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
            Op::If(i) => i.get_parents(),
            Op::For(f) => f.get_parents(),
            Op::Call(c) => c.get_parents(),
            Op::Fold(f) => f.get_parents(),
            Op::Vector(v) => v.get_parents(),
            Op::Matrix(m) => m.get_parents(),
            Op::Accessor(a) => a.get_parents(),
            Op::Parameter(p) => p.get_parents(),
            Op::Value(v) => v.get_parents(),
            Op::None => Default::default(),
        }
    }
    fn add_parent(&mut self, parent: Link<Self::Parent>) {
        match self {
            Op::Enf(e) => e.add_parent(parent),
            Op::Boundary(b) => b.add_parent(parent),
            Op::Add(a) => a.add_parent(parent),
            Op::Sub(s) => s.add_parent(parent),
            Op::Mul(m) => m.add_parent(parent),
            Op::If(i) => i.add_parent(parent),
            Op::For(f) => f.add_parent(parent),
            Op::Call(c) => c.add_parent(parent),
            Op::Fold(f) => f.add_parent(parent),
            Op::Vector(v) => v.add_parent(parent),
            Op::Matrix(m) => m.add_parent(parent),
            Op::Accessor(a) => a.add_parent(parent),
            Op::Parameter(p) => p.add_parent(parent),
            Op::Value(v) => v.add_parent(parent),
            Op::None => {}
        }
    }
    fn remove_parent(&mut self, parent: Link<Self::Parent>) {
        match self {
            Op::Enf(e) => e.remove_parent(parent),
            Op::Boundary(b) => b.remove_parent(parent),
            Op::Add(a) => a.remove_parent(parent),
            Op::Sub(s) => s.remove_parent(parent),
            Op::Mul(m) => m.remove_parent(parent),
            Op::If(i) => i.remove_parent(parent),
            Op::For(f) => f.remove_parent(parent),
            Op::Call(c) => c.remove_parent(parent),
            Op::Fold(f) => f.remove_parent(parent),
            Op::Vector(v) => v.remove_parent(parent),
            Op::Matrix(m) => m.remove_parent(parent),
            Op::Accessor(a) => a.remove_parent(parent),
            Op::Parameter(p) => p.remove_parent(parent),
            Op::Value(v) => v.remove_parent(parent),
            Op::None => {}
        }
    }
}

impl Link<Op> {
    pub fn debug(&self) -> String {
        match self.borrow().deref() {
            Op::Enf(e) => format!("Op::Enf@{}({:#?})", self.get_ptr(), e),
            Op::Boundary(b) => format!("Op::Boundary@{}({:#?})", self.get_ptr(), b),
            Op::Add(a) => format!("Op::Add@{}({:#?})", self.get_ptr(), a),
            Op::Sub(s) => format!("Op::Sub@{}({:#?})", self.get_ptr(), s),
            Op::Mul(m) => format!("Op::Mul@{}({:#?})", self.get_ptr(), m),
            Op::If(i) => format!("Op::If@{}({:#?})", self.get_ptr(), i),
            Op::For(f) => format!("Op::For@{}({:#?})", self.get_ptr(), f),
            Op::Call(c) => format!("Op::Call@{}({:#?})", self.get_ptr(), c),
            Op::Fold(f) => format!("Op::Fold@{}({:#?})", self.get_ptr(), f),
            Op::Vector(v) => format!("Op::Vector@{}({:#?})", self.get_ptr(), v),
            Op::Matrix(m) => format!("Op::Matrix@{}({:#?})", self.get_ptr(), m),
            Op::Accessor(a) => format!("Op::Accessor@{}({:#?})", self.get_ptr(), a),
            Op::Parameter(p) => format!("Op::Parameter@{}({:#?})", self.get_ptr(), p),
            Op::Value(v) => format!("Op::Value@{}({:#?})", self.get_ptr(), v),
            Op::None => "Op::None".to_string(),
        }
    }
    pub fn set(&self, other: &Link<Op>) {
        self.as_node().update(&other.as_node());
        if let Some(owner) = self.as_owner() {
            if let Some(other_owner) = other.as_owner() {
                owner.update(&other_owner);
            }
        }
        self.update(other);
    }
    pub fn as_node(&self) -> Link<Node> {
        let back: BackLink<Op> = self.clone().into();
        match self.clone().borrow_mut().deref_mut() {
            Op::Enf(Enf {
                _node: Some(link), ..
            }) => link.clone(),
            Op::Enf(ref mut enf) => {
                let node: Link<Node> = Node::Enf(back).into();
                enf._node = Some(node.clone());
                node
            }
            Op::Boundary(Boundary {
                _node: Some(link), ..
            }) => link.clone(),
            Op::Boundary(ref mut boundary) => {
                let node: Link<Node> = Node::Boundary(back).into();
                boundary._node = Some(node.clone());
                node
            }
            Op::Add(Add {
                _node: Some(link), ..
            }) => link.clone(),
            Op::Add(ref mut add) => {
                let node: Link<Node> = Node::Add(back).into();
                add._node = Some(node.clone());
                node
            }
            Op::Sub(Sub {
                _node: Some(link), ..
            }) => link.clone(),
            Op::Sub(ref mut sub) => {
                let node: Link<Node> = Node::Sub(back).into();
                sub._node = Some(node.clone());
                node
            }
            Op::Mul(Mul {
                _node: Some(link), ..
            }) => link.clone(),
            Op::Mul(ref mut mul) => {
                let node: Link<Node> = Node::Mul(back).into();
                mul._node = Some(node.clone());
                node
            }
            Op::If(If {
                _node: Some(link), ..
            }) => link.clone(),
            Op::If(ref mut if_op) => {
                let node: Link<Node> = Node::If(back).into();
                if_op._node = Some(node.clone());
                node
            }
            Op::For(For {
                _node: Some(link), ..
            }) => link.clone(),
            Op::For(ref mut for_op) => {
                let node: Link<Node> = Node::For(back).into();
                for_op._node = Some(node.clone());
                node
            }
            Op::Call(Call {
                _node: Some(link), ..
            }) => link.clone(),
            Op::Call(ref mut call) => {
                let node: Link<Node> = Node::Call(back).into();
                call._node = Some(node.clone());
                node
            }
            Op::Fold(Fold {
                _node: Some(link), ..
            }) => link.clone(),
            Op::Fold(ref mut fold) => {
                let node: Link<Node> = Node::Fold(back).into();
                fold._node = Some(node.clone());
                node
            }
            Op::Vector(Vector {
                _node: Some(link), ..
            }) => link.clone(),
            Op::Vector(ref mut vector) => {
                let node: Link<Node> = Node::Vector(back).into();
                vector._node = Some(node.clone());
                node
            }
            Op::Matrix(Matrix {
                _node: Some(link), ..
            }) => link.clone(),
            Op::Matrix(ref mut matrix) => {
                let node: Link<Node> = Node::Matrix(back).into();
                matrix._node = Some(node.clone());
                node
            }
            Op::Accessor(Accessor {
                _node: Some(link), ..
            }) => link.clone(),
            Op::Accessor(ref mut accessor) => {
                let node: Link<Node> = Node::Accessor(back).into();
                accessor._node = Some(node.clone());
                node
            }
            Op::Parameter(Parameter {
                _node: Some(link), ..
            }) => link.clone(),
            Op::Parameter(ref mut parameter) => {
                let node: Link<Node> = Node::Parameter(back).into();
                parameter._node = Some(node.clone());
                node
            }
            Op::Value(Value {
                _node: Some(link), ..
            }) => link.clone(),
            Op::Value(ref mut value) => {
                let node: Link<Node> = Node::Value(back).into();
                value._node = Some(node.clone());
                node
            }
            Op::None => Node::None.into(),
        }
    }
    pub fn as_owner(&self) -> Option<Link<Owner>> {
        let back: BackLink<Op> = self.clone().into();
        match self.clone().borrow_mut().deref_mut() {
            Op::Enf(Enf {
                _owner: Some(link), ..
            }) => Some(link.clone()),
            Op::Enf(ref mut enf) => {
                let owner: Link<Owner> = Owner::Enf(back).into();
                enf._owner = Some(owner.clone());
                enf._owner.clone()
            }
            Op::Boundary(Boundary {
                _owner: Some(link), ..
            }) => Some(link.clone()),
            Op::Boundary(ref mut boundary) => {
                let owner: Link<Owner> = Owner::Boundary(back).into();
                boundary._owner = Some(owner.clone());
                boundary._owner.clone()
            }
            Op::Add(Add {
                _owner: Some(link), ..
            }) => Some(link.clone()),
            Op::Add(ref mut add) => {
                let owner: Link<Owner> = Owner::Add(back).into();
                add._owner = Some(owner.clone());
                add._owner.clone()
            }
            Op::Sub(Sub {
                _owner: Some(link), ..
            }) => Some(link.clone()),
            Op::Sub(ref mut sub) => {
                let owner: Link<Owner> = Owner::Sub(back).into();
                sub._owner = Some(owner.clone());
                sub._owner.clone()
            }
            Op::Mul(Mul {
                _owner: Some(link), ..
            }) => Some(link.clone()),
            Op::Mul(ref mut mul) => {
                let owner: Link<Owner> = Owner::Mul(back).into();
                mul._owner = Some(owner.clone());
                mul._owner.clone()
            }
            Op::If(If {
                _owner: Some(link), ..
            }) => Some(link.clone()),
            Op::If(ref mut if_op) => {
                let owner: Link<Owner> = Owner::If(back).into();
                if_op._owner = Some(owner.clone());
                if_op._owner.clone()
            }
            Op::For(For {
                _owner: Some(link), ..
            }) => Some(link.clone()),
            Op::For(ref mut for_op) => {
                let owner: Link<Owner> = Owner::For(back).into();
                for_op._owner = Some(owner.clone());
                for_op._owner.clone()
            }
            Op::Call(Call {
                _owner: Some(link), ..
            }) => Some(link.clone()),
            Op::Call(ref mut call) => {
                let owner: Link<Owner> = Owner::Call(back).into();
                call._owner = Some(owner.clone());
                call._owner.clone()
            }
            Op::Fold(Fold {
                _owner: Some(link), ..
            }) => Some(link.clone()),
            Op::Fold(ref mut fold) => {
                let owner: Link<Owner> = Owner::Fold(back).into();
                fold._owner = Some(owner.clone());
                fold._owner.clone()
            }
            Op::Vector(Vector {
                _owner: Some(link), ..
            }) => Some(link.clone()),
            Op::Vector(ref mut vector) => {
                let owner: Link<Owner> = Owner::Vector(back).into();
                vector._owner = Some(owner.clone());
                vector._owner.clone()
            }
            Op::Matrix(Matrix {
                _owner: Some(link), ..
            }) => Some(link.clone()),
            Op::Matrix(ref mut matrix) => {
                let owner: Link<Owner> = Owner::Matrix(back).into();
                matrix._owner = Some(owner.clone());
                matrix._owner.clone()
            }
            Op::Accessor(Accessor {
                _owner: Some(link), ..
            }) => Some(link.clone()),
            Op::Accessor(ref mut accessor) => {
                let owner: Link<Owner> = Owner::Accessor(back).into();
                accessor._owner = Some(owner.clone());
                accessor._owner.clone()
            }
            Op::Parameter(_) => None,
            Op::Value(_) => None,
            Op::None => None,
        }
    }
    pub fn as_enf(&self) -> Option<Ref<Enf>> {
        get_inner(self.borrow(), |op| match op {
            Op::Enf(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_enf_mut(&self) -> Option<RefMut<Enf>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Enf(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_boundary(&self) -> Option<Ref<Boundary>> {
        get_inner(self.borrow(), |op| match op {
            Op::Boundary(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_boundary_mut(&self) -> Option<RefMut<Boundary>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Boundary(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_add(&self) -> Option<Ref<Add>> {
        get_inner(self.borrow(), |op| match op {
            Op::Add(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_add_mut(&self) -> Option<RefMut<Add>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Add(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_sub(&self) -> Option<Ref<Sub>> {
        get_inner(self.borrow(), |op| match op {
            Op::Sub(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_sub_mut(&self) -> Option<RefMut<Sub>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Sub(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_mul(&self) -> Option<Ref<Mul>> {
        get_inner(self.borrow(), |op| match op {
            Op::Mul(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_mul_mut(&self) -> Option<RefMut<Mul>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Mul(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_if(&self) -> Option<Ref<If>> {
        get_inner(self.borrow(), |op| match op {
            Op::If(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_if_mut(&self) -> Option<RefMut<If>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::If(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_for(&self) -> Option<Ref<For>> {
        get_inner(self.borrow(), |op| match op {
            Op::For(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_for_mut(&self) -> Option<RefMut<For>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::For(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_call(&self) -> Option<Ref<Call>> {
        get_inner(self.borrow(), |op| match op {
            Op::Call(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_call_mut(&self) -> Option<RefMut<Call>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Call(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_fold(&self) -> Option<Ref<Fold>> {
        get_inner(self.borrow(), |op| match op {
            Op::Fold(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_fold_mut(&self) -> Option<RefMut<Fold>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Fold(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_vector(&self) -> Option<Ref<Vector>> {
        get_inner(self.borrow(), |op| match op {
            Op::Vector(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_vector_mut(&self) -> Option<RefMut<Vector>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Vector(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_matrix(&self) -> Option<Ref<Matrix>> {
        get_inner(self.borrow(), |op| match op {
            Op::Matrix(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_matrix_mut(&self) -> Option<RefMut<Matrix>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Matrix(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_accessor(&self) -> Option<Ref<Accessor>> {
        get_inner(self.borrow(), |op| match op {
            Op::Accessor(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_accessor_mut(&self) -> Option<RefMut<Accessor>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Accessor(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_parameter(&self) -> Option<Ref<Parameter>> {
        get_inner(self.borrow(), |op| match op {
            Op::Parameter(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_parameter_mut(&self) -> Option<RefMut<Parameter>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Parameter(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_value(&self) -> Option<Ref<Value>> {
        get_inner(self.borrow(), |op| match op {
            Op::Value(inner) => Some(inner),
            _ => None,
        })
    }
    pub fn as_value_mut(&self) -> Option<RefMut<Value>> {
        get_inner_mut(self.borrow_mut(), |op| match op {
            Op::Value(inner) => Some(inner),
            _ => None,
        })
    }
}
