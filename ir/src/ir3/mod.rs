#![allow(unused)]
mod constraints;
mod graph;
mod leaf;
mod link;
mod mir;
mod nodes;
mod op;
mod owner;
mod root;
mod trace;

use std::ops::DerefMut;

pub use constraints::ConstraintError;
pub use graph::Graph;
pub use leaf::Leaf;
pub use link::{BackLink, Link};
pub use mir::Mir;
pub use nodes::*;
pub use trace::TraceAccess;
pub use op::Op;
pub use owner::Owner;
pub use root::Root;

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
