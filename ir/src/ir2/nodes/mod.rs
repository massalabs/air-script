mod aggregated_op;
mod binary_op;
mod blocks;
mod leaf_op;
mod scope;
mod structured_op;
mod unary_op;

use crate::ir2::{BackLink, Graph, IsChild, IsLeaf, IsNode, IsParent, Leaf, Link};
use crate::FoldOperator;

pub use aggregated_op::{Matrix, Vector};
pub use binary_op::{Add, Mul, Sub};
pub use blocks::{Evaluator, For, Function, If};
pub use leaf_op::*;
pub use scope::Scope;
pub use structured_op::{Call, Fold};
pub use unary_op::{Boundary, Enf};

use air_parser::ast;
use miden_diagnostics::SourceSpan;

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

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

#[derive(Clone, Eq, PartialEq, IsLeaf)]
pub enum LeafNode {
    Value(Leaf<SpannedMirValue>),
    Parameter(Leaf<Parameter>),
}

impl IsParent for LeafNode {
    fn get_children(&self) -> Link<Vec<Link<NodeType>>> {
        unreachable!("LeafNode has no children: {:?}", self)
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
    Vector(Vector),
    Matrix(Matrix),
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

// Binary
impl Add {
    pub fn new(lhs: Link<NodeType>, rhs: Link<NodeType>) -> Add {
        let mut add_node: Add = Add::default();
        add_node.add_lhs(lhs);
        add_node.add_rhs(rhs);
        add_node
    }
    fn add_lhs(&mut self, lhs: Link<NodeType>) {
        self.get_children().borrow_mut().push(lhs.clone());
        match lhs.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
    fn add_rhs(&mut self, rhs: Link<NodeType>) {
        self.get_children().borrow_mut().push(rhs.clone());
        match rhs.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
}

impl Sub {
    pub fn new(lhs: Link<NodeType>, rhs: Link<NodeType>) -> Sub {
        let mut sub_node = Sub::default();
        sub_node.add_lhs(lhs);
        sub_node.add_rhs(rhs);
        sub_node
    }
    fn add_lhs(&mut self, lhs: Link<NodeType>) {
        self.get_children().borrow_mut().push(lhs.clone());
        match lhs.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
    fn add_rhs(&mut self, rhs: Link<NodeType>) {
        self.get_children().borrow_mut().push(rhs.clone());
        match rhs.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
}

impl Mul {
    pub fn new(lhs: Link<NodeType>, rhs: Link<NodeType>) -> Mul {
        let mut mul_node = Mul::default();
        mul_node.add_lhs(lhs);
        mul_node.add_rhs(rhs);
        mul_node
    }
    fn add_lhs(&mut self, lhs: Link<NodeType>) {
        self.get_children().borrow_mut().push(lhs.clone());
        match lhs.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
    fn add_rhs(&mut self, rhs: Link<NodeType>) {
        self.get_children().borrow_mut().push(rhs.clone());
        match rhs.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
}

// Unary
impl Enf {
    pub fn new(child: Link<NodeType>) -> Enf {
        let mut enf_node = Enf::default();
        enf_node.add_child(child);
        enf_node
    }
}

impl Boundary {
    pub fn new(child: Link<NodeType>, boundary: ast::Boundary) -> Boundary {
        let mut boundary_node = Boundary::default();
        boundary_node.kind = boundary;
        boundary_node.add_child(child);
        boundary_node
    }
}

// Aggregated
impl Vector {
    pub fn new(values: Vec<Link<NodeType>>) -> Vector {
        let mut vec_node = Vector::default();
        for child in values {
            vec_node.add_child(child);
        }
        vec_node
    }
}

impl Matrix {
    pub fn new(values: Vec<Vec<Link<NodeType>>>) -> Matrix {
        let mut matrix_node = Matrix::default();
        for row in values {
            let vec_node = Vector::new(row);
            matrix_node.add_child(vec_node.into());
        }
        matrix_node
    }
}

// Blocks
impl Function {
    pub fn new(args: Vec<Link<NodeType>>, ret: Link<NodeType>, body: Link<NodeType>) -> Function {
        let mut func_node = Function::default();
        for arg in args {
            func_node.add_arg(arg);
        }
        func_node.add_ret(ret);
        func_node.add_body(body);
        func_node
    }
    pub fn add_arg(&mut self, arg: Link<NodeType>) {
        self.get_children()
            .borrow_mut()
            .insert(self.args_count, arg.clone());
        match arg.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
        self.args_count += 1;
    }
    pub fn add_ret(&mut self, ret: Link<NodeType>) {
        self.get_children()
            .borrow_mut()
            .insert(self.args_count, ret.clone());
        match ret.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
    fn add_body(&mut self, body: Link<NodeType>) {
        self.get_children()
            .borrow_mut()
            .insert(self.args_count + 1, body.clone());
        match body.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
}

impl Evaluator {
    pub fn new(args: Vec<Link<NodeType>>, body: Link<NodeType>) -> Evaluator {
        let mut eval_node = Evaluator::default();
        for arg in args {
            eval_node.add_arg(arg);
        }
        eval_node.add_body(body);
        eval_node
    }
    pub fn add_arg(&mut self, arg: Link<NodeType>) {
        self.get_children()
            .borrow_mut()
            .insert(self.args_count, arg.clone());
        match arg.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
        self.args_count += 1;
    }
    fn add_body(&mut self, body: Link<NodeType>) {
        self.get_children()
            .borrow_mut()
            .insert(self.args_count, body.clone());
        match body.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
}

impl If {
    pub fn new(
        cond: Link<NodeType>,
        then_branch: Link<NodeType>,
        else_branch: Link<NodeType>,
    ) -> If {
        let mut if_node = If::default();
        if_node.add_cond(cond);
        if_node.add_then_branch(then_branch);
        if_node.add_else_branch(else_branch);
        if_node
    }
    fn add_cond(&mut self, cond: Link<NodeType>) {
        self.get_children().borrow_mut().push(cond.clone());
        match cond.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
    fn add_then_branch(&mut self, then_branch: Link<NodeType>) {
        self.get_children().borrow_mut().push(then_branch.clone());
        match then_branch.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
    fn add_else_branch(&mut self, else_branch: Link<NodeType>) {
        self.get_children().borrow_mut().push(else_branch.clone());
        match else_branch.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
}

impl For {
    pub fn new(
        iterators: Vec<Link<NodeType>>,
        body: Link<NodeType>,
        selector: Option<Link<NodeType>>,
    ) -> For {
        let mut for_node = For::default();
        for iter in iterators {
            for_node.add_iterator(iter);
        }
        for_node.add_body(body);
        if let Some(selector) = selector {
            for_node.add_selector(selector);
        }
        for_node
    }
    fn add_iterator(&mut self, iterator: Link<NodeType>) {
        self.get_children()
            .borrow_mut()
            .insert(self.iterators_count, iterator.clone());
        match iterator.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
        self.iterators_count += 1;
    }
    fn add_body(&mut self, body: Link<NodeType>) {
        self.get_children()
            .borrow_mut()
            .insert(self.iterators_count, body.clone());
        match body.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
    fn add_selector(&mut self, selector: Link<NodeType>) {
        self.get_children()
            .borrow_mut()
            .insert(self.iterators_count + 1, selector.clone());
        match selector.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
}

// Structured
impl Fold {
    pub fn new(
        iterator: Link<NodeType>,
        operator: FoldOperator,
        initial_value: Link<NodeType>,
    ) -> Fold {
        let mut fold_node = Fold::default();
        fold_node.operator = operator;
        fold_node.add_iterator(iterator);
        fold_node.add_initial_value(initial_value);
        fold_node
    }
    fn add_iterator(&mut self, iterator: Link<NodeType>) {
        self.get_children().borrow_mut().push(iterator.clone());
        match iterator.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
    fn add_initial_value(&mut self, initial_value: Link<NodeType>) {
        self.get_children().borrow_mut().push(initial_value.clone());
        match initial_value.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
}

impl Call {
    pub fn new(function: Link<NodeType>, arguments: Vec<Link<NodeType>>) -> Call {
        let mut call_node = Call::default();
        call_node.add_function(function);
        for arg in arguments {
            call_node.add_argument(arg);
        }
        call_node
    }
    fn add_function(&mut self, function: Link<NodeType>) {
        self.get_children().borrow_mut().insert(0, function.clone());
        match function.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
    }
    fn add_argument(&mut self, argument: Link<NodeType>) {
        self.get_children()
            .borrow_mut()
            .insert(self.arguments_count + 1, argument.clone());
        match argument.borrow_mut().deref_mut() {
            NodeType::LeafNode(leaf_node) => leaf_node.swap_parent(self.clone().into()),
            NodeType::RootNode(root_node) => root_node.swap_parent(self.clone().into()),
            NodeType::MiddleNode(parent_and_child) => {
                parent_and_child.swap_parent(self.clone().into())
            }
        }
        self.arguments_count += 1;
    }
}

// Scope
impl Scope {
    pub fn new(child: Link<NodeType>) -> Scope {
        let mut scope_node = Scope::default();
        scope_node.add_child(child);
        scope_node
    }
}

// Leaf
/*impl Value {
    pub fn new<T>(data: T) -> Value
    where
        T: Into<Link<NodeType>>,
    {
        let node = data.into();
        node
    }
}*/

impl Parameter {
    pub fn new(span: SourceSpan, ty: MirType, argument_position: usize) -> Parameter {
        Parameter {
            span,
            ty,
            argument_position,
        }
    }
}
