use std::ops::Deref;

use air_parser::ast::AccessType;
use miden_diagnostics::{SourceSpan, Spanned};

use crate::{
    CompileError,
    ir::{
        Add, Boundary, ConstantValue, Enf, Exp, FoldOperator, Link, Matrix, MirValue, Mul, Op,
        Parent, SpannedMirValue, Sub, TraceAccess, TraceAccessBinding, Value, Vector,
    },
};

/// Unrolls a `Value` node
pub fn unroll_value(value: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    // safe to unwrap because we just dispatched on it
    let mut updated_value = None;
    {
        let value_ref = value.as_value().unwrap();
        let mir_value = value_ref.value.value.clone();
        match &mir_value {
            MirValue::Constant(c) => match c {
                ConstantValue::Felt(_) => {},
                ConstantValue::Vector(v) => {
                    updated_value = Some(unroll_constant_vector(v, value_ref.span()));
                },
                ConstantValue::Matrix(m) => {
                    updated_value = Some(unroll_constant_matrix(m, value_ref.span()));
                },
            },
            MirValue::TraceAccessBinding(trace_access_binding) => {
                updated_value =
                    Some(unroll_trace_access_binding(trace_access_binding, value_ref.span()));
            },
            MirValue::TraceAccess(_)
            | MirValue::PeriodicColumn(_)
            | MirValue::PublicInput(_)
            | MirValue::PublicInputTable(_)
            | MirValue::RandomValue(_)
            | MirValue::BusAccess(_)
            | MirValue::Null
            | MirValue::Unconstrained => {},
        }
    }
    Ok(updated_value)
}

/// Unrolls an `Add` node.
pub fn unroll_add(add: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    // safe to unwrap because we just dispatched on it
    let add_ref = add.as_add().unwrap();
    let lhs = add_ref.lhs.clone();
    let rhs = add_ref.rhs.clone();
    unroll_binary_op(lhs, rhs, add.clone(), add_ref.span())
}

/// Unrolls a `Sub` node.
pub fn unroll_sub(sub: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    // safe to unwrap because we just dispatched on it
    let sub_ref = sub.as_sub().unwrap();
    let lhs = sub_ref.lhs.clone();
    let rhs = sub_ref.rhs.clone();
    unroll_binary_op(lhs, rhs, sub.clone(), sub_ref.span())
}

/// Unrolls a `Mul` node.
pub fn unroll_mul(mul: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    // safe to unwrap because we just dispatched on it
    let mul_ref = mul.as_mul().unwrap();
    let lhs = mul_ref.lhs.clone();
    let rhs = mul_ref.rhs.clone();
    unroll_binary_op(lhs, rhs, mul.clone(), mul_ref.span())
}

/// Unrolls an `Exp` node.
pub fn unroll_exp(exp: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    // safe to unwrap because we just dispatched on it
    let exp_ref = exp.as_exp().unwrap();
    let lhs = exp_ref.lhs.clone();
    let rhs = exp_ref.rhs.clone();
    unroll_binary_op(lhs, rhs, exp.clone(), exp_ref.span())
}

/// Unrolls an `Enf` node.
pub fn unroll_enf(enf: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    let mut updated_enf = None;
    {
        let enf_ref = enf.as_enf().unwrap();
        let expr = enf_ref.expr.clone();
        if let Op::Vector(vec) = expr.borrow().deref() {
            let ops = vec.children().borrow().deref().clone();
            let mut new_vec = vec![];
            for op in ops.iter() {
                let new_node = Enf::create(op.clone(), enf_ref.span());
                new_vec.push(new_node);
            }
            updated_enf = Some(Vector::create(new_vec, enf_ref.span()));
        };
    }
    Ok(updated_enf)
}

/// Unrolls a `Boundary` node.
pub fn unroll_boundary(boundary: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    let mut updated_boundary = None;
    {
        // safe to unwrap because we just dispatched on it
        let boundary_ref = boundary.as_boundary().unwrap();
        let expr = boundary_ref.expr.clone();
        let kind = boundary_ref.kind;
        if let Op::Vector(vec) = expr.borrow().deref() {
            let expr_vec = vec.children().borrow().deref().clone();
            let mut new_vec = vec![];
            for expr in expr_vec.iter() {
                let new_node = Boundary::create(expr.clone(), kind, boundary_ref.span());
                new_vec.push(new_node);
            }
            updated_boundary = Some(Vector::create(new_vec, boundary_ref.span()));
        };
    }
    Ok(updated_boundary)
}

/// Unrolls an `Accessor` node.
pub fn unroll_accessor(accessor: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    let mut updated_accessor = None;
    {
        let accessor_ref = accessor.as_accessor().unwrap();
        let indexable = accessor_ref.indexable.clone();
        let access_type = accessor_ref.access_type.clone();
        let offset = accessor_ref.offset;
        // If the indexable is a parameter, we keep the accessor as is, in
        // order to handle nested For nodes
        if indexable.clone().as_parameter().is_none() {
            match access_type {
                AccessType::Default => {
                    updated_accessor = unroll_accessor_default_access_type(indexable, offset);
                },
                AccessType::Index(index) => {
                    updated_accessor = unroll_accessor_index_access_type(indexable, index, offset);
                },
                AccessType::Matrix(row, col) => {
                    updated_accessor = unroll_accessor_matrix_access_type(indexable, row, col);
                },
                AccessType::Slice(_range_expr) => {
                    unreachable!(); // Slices are not scalar, raise diag
                },
            }
        }
    }
    Ok(updated_accessor)
}

/// Unrolls a `Fold` node.
pub fn unroll_fold(fold: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    let updated_fold;
    {
        let fold_ref = fold.as_fold().unwrap();
        let iterator = fold_ref.iterator.clone();
        let operator = fold_ref.operator.clone();
        let initial_value = fold_ref.initial_value.clone();
        let iterator_ref = iterator.borrow();
        let Op::Vector(iterator_vector) = iterator_ref.deref() else {
            unreachable!("Expected vector iterator in fold, found: {:?}", iterator_ref);
        };
        let iterator_nodes = iterator_vector.children().borrow().deref().clone();
        let mut acc_node = initial_value;
        for iterator_node in iterator_nodes {
            let new_acc_node = match operator {
                FoldOperator::Add => Add::create(acc_node, iterator_node, fold_ref.span()),
                FoldOperator::Mul => Mul::create(acc_node, iterator_node, fold_ref.span()),
                FoldOperator::None => {
                    unreachable!("Unexpected unrolling of Fold with None FoldOperator")
                },
            };
            acc_node = new_acc_node;
        }
        updated_fold = Some(acc_node);
    }
    Ok(updated_fold)
}

/// Unrolls a `Vector` node.
pub fn unroll_vector(vector: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    let mut updated_vector = None;
    {
        // safe to unwrap because we just dispatched on it
        let vector_ref = vector.as_vector().unwrap();
        let children = vector_ref.elements.borrow().deref().clone();
        let size = vector_ref.size;
        if size == 1 {
            let child = children.first().unwrap();
            updated_vector = Some(child.clone());
        }
    }
    Ok(updated_vector)
}

/// Unrolls a `Matrix` node.
pub fn unroll_matrix(_matrix: Link<Op>) -> Result<Option<Link<Op>>, CompileError> {
    Ok(None) // Matrix are already unrolled, we have nothing to do
}

// HELPERS FUNCTIONS
// ================================================================================================

/// Unrolls a trace access binding into either:
/// - a `TraceAccess` if it is of size 1
/// - or a `Vector<TraceAccess>` otherwise.
fn unroll_trace_access_binding(
    trace_access_binding: &TraceAccessBinding,
    span: SourceSpan,
) -> Link<Op> {
    if trace_access_binding.size == 1 {
        Value::create(SpannedMirValue {
            span,
            value: MirValue::TraceAccess(TraceAccess {
                segment: trace_access_binding.segment,
                column: trace_access_binding.offset,
                row_offset: 0,
            }),
        })
    } else {
        let mut vec = vec![];
        for index in 0..trace_access_binding.size {
            let val = Value::create(SpannedMirValue {
                span,
                value: MirValue::TraceAccess(TraceAccess {
                    segment: trace_access_binding.segment,
                    column: trace_access_binding.offset + index,
                    row_offset: 0,
                }),
            });
            vec.push(val);
        }
        Vector::create(vec, span)
    }
}

/// Unrolls a constant vector into a `Vector<ConstantValue::Felt>`
fn unroll_constant_vector(constant_vector: &Vec<u64>, span: SourceSpan) -> Link<Op> {
    let mut vec = vec![];
    for val in constant_vector {
        let val = Value::create(SpannedMirValue {
            span,
            value: MirValue::Constant(ConstantValue::Felt(*val)),
        });
        vec.push(val);
    }
    Vector::create(vec, span)
}

/// Unrolls a constant matrix into a `Matrix<Vector<ConstantValue::Felt>>`
fn unroll_constant_matrix(constant_matrix: &Vec<Vec<u64>>, span: SourceSpan) -> Link<Op> {
    let mut res_m = vec![];
    for row in constant_matrix {
        let mut res_row = vec![];
        for val in row {
            let val = Value::create(SpannedMirValue {
                span,
                value: MirValue::Constant(ConstantValue::Felt(*val)),
            });
            res_row.push(val);
        }
        let res_row_vec = Vector::create(res_row, span);
        res_m.push(res_row_vec);
    }
    Matrix::create(res_m, span)
}

/// Unrolls a binary operation (to be dispatched from `Add`, `Sub`, `Mul`, `Exp`)
fn unroll_binary_op(
    lhs: Link<Op>,
    rhs: Link<Op>,
    parent: Link<Op>,
    span: SourceSpan,
) -> Result<Option<Link<Op>>, CompileError> {
    let mut updated_binary_op = None;

    if let (Op::Vector(lhs_vector), Op::Vector(rhs_vector)) =
        (lhs.borrow().deref(), rhs.borrow().deref())
    {
        let lhs_vec = lhs_vector.children().borrow().deref().clone();
        let rhs_vec = rhs_vector.children().borrow().deref().clone();

        if lhs_vec.len() != rhs_vec.len() {
            unreachable!("Binary operation children type mismatch: {:?}", parent);
        } else {
            let mut new_vec = vec![];
            for (lhs, rhs) in lhs_vec.iter().zip(rhs_vec.iter()) {
                let new_node = match parent.borrow().deref() {
                    Op::Add(_) => Add::create(lhs.clone(), rhs.clone(), span),
                    Op::Sub(_) => Sub::create(lhs.clone(), rhs.clone(), span),
                    Op::Mul(_) => Mul::create(lhs.clone(), rhs.clone(), span),
                    Op::Exp(_) => Exp::create(lhs.clone(), rhs.clone(), span),
                    _ => unreachable!("Unexpected parent operation: {:?}", parent),
                };
                new_vec.push(new_node);
            }
            updated_binary_op = Some(Vector::create(new_vec, parent.span()));
        }
    }

    Ok(updated_binary_op)
}

/// Helper function to handle the `AccessType::Default` case during accessor unrolling
fn unroll_accessor_default_access_type(
    indexable: Link<Op>,
    accessor_offset: usize,
) -> Option<Link<Op>> {
    let mut updated_accessor = Some(indexable.clone());

    if let Some(value) = indexable.clone().as_value() {
        let mir_value = value.value.value.clone();

        if let MirValue::TraceAccess(trace_access) = mir_value {
            let new_node = Value::create(SpannedMirValue {
                span: value.value.span(),
                value: MirValue::TraceAccess(TraceAccess {
                    segment: trace_access.segment,
                    column: trace_access.column,
                    row_offset: trace_access.row_offset + accessor_offset,
                }),
            });
            updated_accessor = Some(new_node);
        }
    }
    updated_accessor
}

/// Helper function to handle the `AccessType::Index(index)` case during accessor unrolling
fn unroll_accessor_index_access_type(
    indexable: Link<Op>,
    index: usize,
    accessor_offset: usize,
) -> Option<Link<Op>> {
    let updated_accessor;

    // Check that the child node is a vector, raise diag otherwise
    // Replace the current node by the index-th element of the vector
    // Raise diag if index is out of bounds
    if let Op::Vector(indexable_vector) = indexable.borrow().deref() {
        let indexable_vec = indexable_vector.children().borrow().deref().clone();
        let child_accessed = match indexable_vec.get(index) {
            Some(child_accessed) => child_accessed,
            None => unreachable!(), // raise diag
        };
        if let Some(value) = child_accessed.clone().as_value() {
            let mir_value = value.value.value.clone();
            match mir_value {
                MirValue::TraceAccess(trace_access) => {
                    let new_node = Value::create(SpannedMirValue {
                        span: value.value.span(),
                        value: MirValue::TraceAccess(TraceAccess {
                            segment: trace_access.segment,
                            column: trace_access.column,
                            row_offset: trace_access.row_offset + accessor_offset,
                        }),
                    });
                    updated_accessor = Some(new_node);
                },
                _ => {
                    updated_accessor = Some(child_accessed.clone());
                },
            }
        } else {
            updated_accessor = Some(child_accessed.clone());
        }
    } else {
        unreachable!("indexable is {:?}", indexable); // raise diag
    };
    updated_accessor
}

/// Helper function to handle the `AccessType::Matrix(row, col)` case during accessor unrolling
fn unroll_accessor_matrix_access_type(
    indexable: Link<Op>,
    row: usize,
    col: usize,
) -> Option<Link<Op>> {
    // Check that the child node is a matrix, raise diag otherwise
    // Replace the current node by the index-th element of the vector
    // Raise diag if index is out of bounds
    if let Op::Vector(indexable_vector) = indexable.borrow().deref() {
        let indexable_vec = indexable_vector.children().borrow().deref().clone();
        let row_accessed = match indexable_vec.get(row) {
            Some(row_accessed) => row_accessed,
            None => unreachable!("Matrix access out of bounds for indexable: {:?}", indexable),
        };
        if let Op::Vector(row_accessed_vector) = row_accessed.borrow().deref() {
            let row_accessed_vec = row_accessed_vector.children().borrow().deref().clone();
            let child_accessed = match row_accessed_vec.get(col) {
                Some(child_accessed) => child_accessed,
                None => unreachable!("Matrix access out of bounds for indexable: {:?}", indexable),
            };
            Some(child_accessed.clone())
        } else {
            unreachable!("unexpected non-vector child of a Matrix: {:?}", row_accessed);
        }
    } else if let Op::Matrix(indexable_matrix) = indexable.borrow().deref() {
        let indexable_vec = indexable_matrix.children().borrow().deref().clone();
        let row_accessed = match indexable_vec.get(row) {
            Some(row_accessed) => row_accessed,
            None => unreachable!("Matrix access out of bounds for indexable: {:?}", indexable),
        };
        if let Op::Vector(row_accessed_vector) = row_accessed.borrow().deref() {
            let row_accessed_vec = row_accessed_vector.children().borrow().deref().clone();
            let child_accessed = match row_accessed_vec.get(col) {
                Some(child_accessed) => child_accessed,
                None => unreachable!("Matrix access out of bounds for indexable: {:?}", indexable),
            };
            Some(child_accessed.clone())
        } else {
            unreachable!("unexpected non-vector child of a Matrix: {:?}", row_accessed);
        }
    } else {
        unreachable!("unexpected matrix access type on indexable {:?}", indexable);
    }
}
