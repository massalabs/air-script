use air_parser::{ast::Identifier, LexicalScope};
use air_pass::Pass;
use miden_diagnostics::DiagnosticsHandler;

use super::visitor::Visitor;
use crate::{
    ir::{
        extract_all_roots, Graph, Link, Mir, MirValue, Node, Op, Parent, Root, SpannedMirValue,
        Value,
    },
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
}

impl Visitor for ConstantPropagation<'_> {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }
    fn root_nodes_to_visit(
        &self,
        graph: &crate::ir::Graph,
    ) -> Vec<crate::ir::Link<crate::ir::Node>> {
        extract_all_roots(graph)
            .iter()
            .filter(|root| has_constant(root))
            .cloned()
            .collect()
    }
    fn visit_function(
        &mut self,
        _graph: &mut Graph,
        function: Link<Root>,
    ) -> Result<(), CompileError> {
        debug(&function.as_node());
        Ok(())
    }
    fn visit_evaluator(
        &mut self,
        _graph: &mut Graph,
        evaluator: Link<Root>,
    ) -> Result<(), CompileError> {
        debug(&evaluator.as_node());
        Ok(())
    }
    fn visit_enf(&mut self, _graph: &mut Graph, enf: Link<Op>) -> Result<(), CompileError> {
        debug(&enf.as_node());
        Ok(())
    }
    fn visit_boundary(
        &mut self,
        _graph: &mut Graph,
        boundary: Link<Op>,
    ) -> Result<(), CompileError> {
        debug(&boundary.as_node());
        Ok(())
    }
    fn visit_add(&mut self, _graph: &mut Graph, add: Link<Op>) -> Result<(), CompileError> {
        debug(&add.as_node());
        Ok(())
    }
    fn visit_sub(&mut self, _graph: &mut Graph, sub: Link<Op>) -> Result<(), CompileError> {
        debug(&sub.as_node());
        Ok(())
    }
    fn visit_mul(&mut self, _graph: &mut Graph, mul: Link<Op>) -> Result<(), CompileError> {
        debug(&mul.as_node());
        Ok(())
    }
    fn visit_exp(&mut self, _graph: &mut Graph, exp: Link<Op>) -> Result<(), CompileError> {
        debug(&exp.as_node());
        Ok(())
    }
    fn visit_if(&mut self, _graph: &mut Graph, if_node: Link<Op>) -> Result<(), CompileError> {
        debug(&if_node.as_node());
        Ok(())
    }
    fn visit_for(&mut self, _graph: &mut Graph, for_node: Link<Op>) -> Result<(), CompileError> {
        debug(&for_node.as_node());
        Ok(())
    }
    fn visit_call(&mut self, _graph: &mut Graph, call: Link<Op>) -> Result<(), CompileError> {
        debug(&call.as_node());
        Ok(())
    }
    fn visit_fold(&mut self, _graph: &mut Graph, fold: Link<Op>) -> Result<(), CompileError> {
        debug(&fold.as_node());
        Ok(())
    }
    fn visit_vector(&mut self, _graph: &mut Graph, vector: Link<Op>) -> Result<(), CompileError> {
        debug(&vector.as_node());
        Ok(())
    }
    fn visit_matrix(&mut self, _graph: &mut Graph, matrix: Link<Op>) -> Result<(), CompileError> {
        debug(&matrix.as_node());
        Ok(())
    }
    fn visit_accessor(
        &mut self,
        _graph: &mut Graph,
        accessor: Link<Op>,
    ) -> Result<(), CompileError> {
        debug(&accessor.as_node());
        Ok(())
    }
    fn visit_bus_op(&mut self, _graph: &mut Graph, bus_op: Link<Op>) -> Result<(), CompileError> {
        debug(&bus_op.as_node());
        Ok(())
    }
    fn visit_parameter(
        &mut self,
        _graph: &mut Graph,
        parameter: Link<Op>,
    ) -> Result<(), CompileError> {
        debug(&parameter.as_node());
        Ok(())
    }
    fn visit_value(&mut self, _graph: &mut Graph, value: Link<Op>) -> Result<(), CompileError> {
        debug(&value.as_node());
        Ok(())
    }
}

fn is_constant(node: &Link<Node>) -> bool {
    if node.is_parent() {
        return node
            .children()
            .borrow()
            .iter()
            .all(|child| is_constant(&child.as_node()));
    } else if let Some(op) = node.as_op() {
        if let Some(v) = op.as_value() {
            if let Value {
                value:
                    SpannedMirValue {
                        value: MirValue::Constant(_),
                        ..
                    },
                ..
            } = v.deref()
            {
                // This is a constant
                return true;
            }
            // This is a value that is not a constant
            return false;
        }
        // This is an operation that is not a value, nor a parent
        return false;
    }
    // This is a stale node
    false
}

fn has_constant(node: &Link<Node>) -> bool {
    if node.is_parent() {
        return node
            .children()
            .borrow()
            .iter()
            .any(|child| has_constant(&child.as_node()));
    } else if let Some(op) = node.as_op() {
        if let Some(v) = op.as_value() {
            if let Value {
                value:
                    SpannedMirValue {
                        value: MirValue::Constant(_),
                        ..
                    },
                ..
            } = v.deref()
            {
                // This is a constant
                return true;
            }
            // This is a value that is not a constant
            return false;
        }
        // This is an operation that is not a value, nor a parent
        return false;
    }
    // This is a stale node
    false
}

fn debug(node: &Link<Node>) {
    eprintln!(
        "is_constant: {}, has_constant: {}, node: {}",
        is_constant(node),
        has_constant(node),
        node.debug(),
    );
}
