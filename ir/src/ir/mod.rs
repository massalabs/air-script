mod constraints;
mod graph;
mod leaf;
mod link;
mod mir;
mod node;
mod nodes;
mod op;
mod owner;
mod root;
mod trace;
pub extern crate derive_ir;

use std::ops::DerefMut;

pub use constraints::ConstraintError;
pub use derive_ir::Builder;
pub use graph::Graph;
pub use leaf::Leaf;
pub use link::{BackLink, Link};
pub use mir::Mir;
pub use node::Node;
pub use nodes::*;
pub use op::Op;
pub use owner::Owner;
pub use root::Root;
pub use trace::TraceAccess;

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

impl<T> Parent for Link<T>
where
    T: Parent,
{
    type Child = T::Child;
    fn children(&self) -> Link<Vec<Link<Self::Child>>> {
        self.borrow().children()
    }
    fn remove_child(&mut self, child: Link<Self::Child>)
    where
        Self::Child: PartialEq,
    {
        self.borrow_mut().remove_child(child);
    }
}

/// A trait for nodes that can have a parent
/// This is used with the Parent trait to allow for easy traversal and manipulation of the graph
pub trait Child: Clone + Into<Link<Self>> + PartialEq {
    type Parent;
    fn get_parents(&self) -> Vec<BackLink<Self::Parent>>;
    fn add_parent(&mut self, parent: Link<Self::Parent>);
    fn remove_parent(&mut self, parent: Link<Self::Parent>);
    /*fn swap_parent(&mut self, old_parent: Link<Self::Parent>, new_parent: Link<Self::Parent>)
    where
        Self::Parent: PartialEq + Parent<Child = Self>,
    {
        self.remove_parent(old_parent);
        self.add_parent(new_parent);
    }*/
}

impl<T> Child for Link<T>
where
    T: Child,
{
    type Parent = T::Parent;

    fn get_parents(&self) -> Vec<BackLink<Self::Parent>> {
        self.borrow().get_parents()
    }
    fn add_parent(&mut self, parent: Link<Self::Parent>) {
        self.borrow_mut().add_parent(parent)
    }
    fn remove_parent(&mut self, parent: Link<Self::Parent>) {
        self.borrow_mut().remove_parent(parent)
    }
    /*fn swap_parent(&mut self, old_parent: Link<Self::Parent>, new_parent: Link<Self::Parent>) {
        self.borrow_mut().swap_parent(old_parent, new_parent)
    }*/
}

/// A helper struct used with the Builder trait to indicate that a field has not been set
pub struct NotSet;

/// A trait implemented by all nodes.
/// Will be derivable later. The implementation and type-safe builder is currently manual while we tweak the design
pub trait Builder {
    type Empty;
    type Full;
    /// Create a new empty builder that exposes all fields
    fn builder() -> Self::Empty;
}
