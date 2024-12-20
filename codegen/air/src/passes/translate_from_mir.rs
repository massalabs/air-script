use std::ops::Deref;

use air_parser::ast::TraceSegment;
use air_pass::Pass;

use miden_diagnostics::{DiagnosticsHandler, Severity, SourceSpan};
use mir::ir::*;

use crate::{graph::NodeIndex, ir::*, CompileError};

pub struct MirToAir<'a> {
    diagnostics: &'a DiagnosticsHandler,
}
impl<'a> MirToAir<'a> {
    /// Create a new instance of this pass
    #[inline]
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}
impl<'p> Pass for MirToAir<'p> {
    type Input<'a> = Mir;
    type Output<'a> = Air;
    type Error = CompileError;

    fn run<'a>(&mut self, mir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut air = Air::new(mir.name);

        air.trace_segment_widths = mir.trace_columns.iter().map(|ts| ts.size as u16).collect();
        air.num_random_values = mir.num_random_values;
        air.periodic_columns = mir.periodic_columns.clone();
        air.public_inputs = mir.public_inputs.clone();

        let mut builder = AirBuilder {
            diagnostics: self.diagnostics,
            air: &mut air,
            trace_columns: mir.trace_columns.clone(),
        };

        let graph = mir.constraint_graph();

        for bc in graph.boundary_constraints_roots.borrow().deref().iter() {
            builder.build_boundary_constraint(bc)?;
        }

        for ic in graph.integrity_constraints_roots.borrow().deref().iter() {
            builder.build_integrity_constraint(ic)?;
        }

        Ok(air)
    }
}

struct AirBuilder<'a> {
    diagnostics: &'a DiagnosticsHandler,
    air: &'a mut Air,
    trace_columns: Vec<TraceSegment>,
}

impl<'a> AirBuilder<'a> {
    fn insert_mir_operation(&mut self, mir_node: &Link<Op>) -> NodeIndex {
        match mir_node.borrow().deref() {
            Op::Add(add) => {
                let lhs = add.lhs.clone();
                let rhs = add.rhs.clone();
                let lhs_node_index = self.insert_mir_operation(&lhs);
                let rhs_node_index = self.insert_mir_operation(&rhs);
                return self.insert_op(Operation::Add(lhs_node_index, rhs_node_index));
            }
            Op::Sub(sub) => {
                let lhs = sub.lhs.clone();
                let rhs = sub.rhs.clone();
                let lhs_node_index = self.insert_mir_operation(&lhs);
                let rhs_node_index = self.insert_mir_operation(&rhs);
                return self.insert_op(Operation::Sub(lhs_node_index, rhs_node_index));
            }
            Op::Mul(mul) => {
                let lhs = mul.lhs.clone();
                let rhs = mul.rhs.clone();
                let lhs_node_index = self.insert_mir_operation(&lhs);
                let rhs_node_index = self.insert_mir_operation(&rhs);
                return self.insert_op(Operation::Mul(lhs_node_index, rhs_node_index));
            }
            Op::Value(value) => {
                let mir_value = &value.value.value;

                let value = match mir_value {
                    MirValue::Constant(constant_value) => {
                        if let ConstantValue::Felt(felt) = constant_value {
                            crate::ir::Value::Constant(*felt)
                        } else {
                            unreachable!()
                        }
                    }
                    MirValue::TraceAccess(trace_access) => {
                        crate::ir::Value::TraceAccess(crate::ir::TraceAccess {
                            segment: trace_access.segment,
                            column: trace_access.column,
                            row_offset: trace_access.row_offset,
                        })
                    }
                    MirValue::PeriodicColumn(periodic_column_access) => {
                        crate::ir::Value::PeriodicColumn(crate::ir::PeriodicColumnAccess {
                            name: periodic_column_access.name.clone(),
                            cycle: periodic_column_access.cycle,
                        })
                    }
                    MirValue::PublicInput(public_input_access) => {
                        crate::ir::Value::PublicInput(crate::ir::PublicInputAccess {
                            name: public_input_access.name.clone(),
                            index: public_input_access.index,
                        })
                    }
                    MirValue::RandomValue(rv) => crate::ir::Value::RandomValue(*rv),
                    _ => unreachable!(),
                };

                return self.insert_op(Operation::Value(value));
            }
            _ => unreachable!(),
        }
    }

    fn build_boundary_constraint(&mut self, bc: &Link<Op>) -> Result<(), CompileError> {
        match bc.borrow().deref() {
            Op::Vector(vector) => {
                let vec = vector.elements.borrow().deref().clone();
                for node in vec.iter() {
                    self.build_boundary_constraint(node)?;
                }
                return Ok(());
            }
            Op::Matrix(matrix) => {
                let rows = matrix.elements.borrow().deref().clone();
                for row in rows.iter() {
                    let vec = row.borrow().deref().children().borrow().deref().clone();
                    for node in vec.iter() {
                        self.build_boundary_constraint(node)?;
                    }
                }
                return Ok(());
            }
            Op::Enf(enf) => {
                let child_op = enf.expr.clone();

                let Op::Sub(_sub) = child_op.borrow().deref().clone() else {
                    unreachable!(); // Raise diag
                };

                self.build_boundary_constraint(&child_op)?;
                return Ok(());
            }
            Op::Sub(sub) => {
                // Check that lhs is a Bounded trace access
                let lhs = sub.lhs.clone();
                let rhs = sub.rhs.clone();

                let Op::Boundary(boundary) = lhs.borrow().deref().clone() else {
                    unreachable!(); // Raise diag
                };
                let expected_trace_access_expr = boundary.expr.clone();
                let Op::Value(value) = expected_trace_access_expr.borrow().deref().clone() else {
                    unreachable!(); // Raise diag
                };

                let (trace_access, lhs_span) = match value.value {
                    SpannedMirValue {
                        value: MirValue::TraceAccess(trace_access),
                        span: lhs_span,
                    } => (trace_access, lhs_span),
                    SpannedMirValue {
                        value: MirValue::TraceAccessBinding(trace_access_binding),
                        span: lhs_span,
                    } => {
                        if trace_access_binding.size != 1 {
                            self.diagnostics.diagnostic(Severity::Error)
                                        .with_message("invalid boundary constraint")
                                        .with_primary_label(lhs_span, "this has a trace access binding with a size greater than 1")
                                        .with_note("Boundary constraints require both sides of the constraint to be single columns.")
                                        .emit();
                            return Err(CompileError::Failed);
                        }
                        let trace_access = mir::ir::TraceAccess {
                            segment: trace_access_binding.segment,
                            column: trace_access_binding.offset,
                            row_offset: 0,
                        };
                        (trace_access, lhs_span)
                    }
                    _ => unreachable!("Expected TraceAccess, received {:?}", value.value), // Raise diag
                };

                if let Some(prev) = self.trace_columns[trace_access.segment].mark_constrained(
                    lhs_span,
                    trace_access.column,
                    boundary.kind,
                ) {
                    self.diagnostics
                                .diagnostic(Severity::Error)
                                .with_message("overlapping boundary constraints")
                                .with_primary_label(
                                    lhs_span,
                                    "this constrains a column and boundary that has already been constrained",
                                )
                                .with_secondary_label(prev, "previous constraint occurs here")
                                .emit();
                    return Err(CompileError::Failed);
                }

                let lhs = self
                    .air
                    .constraint_graph_mut()
                    .insert_node(Operation::Value(crate::ir::Value::TraceAccess(
                        crate::ir::TraceAccess {
                            segment: trace_access.segment,
                            column: trace_access.column,
                            row_offset: trace_access.row_offset,
                        },
                    )));
                let rhs = self.insert_mir_operation(&rhs);

                // Compare the inferred trace segment and domain of the operands
                let domain = boundary.kind.into();
                {
                    let graph = self.air.constraint_graph();
                    let (lhs_segment, lhs_domain) = graph.node_details(&lhs, domain)?;
                    let (rhs_segment, rhs_domain) = graph.node_details(&rhs, domain)?;
                    if lhs_segment < rhs_segment {
                        // trace segment inference defaults to the lowest segment (the main trace) and is
                        // adjusted according to the use of random values and trace columns.
                        let lhs_segment_name = self.trace_columns[lhs_segment].name;
                        let rhs_segment_name = self.trace_columns[rhs_segment].name;
                        self.diagnostics.diagnostic(Severity::Error)
                                    .with_message("invalid boundary constraint")
                                    .with_primary_label(lhs_span, format!("this constrains a column in the '{lhs_segment_name}' trace segment"))
                                    .with_secondary_label(SourceSpan::UNKNOWN, format!("but this expression implies the '{rhs_segment_name}' trace segment"))
                                    .with_note("Boundary constraints require both sides of the constraint to apply to the same trace segment.")
                                    .emit();
                        return Err(CompileError::Failed);
                    }
                    if lhs_domain != rhs_domain {
                        self.diagnostics.diagnostic(Severity::Error)
                                    .with_message("invalid boundary constraint")
                                    .with_primary_label(lhs_span, format!("this has a constraint domain of {lhs_domain}"))
                                    .with_secondary_label(SourceSpan::UNKNOWN, format!("this has a constraint domain of {rhs_domain}"))
                                    .with_note("Boundary constraints require both sides of the constraint to be in the same domain.")
                                    .emit();
                        return Err(CompileError::Failed);
                    }
                }

                // Merge the expressions into a single constraint
                let root = self.insert_op(Operation::Sub(lhs, rhs));

                // Store the generated constraint
                self.air
                    .constraints
                    .insert_constraint(trace_access.segment, root, domain);
                return Ok(());
            }
            _ => unreachable!(),
        }
    }

    fn build_integrity_constraint(&mut self, ic: &Link<Op>) -> Result<(), CompileError> {
        match ic.borrow().deref() {
            Op::Vector(vector) => {
                let vec = vector.children().borrow().deref().clone();
                for node in vec.iter() {
                    self.build_integrity_constraint(node)?;
                }
            }
            Op::Matrix(matrix) => {
                let rows = matrix.elements.borrow().deref().clone();
                for row in rows.iter() {
                    let vec = row.borrow().deref().children().borrow().deref().clone();
                    for node in vec.iter() {
                        self.build_integrity_constraint(node)?;
                    }
                }
            }
            Op::Enf(enf) => {
                let child_op = enf.expr.clone();
                match child_op.clone().borrow().deref() {
                    Op::Sub(_sub) => {
                        self.build_integrity_constraint(&child_op)?;
                    }
                    Op::If(if_node) => {
                        let cond = if_node.condition.clone();
                        let then_branch = if_node.then_branch.clone();
                        let else_branch = if_node.else_branch.clone();
                        let cond_node_index = self.insert_mir_operation(&cond);
                        let then_node_index = self.insert_mir_operation(&then_branch);
                        let else_node_index = self.insert_mir_operation(&else_branch);

                        let pos_root =
                            self.insert_op(Operation::Mul(then_node_index, cond_node_index));
                        let one = self.insert_op(Operation::Value(crate::ir::Value::Constant(1)));
                        let neg_cond = self.insert_op(Operation::Sub(one, cond_node_index));
                        let neg_root = self.insert_op(Operation::Mul(else_node_index, neg_cond));

                        let (trace_segment, domain) = self
                            .air
                            .constraint_graph()
                            .node_details(&pos_root, ConstraintDomain::EveryRow)?;
                        self.air
                            .constraints
                            .insert_constraint(trace_segment, pos_root, domain);
                        let (trace_segment, domain) = self
                            .air
                            .constraint_graph()
                            .node_details(&neg_root, ConstraintDomain::EveryRow)?;
                        self.air
                            .constraints
                            .insert_constraint(trace_segment, neg_root, domain);
                    }
                    _ => unreachable!(),
                }
            }
            Op::Sub(sub) => {
                let lhs = sub.lhs.clone();
                let rhs = sub.rhs.clone();
                let lhs_node_index = self.insert_mir_operation(&lhs);
                let rhs_node_index = self.insert_mir_operation(&rhs);
                let root = self.insert_op(Operation::Sub(lhs_node_index, rhs_node_index));
                let (trace_segment, domain) = self
                    .air
                    .constraint_graph()
                    .node_details(&root, ConstraintDomain::EveryRow)?;
                self.air
                    .constraints
                    .insert_constraint(trace_segment, root, domain);
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    /// Adds the specified operation to the graph and returns the index of its node.
    #[inline]
    fn insert_op(&mut self, op: Operation) -> NodeIndex {
        self.air.constraint_graph_mut().insert_node(op)
    }
}
