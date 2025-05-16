use air_parser::ast::AccessType;
use air_pass::Pass;
use miden_diagnostics::DiagnosticsHandler;

use super::{duplicate_node, visitor::Visitor};
use crate::{
    ir::{
        extract_all_roots, ConstantValue, Graph, Link, Mir, MirValue, Node, Op, Parent, Root,
        SpannedMirValue, Value,
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
}

impl Pass for ConstantPropagation<'_> {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        Visitor::run(self, ir.constraint_graph_mut())?;
        ir.constraint_graph_mut().constants.clear();
        dbg!(&ir);
        Ok(ir)
    }
}

impl<'a> ConstantPropagation<'a> {
    #[allow(unused)]
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self {
            diagnostics,
            work_stack: Default::default(),
        }
    }
    fn replace_constant_children(
        &mut self,
        graph: &mut Graph,
        node: Link<Node>,
    ) -> Result<(), CompileError> {
        debug(&node);
        let children = node.children();
        for child in children.borrow().iter() {
            if has_constant(&child.as_node()) {
                self.visit_node(graph, child.as_node())?;
            }
        }
        Ok(())
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
        graph: &mut Graph,
        function: Link<Root>,
    ) -> Result<(), CompileError> {
        self.replace_constant_children(graph, function.as_node())?;
        debug(&function.as_node());
        Ok(())
    }
    fn visit_evaluator(
        &mut self,
        graph: &mut Graph,
        evaluator: Link<Root>,
    ) -> Result<(), CompileError> {
        self.replace_constant_children(graph, evaluator.as_node())?;
        debug(&evaluator.as_node());
        Ok(())
    }
    fn visit_enf(&mut self, graph: &mut Graph, enf: Link<Op>) -> Result<(), CompileError> {
        self.replace_constant_children(graph, enf.as_node())?;
        Ok(())
    }
    fn visit_boundary(
        &mut self,
        graph: &mut Graph,
        boundary: Link<Op>,
    ) -> Result<(), CompileError> {
        self.replace_constant_children(graph, boundary.as_node())?;
        debug(&boundary.as_node());
        Ok(())
    }
    fn visit_add(&mut self, graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        self.replace_constant_children(graph, op.as_node())?;
        if !is_constant(&op.as_node()) {
            // This is not a constant, so we don't need to replace it
            return Ok(());
        }
        let orig = duplicate_node(op.clone(), &mut Default::default());
        if let Some(add) = orig.as_add() {
            if let (Some(lhs), Some(rhs)) = (as_constant(&add.lhs), as_constant(&add.rhs)) {
                op.update(&(lhs + rhs).into());
            };
        }
        debug(&op.as_node());
        Ok(())
    }
    fn visit_sub(&mut self, graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        self.replace_constant_children(graph, op.as_node())?;
        if !is_constant(&op.as_node()) {
            // This is not a constant, so we don't need to replace it
            return Ok(());
        }
        let orig = duplicate_node(op.clone(), &mut Default::default());
        if let Some(sub) = orig.as_sub() {
            if let (Some(lhs), Some(rhs)) = (as_constant(&sub.lhs), as_constant(&sub.rhs)) {
                op.update(&(lhs - rhs).into());
            };
        }
        debug(&op.as_node());
        Ok(())
    }
    fn visit_mul(&mut self, graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        self.replace_constant_children(graph, op.as_node())?;
        if !is_constant(&op.as_node()) {
            // This is not a constant, so we don't need to replace it
            return Ok(());
        }
        let orig = duplicate_node(op.clone(), &mut Default::default());
        if let Some(mul) = orig.as_mul() {
            if let (Some(lhs), Some(rhs)) = (as_constant(&mul.lhs), as_constant(&mul.rhs)) {
                op.update(&(lhs * rhs).into());
            };
        }
        debug(&op.as_node());
        Ok(())
    }
    fn visit_exp(&mut self, graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        self.replace_constant_children(graph, op.as_node())?;
        if !is_constant(&op.as_node()) {
            // This is not a constant, so we don't need to replace it
            return Ok(());
        }
        let orig = duplicate_node(op.clone(), &mut Default::default());
        if let Some(mul) = orig.as_exp() {
            if let (Some(lhs), Some(rhs)) = (as_constant(&mul.lhs), as_constant(&mul.rhs)) {
                assert!(rhs < 2_u64.pow(32));
                op.update(&(lhs.pow(rhs as u32)).into());
            };
        }
        debug(&op.as_node());
        Ok(())
    }
    fn visit_if(&mut self, graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        self.replace_constant_children(graph, op.as_node())?;
        Ok(())
    }
    fn visit_for(&mut self, graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        self.replace_constant_children(graph, op.as_node())?;
        Ok(())
    }
    fn visit_call(&mut self, graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        self.replace_constant_children(graph, op.as_node())?;
        Ok(())
    }
    fn visit_fold(&mut self, graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        self.replace_constant_children(graph, op.as_node())?;
        Ok(())
    }
    fn visit_vector(&mut self, graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        self.replace_constant_children(graph, op.as_node())?;
        Ok(())
    }
    fn visit_matrix(&mut self, graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        self.replace_constant_children(graph, op.as_node())?;
        Ok(())
    }
    fn visit_accessor(&mut self, graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        self.replace_constant_children(graph, op.as_node())?;
        if !is_constant(&op.as_node()) {
            // This is not a constant, so we don't need to replace it
            return Ok(());
        }
        let orig = duplicate_node(op.clone(), &mut Default::default());
        if let Some(accessor) = orig.as_accessor() {
            let indexable = accessor.indexable.clone();
            match (
                accessor.access_type.clone(),
                accessor.indexable.borrow().deref(),
            ) {
                (AccessType::Default, _) => {
                    if let Some(value) = as_constant(&indexable) {
                        op.update(&value.into());
                    }
                }
                (AccessType::Slice(_), _) => {
                    unimplemented!("Slice accessors are not supported yet");
                }
                (AccessType::Index(index), Op::Vector(v)) => {
                    let value = v
                        .get_element(index)
                        .unwrap_or_else(|| panic!("Index out of bounds: {} >= {}", index, v.size));
                    if let Some(value) = as_constant(&value) {
                        op.update(&value.into());
                    }
                }
                (AccessType::Index(index), Op::Matrix(m)) => {
                    let value = m
                        .get_element(index)
                        .unwrap_or_else(|| panic!("Index out of bounds: {} >= {}", index, m.size));
                    if let Some(value) = as_constant(&value) {
                        op.update(&value.into());
                    }
                }
                (AccessType::Matrix(row_idx, col_idx), Op::Matrix(m)) => {
                    let row = m.get_element(row_idx).unwrap_or_else(|| {
                        panic!("Index out of bounds: {} >= {}", row_idx, m.size)
                    });
                    let vector = row
                        .as_vector()
                        .unwrap_or_else(|| panic!("Expected a vector, got: {:?}", m));
                    let value = vector.get_element(col_idx).unwrap_or_else(|| {
                        panic!("Index out of bounds: {} >= {}", col_idx, m.size)
                    });
                    if let Some(value) = as_constant(&value) {
                        op.update(&value.into());
                    }
                }
                _ => {
                    unimplemented!(
                        "Unsupported access type: {:?} for {:?}",
                        accessor.access_type,
                        indexable
                    );
                }
            };
        }
        Ok(())
    }
    fn visit_bus_op(&mut self, graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        self.replace_constant_children(graph, op.as_node())?;
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

fn as_constant(op: &Link<Op>) -> Option<u64> {
    op.as_value().map(|v| {
        if let Value {
            value:
                SpannedMirValue {
                    value: MirValue::Constant(ConstantValue::Felt(value)),
                    ..
                },
            ..
        } = v.deref()
        {
            Some(*value)
        } else {
            None
        }
    })?
}
