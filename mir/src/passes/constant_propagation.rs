use air_parser::ast::AccessType;
use air_pass::Pass;
use miden_diagnostics::DiagnosticsHandler;

use super::{duplicate_node, visitor::Visitor};
use crate::{
    ir::{
        extract_all_roots, Accessor, ConstantValue, Graph, Link, Mir, MirValue, Node, Op,
        Parameter, Parent, Root, SpannedMirValue, Value,
    },
    CompileError,
};

use std::{collections::BTreeMap, ops::Deref};

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
    swap_map_ops: BTreeMap<usize, Link<Op>>,
}

impl Pass for ConstantPropagation<'_> {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        Visitor::run(self, ir.constraint_graph_mut())?;
        #[cfg(feature = "debug_const_prop")]
        eprintln!(
            "Constant Propagation: Final graph:\n{:#?}",
            ir.constraint_graph()
        );
        ir.constraint_graph_mut().constants.clear();
        Ok(ir)
    }
}

impl<'a> ConstantPropagation<'a> {
    #[allow(unused)]
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self {
            diagnostics,
            work_stack: Default::default(),
            swap_map_ops: BTreeMap::new(),
        }
    }

    #[allow(unused)]
    fn debug(&mut self, prefix: &str, node: &Link<Node>) {
        eprintln!(
            "{}is_constant: {}, has_constant: {}, node: {}",
            prefix,
            is_constant(node),
            has_constant(node),
            node.debug(),
        );
    }

    fn swap_op(&mut self, old: &Link<Op>, new: Link<Op>) {
        self.swap_map_ops.insert(old.get_ptr(), new);
    }
}

impl Visitor for ConstantPropagation<'_> {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }

    fn post_visit(&mut self, _graph: &mut Graph, node: Link<Node>) -> Result<(), CompileError> {
        if let Some(op) = node.as_op() {
            if let Some(new_op) = self.swap_map_ops.remove(&op.get_ptr()) {
                // If we have a new op to swap, do it
                op.set(&new_op);
                #[cfg(feature = "debug_const_prop")]
                self.debug("  -> Swapped with: ", &new_op.as_node());
            } else {
                #[cfg(feature = "debug_const_prop")]
                self.debug("  -> No swap", &node);
            }
        }
        Ok(())
    }

    #[allow(unused)]
    fn debug(&mut self, prefix: &str, node: &Link<Node>) {
        eprintln!(
            "{}is_constant: {}, has_constant: {}, node: {}",
            prefix,
            is_constant(node),
            has_constant(node),
            node.debug(),
        );
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
        _function: Link<Root>,
    ) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_evaluator(
        &mut self,
        _graph: &mut Graph,
        _evaluator: Link<Root>,
    ) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_enf(&mut self, _graph: &mut Graph, _enf: Link<Op>) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_boundary(
        &mut self,
        _graph: &mut Graph,
        _boundary: Link<Op>,
    ) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_add(&mut self, _graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        if !is_constant(&op.as_node()) {
            // This is not a constant, so we don't need to replace it
            return Ok(());
        }
        let orig = duplicate_node(op.clone(), &mut Default::default());
        if let Some(add) = orig.as_add() {
            if let (Some(lhs), Some(rhs)) = (as_constant(&add.lhs), as_constant(&add.rhs)) {
                let new: Link<Op> = (lhs + rhs).into();
                new.as_value_mut().unwrap().value.span = add.span;
                self.swap_op(&op, new);
            };
        }
        Ok(())
    }

    fn visit_sub(&mut self, _graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        if !is_constant(&op.as_node()) {
            // This is not a constant, so we don't need to replace it
            return Ok(());
        }
        let orig = duplicate_node(op.clone(), &mut Default::default());
        if let Some(sub) = orig.as_sub() {
            if let (Some(lhs), Some(rhs)) = (as_constant(&sub.lhs), as_constant(&sub.rhs)) {
                let new: Link<Op> = (lhs - rhs).into();
                new.as_value_mut().unwrap().value.span = sub.span;
                self.swap_op(&op, new);
            };
        }
        Ok(())
    }

    fn visit_mul(&mut self, _graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        if !is_constant(&op.as_node()) {
            // This is not a constant, so we don't need to replace it
            return Ok(());
        }
        let orig = duplicate_node(op.clone(), &mut Default::default());
        if let Some(mul) = orig.as_mul() {
            if let (Some(lhs), Some(rhs)) = (as_constant(&mul.lhs), as_constant(&mul.rhs)) {
                let new: Link<Op> = (lhs * rhs).into();
                new.as_value_mut().unwrap().value.span = mul.span;
                self.swap_op(&op, new);
            };
        }
        Ok(())
    }

    fn visit_exp(&mut self, _graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
        if !is_constant(&op.as_node()) {
            // This is not a constant, so we don't need to replace it
            return Ok(());
        }
        let orig = duplicate_node(op.clone(), &mut Default::default());
        if let Some(exp) = orig.as_exp() {
            if let (Some(lhs), Some(rhs)) = (as_constant(&exp.lhs), as_constant(&exp.rhs)) {
                assert!(rhs < 2_u64.pow(32));
                let new: Link<Op> = (lhs.pow(rhs as u32)).into();
                new.as_value_mut().unwrap().value.span = exp.span;
                self.swap_op(&op, new);
            };
        }
        Ok(())
    }

    fn visit_if(&mut self, _graph: &mut Graph, _if_node: Link<Op>) -> Result<(), CompileError> {
        if !is_constant(&_if_node.as_node()) {
            // This is not a constant, so we don't need to replace it
            return Ok(());
        }
        Ok(())
    }

    fn visit_for(&mut self, _graph: &mut Graph, _for_node: Link<Op>) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_call(&mut self, _graph: &mut Graph, _call: Link<Op>) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_fold(&mut self, _graph: &mut Graph, _fold: Link<Op>) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_vector(&mut self, _graph: &mut Graph, _vector: Link<Op>) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_matrix(&mut self, _graph: &mut Graph, _matrix: Link<Op>) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_accessor(&mut self, graph: &mut Graph, op: Link<Op>) -> Result<(), CompileError> {
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
                        let new: Link<Op> = value.into();
                        new.as_value_mut().unwrap().value.span = accessor.span;
                        self.swap_op(&op, new);
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
                        let new: Link<Op> = value.into();
                        new.as_value_mut().unwrap().value.span = accessor.span;
                        self.swap_op(&op, new);
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
                        let new: Link<Op> = value.into();
                        new.as_value_mut().unwrap().value.span = accessor.span;
                        self.swap_op(&op, new);
                    }
                }
                (AccessType::Index(index), Op::Matrix(m)) => {
                    let value = m
                        .get_element(index)
                        .unwrap_or_else(|| panic!("Index out of bounds: {} >= {}", index, m.size));
                    if !is_constant(&value.as_node()) {
                        // This is not a constant, so we don't need to replace it
                        return Ok(());
                    }
                    let new_accessor =
                        Accessor::create(value.clone(), AccessType::Default, 0, accessor.span);
                    self.swap_op(&op, new_accessor);
                    self.scan_node(graph, op.as_node())?;
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

    fn visit_bus_op(&mut self, _graph: &mut Graph, _bus_op: Link<Op>) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_parameter(
        &mut self,
        _graph: &mut Graph,
        _parameter: Link<Op>,
    ) -> Result<(), CompileError> {
        Ok(())
    }

    fn visit_value(&mut self, _graph: &mut Graph, _value: Link<Op>) -> Result<(), CompileError> {
        Ok(())
    }
}

const DO_ALL: bool = true;
const DO_ANY: bool = false;
fn is_or_has_constant<F>(f: F, all: bool, node: &Link<Node>) -> bool
where
    F: Fn(&Link<Node>) -> bool,
{
    if node.is_parent() {
        if all {
            return node
                .children()
                .borrow()
                .iter()
                .all(|child| f(&child.as_node()));
        } else {
            return node
                .children()
                .borrow()
                .iter()
                .any(|child| f(&child.as_node()));
        }
    } else if let Some(op) = node.as_op() {
        match op.borrow().deref() {
            Op::Value(Value {
                value:
                    SpannedMirValue {
                        value: MirValue::Constant(_),
                        ..
                    },
                ..
            }) => {
                // This is a constant
                return true;
            }
            Op::Parameter(Parameter { ref_node, .. }) => {
                let Some(ref_node_link) = ref_node.to_link() else {
                    // This is a stale node
                    return false;
                };
                let Some(ref_op) = ref_node_link.as_op() else {
                    return false;
                };
                match ref_op.borrow().deref() {
                    Op::For(for_ref) => {
                        if all {
                            return for_ref
                                .iterators
                                .borrow()
                                .iter()
                                .all(|iterator| f(&iterator.as_node()));
                        } else {
                            return for_ref
                                .iterators
                                .borrow()
                                .iter()
                                .any(|iterator| f(&iterator.as_node()));
                        }
                    }
                    Op::Call(call_ref) => {
                        if all {
                            return call_ref
                                .arguments
                                .borrow()
                                .iter()
                                .all(|arg| f(&arg.as_node()));
                        } else {
                            return call_ref
                                .arguments
                                .borrow()
                                .iter()
                                .any(|arg| f(&arg.as_node()));
                        }
                    }
                    _ => {
                        // This has no reference to a potential constant
                        return false;
                    }
                };
            }
            _ => return false,
        }
    }
    // This is a stale node
    false
}

fn is_constant(node: &Link<Node>) -> bool {
    is_or_has_constant(is_constant, DO_ALL, node)
}
fn has_constant(node: &Link<Node>) -> bool {
    is_or_has_constant(has_constant, DO_ANY, node)
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
