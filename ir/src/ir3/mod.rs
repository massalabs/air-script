mod graph;
mod link;
mod nodes;
use std::ops::DerefMut;

pub use graph::Graph;
pub use link::{BackLink, Link};
pub use nodes::{
    Add, Boundary, Call, Enf, Evaluator, Fold, For, Function, If, Matrix, Mul, Parameter, Sub,
    Value, Vector,
};

pub trait Parent {
    type Child;
    fn children(&self) -> Link<Vec<Link<Self::Child>>>;
    fn remove_child(&mut self, child: Link<Self::Child>)
    where
        Self::Child: PartialEq,
    {
        let children = self.children();
        children.borrow_mut().deref_mut().retain(|c| c != &child);
    }
}

pub trait Builder: Parent {}

pub trait Child: Clone + Into<Link<Self>> + PartialEq {
    type Parent;
    fn get_parent(&self) -> BackLink<Self::Parent>;
    fn set_parent(&mut self, parent: Link<Self::Parent>);
    fn swap_parent(&mut self, new_parent: Link<Self::Parent>)
    where
        Self::Parent: PartialEq + Parent<Child = Self>,
    {
        // Grab the old parent before we change it
        let old_parent = self.get_parent().to_link();
        // Remove self from the old parent's children
        if let Some(parent) = old_parent {
            if parent != new_parent {
                parent
                    .borrow_mut()
                    .deref_mut()
                    .remove_child(self.clone().into());
            }
        }
        // Change the parent
        self.set_parent(new_parent);
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Root {
    Function(Function),
    Evaluator(Evaluator),
    #[default]
    None,
}

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
    #[default]
    None,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Owner {
    Function(Function),
    Evaluator(Evaluator),
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
    #[default]
    None,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Leaf {
    Parameter(Parameter),
    Value(Value),
    #[default]
    None,
}
