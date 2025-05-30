use std::collections::BTreeMap;

use air_pass::Pass;
use miden_diagnostics::DiagnosticsHandler;

use crate::{
    ir::{extract_all_roots, Graph, Link, Mir, Node, Op},
    CompileError,
};

use super::Visitor;

pub struct Unrolling2<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,
    work_stack: Vec<Link<Node>>,
    swap_map_ops: BTreeMap<usize, Link<Op>>,
}

impl<'a> Unrolling2<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self {
            diagnostics,
            work_stack: Vec::new(),
            swap_map_ops: BTreeMap::new(),
        }
    }
}

impl Pass for Unrolling2<'_> {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        Visitor::run(self, ir.constraint_graph_mut())?;
        #[cfg(feature = "debug_unrolling")]
        eprintln!("Unrolling: Final graph:\n{:#?}", ir.constraint_graph());
        ir.constraint_graph_mut().constants.clear();
        Ok(ir)
    }
}

impl Visitor for Unrolling2<'_> {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }

    fn pre_visit(&mut self, _graph: &mut Graph, _node: Link<Node>) -> Result<(), CompileError> {
        #[cfg(feature = "debug_unrolling")]
        self.debug(1, format!("{:?} ", _node).as_str(), &_node);
        Ok(())
    }

    fn post_visit(&mut self, _graph: &mut Graph, node: Link<Node>) -> Result<(), CompileError> {
        if let Some(op) = node.as_op() {
            if let Some(new_op) = self.swap_map_ops.remove(&op.get_ptr()) {
                // If we have a new op to swap, do it
                op.set(&new_op);
                #[cfg(feature = "debug_unrolling")]
                self.debug(1, "  -> Swapped with: ", &new_op.as_node());
            } else {
                #[cfg(feature = "debug_unrolling")]
                self.debug(1, "  -> No swap", &node);
            }
        }
        Ok(())
    }

    fn root_nodes_to_visit(
        &self,
        graph: &crate::ir::Graph,
    ) -> Vec<crate::ir::Link<crate::ir::Node>> {
        extract_all_roots(graph)
    }

    fn visit_for(&mut self, _graph: &mut Graph, for_op: Link<Op>) -> Result<(), CompileError> {
        // Here we would implement the logic for unrolling the loop
        // For now, we just print the node
        Ok(())
    }
}
