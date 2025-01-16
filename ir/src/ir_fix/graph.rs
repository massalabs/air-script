use crate::ir_fix::{Evaluator, Function, Link, Op};
use std::collections::BTreeMap;

use air_parser::ast::QualifiedIdentifier;

#[derive(Debug, Default)]
pub struct Graph {
    functions: BTreeMap<QualifiedIdentifier, Link<Function>>,
    evaluators: BTreeMap<QualifiedIdentifier, Link<Evaluator>>,
    pub boundary_constraints_roots: Link<Vec<Link<Op>>>,
    pub integrity_constraints_roots: Link<Vec<Link<Op>>>,
}

impl Graph {
    pub fn create() -> Link<Self> {
        Graph::default().into()
    }
    pub fn insert_function(&mut self, ident: QualifiedIdentifier, node: Link<Function>) {
        self.functions.insert(ident, node);
    }

    pub fn get_function(&self, ident: &QualifiedIdentifier) -> Option<&Link<Function>> {
        self.functions.get(ident)
    }

    pub fn get_function_mut(&mut self, ident: &QualifiedIdentifier) -> Option<&mut Link<Function>> {
        self.functions.get_mut(ident)
    }

    pub fn get_function_nodes(&self) -> Vec<Link<Function>> {
        self.functions.values().cloned().collect()
    }

    pub fn insert_evaluator(&mut self, ident: QualifiedIdentifier, node: Link<Evaluator>) {
        self.evaluators.insert(ident, node);
    }

    pub fn get_evaluator(&self, ident: &QualifiedIdentifier) -> Option<&Link<Evaluator>> {
        self.evaluators.get(ident)
    }

    pub fn get_evaluator_mut(
        &mut self,
        ident: &QualifiedIdentifier,
    ) -> Option<&mut Link<Evaluator>> {
        self.evaluators.get_mut(ident)
    }

    pub fn get_evaluator_nodes(&self) -> Vec<Link<Evaluator>> {
        self.evaluators.values().cloned().collect()
    }

    pub fn insert_boundary_constraints_root(&mut self, root: Link<Op>) {
        if !self.boundary_constraints_roots.borrow().contains(&root) {
            self.boundary_constraints_roots
                .borrow_mut()
                .push(root.clone());
        }
    }

    pub fn remove_boundary_constraints_root(&mut self, root: Link<Op>) {
        self.boundary_constraints_roots
            .borrow_mut()
            .retain(|n| *n != root);
    }

    pub fn insert_integrity_constraints_root(&mut self, root: Link<Op>) {
        if !self.integrity_constraints_roots.borrow().contains(&root) {
            self.integrity_constraints_roots
                .borrow_mut()
                .push(root.clone());
        }
    }

    pub fn remove_integrity_constraints_root(&mut self, root: Link<Op>) {
        self.boundary_constraints_roots
            .borrow_mut()
            .retain(|n| *n != root);
    }
}
