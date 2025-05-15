use air_parser::{ast::Identifier, LexicalScope};
use air_pass::Pass;
use miden_diagnostics::DiagnosticsHandler;

use super::visitor::Visitor;
use crate::{
    ir::{Link, Mir, Node, Op, Parent, Root},
    CompileError,
};

use std::ops::Deref;

/// TODO MIR:
/// If needed, implement constant propagation / folding pass on MIR
/// Run through every operation in the graph
/// If we can deduce the resulting value based on the constants of the operands,
/// replace the operation itself with a constant
///
pub struct ConstantPropagation<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,
    work_stack: Vec<Link<Node>>,
    bindings: LexicalScope<Identifier, Link<Op>>,
}

impl Pass for ConstantPropagation<'_> {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        Visitor::run(self, ir.constraint_graph_mut())?;
        Ok(ir)
    }
}

impl<'a> ConstantPropagation<'a> {
    #[allow(unused)]
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self {
            diagnostics,
            work_stack: Default::default(),
            bindings: Default::default(),
        }
    }

    pub fn has_constant(&self, node: &Link<Node>) -> bool {
        if let Some(root) = node.as_root() {
            match root.borrow().deref() {
                Root::None(_) => false,
                _ => root
                    .children()
                    .borrow()
                    .iter()
                    .any(|child| self.has_constant(&child.as_node())),
            }
        } else if let Some(op) = node.as_op() {
            match op.borrow().deref() {
                Op::Value(_) => true,
                t if t.is_parent() => t
                    .children()
                    .borrow()
                    .iter()
                    .any(|child| self.has_constant(&child.as_node())),
                _ => unimplemented!(),
            }
        } else {
            return false;
        }
    }
}

impl Visitor for ConstantPropagation<'_> {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }
    fn root_nodes_to_visit(
        &self,
        graph: &crate::ir::Graph,
    ) -> Vec<crate::ir::Link<crate::ir::Node>> {
        let boundary_constraints_roots_ref = graph.boundary_constraints_roots.borrow();
        let integrity_constraints_roots_ref = graph.integrity_constraints_roots.borrow();
        let combined_roots = boundary_constraints_roots_ref
            .clone()
            .into_iter()
            .map(|bc| bc.as_node())
            .chain(
                integrity_constraints_roots_ref
                    .clone()
                    .into_iter()
                    .map(|ic| ic.as_node()),
            );
        combined_roots.collect()
    }
}
