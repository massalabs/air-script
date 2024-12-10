#![allow(unused)]
mod graph;
mod link;
mod nodes;
use std::ops::DerefMut;

pub use graph::Graph;
pub use link::{BackLink, Link};
pub use nodes::{
    Accessor, Add, Boundary, Call, Enf, Evaluator, Fold, For, Function, If, Matrix, Mul, Parameter,
    Sub, Value, Vector,
};

/// A trait for nodes that can have children
/// This is used with the Child trait to allow for easy traversal and manipulation of the graph
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

/// A trait for nodes that can have a parent
/// This is used with the Parent trait to allow for easy traversal and manipulation of the graph
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

/// A helper struct used with the Builder trait to indicate that a field has not been set
pub struct NotSet;

/// A trait implemented by all nodes.
/// Will be derivable later. The implementation and type-safe builder is currently manual while we tweak the design
pub trait Builder {
    type BuilderEmpty;
    type BuilderFull;
    /// Create a new empty builder that exposes all fields
    fn builder() -> Self::BuilderEmpty;
    /// Consumes the current node
    /// and returns a new builder with all fields set to expose all fields
    fn edit(self) -> Self::BuilderFull;
}

/// The root nodes of the MIR Graph
/// These represent the top level functions and evaluators
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Root {
    Function(Function),
    Evaluator(Evaluator),
    #[default]
    None,
}

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
    IndexAccess(Accessor),
    Parameter(Parameter),
    Value(Value),
    #[default]
    None,
}

/// The nodes that can own Op nodes
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
    IndexAccess(Accessor),
    #[default]
    None,
}

/// The Final nodes of the MIR Graph.
/// Currently unused in the structure but will be used in the next visitor pattern implementation.
#[derive(Default, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Leaf {
    Parameter(Parameter),
    Value(Value),
    #[default]
    None,
}
