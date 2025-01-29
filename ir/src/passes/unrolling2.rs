use core::panic;
use std::{collections::HashMap, ops::Deref, rc::Rc};

use air_parser::ast::AccessType;
use air_pass::Pass;
use miden_diagnostics::DiagnosticsHandler;
//use miden_diagnostics::DiagnosticsHandler;

use crate::{ir::*, CompileError};

use super::{duplicate_node_or_replace, visitor2::Visitor};

/// This pass follows a similar approach as the Inlining pass.
/// It requires that this Inlining pass has already been done.
///
/// * In the first step, we visit the graph, unrolling each node type except For nodes.
///   Instead, for these node types we gather the context to inline them in the second pass.
/// * In the second pass, we inline the bodies of For nodes.
///
/// TODO:
/// - [ ] Implement diagnostics for better error handling

#[derive(Clone, Debug)]
pub struct ForInliningContext {
    body: Link<Op>,
    iterators: Vec<Link<Op>>,
    selector: Option<Link<Op>>,
    ref_node: Link<Op>,
}

impl ForInliningContext {}

pub struct Unrolling<'a> {
    diagnostics: &'a DiagnosticsHandler,
}

impl<'a> Unrolling<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}

pub struct UnrollingFirstPass<'a> {
    diagnostics: &'a DiagnosticsHandler,

    // general context
    work_stack: Vec<Link<Node>>,

    bodies_to_inline: Vec<(Link<Op>, ForInliningContext)>,
}

impl<'a> UnrollingFirstPass<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self {
            diagnostics,
            work_stack: vec![],
            bodies_to_inline: vec![],
        }
    }
}

pub struct UnrollingSecondPass<'a> {
    diagnostics: &'a DiagnosticsHandler,

    // general context
    work_stack: Vec<Link<Node>>,

    bodies_to_inline: Vec<(Link<Op>, ForInliningContext)>,
    for_inlining_context: Option<ForInliningContext>,
    nodes_to_replace: HashMap<usize, (Link<Op>, Link<Op>)>,
}
impl<'a> UnrollingSecondPass<'a> {
    pub fn new(
        diagnostics: &'a DiagnosticsHandler,
        bodies_to_inline: Vec<(Link<Op>, ForInliningContext)>,
    ) -> Self {
        Self {
            diagnostics,
            work_stack: vec![],
            bodies_to_inline,
            for_inlining_context: None,
            nodes_to_replace: HashMap::new(),
        }
    }
}

impl Pass for Unrolling<'_> {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        /*let graph = ir.constraint_graph();
        let functions = graph.get_function_nodes();
        let evaluators = graph.get_evaluator_nodes();
        let bc = graph.boundary_constraints_roots.borrow().deref().clone();
        let ic = graph.integrity_constraints_roots.borrow().deref().clone();

        println!("Before Unrolling pass");
        println!();
        for fns in functions {
            println!("fns: {:?}", fns);
        }
        println!();
        for evs in evaluators {
            println!("evs: {:?}", evs);
        }
        println!();

        for bc in bc {
            println!("bc: {:?}", bc);
        }
        println!();
        for ic in ic {
            println!("ic: {:?}", ic);
        }
        println!();*/

        /*println!("****************************");
        println!("Starting first UNROLLING pass");
        println!("****************************");*/

        // The first pass unrolls all nodes fully, except for For nodes
        let mut first_pass = UnrollingFirstPass::new(self.diagnostics);
        Visitor::run(&mut first_pass, ir.constraint_graph_mut())?;

        /*println!(
            "first_pass.bodies_to_inline.clone(): {:?}",
            first_pass.bodies_to_inline.clone()
        );*/

        /*println!("****************************");
        println!("Starting second UNROLLING pass");
        println!("****************************");*/

        // The second pass actually inlines the For nodes
        let mut second_pass =
            UnrollingSecondPass::new(self.diagnostics, first_pass.bodies_to_inline.clone());
        Visitor::run(&mut second_pass, ir.constraint_graph_mut())?;

        /*let graph = ir.constraint_graph();
        let functions = graph.get_function_nodes();
        let evaluators = graph.get_evaluator_nodes();
        let bc = graph.boundary_constraints_roots.borrow().deref().clone();
        let ic = graph.integrity_constraints_roots.borrow().deref().clone();

        println!();
        println!();
        println!("After Unrolling pass");
        println!();
        for fns in functions {
            println!("fns: {:?}", fns);
        }
        println!();
        for evs in evaluators {
            println!("evs: {:?}", evs);
        }
        println!();

        for bc in bc {
            println!("bc: {:?}", bc);
        }
        println!();
        for ic in ic {
            println!("ic: {:?}", ic);
        }
        println!();*/

        Ok(ir)
    }
}

impl<'a> UnrollingFirstPass<'a> {
    fn visit_value_bis(
        &mut self,
        _graph: &mut Graph,
        value: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // safe to un wrap because we just dispatched on it
        let mut updated_value = None;

        {
            let value_ref = value.as_value().unwrap();
            let mir_value = value_ref.value.value.clone();
            match mir_value {
                MirValue::Constant(c) => match c {
                    ConstantValue::Felt(_) => {}
                    ConstantValue::Vector(v) => {
                        let mut vec = vec![];
                        for val in v {
                            let val = Value::create(SpannedMirValue {
                                span: value_ref.value.span,
                                value: MirValue::Constant(ConstantValue::Felt(val)),
                            });
                            vec.push(val);
                        }
                        updated_value = Some(Vector::create(vec));
                    }
                    ConstantValue::Matrix(m) => {
                        let mut res_m = vec![];
                        for row in m {
                            let mut res_row = vec![];
                            for val in row {
                                let val = Value::create(SpannedMirValue {
                                    span: value_ref.value.span,
                                    value: MirValue::Constant(ConstantValue::Felt(val)),
                                });
                                res_row.push(val);
                            }
                            let res_row_vec = Vector::create(res_row);
                            res_m.push(res_row_vec);
                        }
                        updated_value = Some(Matrix::create(res_m));
                    }
                },
                MirValue::TraceAccess(_) => {}
                MirValue::PeriodicColumn(_) => {}
                MirValue::PublicInput(_) => {}
                MirValue::RandomValue(_) => {}
                MirValue::TraceAccessBinding(trace_access_binding) => {
                    // Create Trace Access based on this binding
                    if trace_access_binding.size == 1 {
                        let val = Value::create(SpannedMirValue {
                            span: value_ref.value.span,
                            value: MirValue::TraceAccess(TraceAccess {
                                segment: trace_access_binding.segment,
                                column: trace_access_binding.offset,
                                row_offset: 0, // ???
                            }),
                        });
                        updated_value = Some(val);
                    } else {
                        let mut vec = vec![];
                        for index in 0..trace_access_binding.size {
                            let val = Value::create(SpannedMirValue {
                                span: value_ref.value.span,
                                value: MirValue::TraceAccess(TraceAccess {
                                    segment: trace_access_binding.segment,
                                    column: trace_access_binding.offset + index,
                                    row_offset: 0, // ???
                                }),
                            });
                            vec.push(val);
                        }
                        updated_value = Some(Vector::create(vec));
                    }
                }
                MirValue::RandomValueBinding(random_value_binding) => {
                    let mut vec = vec![];
                    for index in 0..random_value_binding.size {
                        let val = Value::create(SpannedMirValue {
                            span: value_ref.value.span,
                            value: MirValue::RandomValue(random_value_binding.offset + index),
                        });
                        vec.push(val);
                    }

                    updated_value = Some(Vector::create(vec));
                }
            }
        }

        Ok(updated_value)
    }

    fn visit_add_bis(
        &mut self,
        _graph: &mut Graph,
        add: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // safe to un wrap because we just dispatched on it

        let mut updated_add = None;

        {
            let add_ref = add.as_add().unwrap();
            let lhs = add_ref.lhs.clone();
            let rhs = add_ref.rhs.clone();

            if let (Op::Vector(lhs_vector), Op::Vector(rhs_vector)) =
                (lhs.borrow().deref(), rhs.borrow().deref())
            {
                let lhs_vec = lhs_vector.children().borrow().deref().clone();
                let rhs_vec = rhs_vector.children().borrow().deref().clone();

                if lhs_vec.len() != rhs_vec.len() {
                    // Raise diag
                    todo!();
                } else {
                    let mut new_vec = vec![];
                    for (lhs, rhs) in lhs_vec.iter().zip(rhs_vec.iter()) {
                        let new_node = Add::create(lhs.clone(), rhs.clone());
                        new_vec.push(new_node);
                    }
                    updated_add = Some(Vector::create(new_vec));
                }
            };
        }

        Ok(updated_add)
    }

    fn visit_sub_bis(
        &mut self,
        _graph: &mut Graph,
        sub: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // safe to unwrap because we just dispatched on it

        let mut updated_sub = None;

        let sub_ref = sub.as_sub().unwrap();
        let lhs = sub_ref.lhs.clone();
        let rhs = sub_ref.rhs.clone();

        if let (Op::Vector(lhs_vector), Op::Vector(rhs_vector)) =
            (lhs.borrow().deref(), rhs.borrow().deref())
        {
            let lhs_vec = lhs_vector.children().borrow().deref().clone();
            let rhs_vec = rhs_vector.children().borrow().deref().clone();

            if lhs_vec.len() != rhs_vec.len() {
                // Raise diag
            } else {
                let mut new_vec = vec![];
                for (lhs, rhs) in lhs_vec.iter().zip(rhs_vec.iter()) {
                    let new_node = Sub::create(lhs.clone(), rhs.clone());
                    new_vec.push(new_node);
                }
                updated_sub = Some(Vector::create(new_vec));
            }
        };

        Ok(updated_sub)
    }

    fn visit_mul_bis(
        &mut self,
        _graph: &mut Graph,
        mul: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let mut updated_mul = None;

        {
            let mul_ref = mul.as_mul().unwrap();
            let lhs = mul_ref.lhs.clone();
            let rhs = mul_ref.rhs.clone();

            if let (Op::Vector(lhs_vector), Op::Vector(rhs_vector)) =
                (lhs.borrow().deref(), rhs.borrow().deref())
            {
                let lhs_vec = lhs_vector.children().borrow().deref().clone();
                let rhs_vec = rhs_vector.children().borrow().deref().clone();

                if lhs_vec.len() != rhs_vec.len() {
                    // Raise diag
                } else {
                    let mut new_vec = vec![];
                    for (lhs, rhs) in lhs_vec.iter().zip(rhs_vec.iter()) {
                        let new_node = Mul::create(lhs.clone(), rhs.clone());
                        new_vec.push(new_node);
                    }
                    updated_mul = Some(Vector::create(new_vec));
                }
            };
        }

        Ok(updated_mul)
    }

    fn visit_enf_bis(
        &mut self,
        _graph: &mut Graph,
        enf: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let mut updated_enf = None;

        {
            let enf_ref = enf.as_enf().unwrap();
            let expr = enf_ref.expr.clone();
            if let Op::Vector(vec) = expr.borrow().deref() {
                let ops = vec.children().borrow().deref().clone();
                let mut new_vec = vec![];
                for op in ops.iter() {
                    let new_node = Enf::create(op.clone());
                    new_vec.push(new_node);
                }
                updated_enf = Some(Vector::create(new_vec));
            };
        }

        Ok(updated_enf)
    }

    fn visit_fold_bis(
        &mut self,
        _graph: &mut Graph,
        fold: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let mut updated_fold = None;

        {
            let fold_ref = fold.as_fold().unwrap();
            let iterator = fold_ref.iterator.clone();
            let operator = fold_ref.operator.clone();
            let initial_value = fold_ref.initial_value.clone();

            let iterator_ref = iterator.borrow();
            let Op::Vector(iterator_vector) = iterator_ref.deref() else {
                unreachable!();
            };
            let iterator_nodes = iterator_vector.children().borrow().deref().clone();

            let mut acc_node = initial_value;
            match operator {
                FoldOperator::Add => {
                    for iterator_node in iterator_nodes {
                        let new_acc_node = Add::create(acc_node, iterator_node);
                        acc_node = new_acc_node;
                    }
                }
                FoldOperator::Mul => {
                    for iterator_node in iterator_nodes {
                        let new_acc_node = Mul::create(acc_node, iterator_node);
                        acc_node = new_acc_node;
                    }
                }
                FoldOperator::None => {}
            }
            updated_fold = Some(acc_node);
        }

        Ok(updated_fold)
    }

    fn visit_parameter_bis(
        &mut self,
        _graph: &mut Graph,
        _parameter: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        // FIXME: Just check that the parameter is a scalar, raise diag otherwise
        // List comprehension bodies should only be scalar expressions
        Ok(None)
    }

    /*fn visit_if_old(&mut self, _graph: &mut Graph, if_node: Link<Op>) -> Result<Option<Link<Op>>, CompileError>{
        let if_ref = if_node.as_if().unwrap();
        let condition = if_ref.condition.clone();
        let then_branch = if_ref.then_branch.clone();
        let else_branch = if_ref.else_branch.clone();

        if let (
            Op::Vector(condition_vector),
            Op::Vector(then_branch_vector),
            Op::Vector(else_branch_vector),
        ) = (
            condition.borrow().deref(),
            then_branch.borrow().deref(),
            else_branch.borrow().deref(),
        ) {
            let condition_vec = condition_vector.children().borrow().deref().clone();
            let then_branch_vec = then_branch_vector.children().borrow().deref().clone();
            let else_branch_vec = else_branch_vector.children().borrow().deref().clone();

            if condition_vec.len() != then_branch_vec.len()
                || condition_vec.len() != else_branch_vec.len()
            {
                // Raise diag
            } else {
                let mut new_vec = vec![];
                for ((condition, then_branch), else_branch) in condition_vec
                    .iter()
                    .zip(then_branch_vec.iter())
                    .zip(else_branch_vec.iter())
                {
                    let new_node =
                        If::create(condition.clone(), then_branch.clone(), else_branch.clone());
                    new_vec.push(new_node);
                }
                if_node.set(&Vector::create(new_vec));
            }
        };

        Ok(())
    }*/

    fn visit_if_bis(
        &mut self,
        _graph: &mut Graph,
        if_node: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let mut updated_if = None;

        {
            let if_ref = if_node.as_if().unwrap();
            let condition = if_ref.condition.clone();
            let then_branch = if_ref.then_branch.clone();
            let else_branch = if_ref.else_branch.clone();

            let mut new_vec = vec![];

            if let Op::Vector(then_branch_vector) = then_branch.clone().borrow().deref() {
                let then_branch_vec = then_branch_vector.children().borrow().deref().clone();

                for then_branch in then_branch_vec {
                    let new_node = Mul::create(condition.clone(), then_branch);
                    new_vec.push(new_node);
                }
            } else {
                let new_node = Mul::create(condition.clone(), then_branch);
                new_vec.push(new_node);
            }

            let one_constant = SpannedMirValue {
                span: Default::default(),
                value: MirValue::Constant(ConstantValue::Felt(1)),
            };

            if let Op::Vector(else_branch_vector) = else_branch.clone().borrow().deref() {
                let else_branch_vec = else_branch_vector.children().borrow().deref().clone();

                for else_branch in else_branch_vec {
                    let new_node = Mul::create(
                        Sub::create(Value::create(one_constant.clone()), condition.clone()),
                        else_branch,
                    );
                    new_vec.push(new_node);
                }
            } else {
                let new_node = Mul::create(
                    Sub::create(Value::create(one_constant.clone()), condition.clone()),
                    else_branch,
                );
                new_vec.push(new_node);
            }

            updated_if = Some(Vector::create(new_vec));
        }

        Ok(updated_if)
    }

    fn visit_boundary_bis(
        &mut self,
        _graph: &mut Graph,
        boundary: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
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
                    let new_node = Boundary::create(expr.clone(), kind);
                    new_vec.push(new_node);
                }
                updated_boundary = Some(Vector::create(new_vec));
            };
        }

        Ok(updated_boundary)
    }

    fn visit_accessor_bis(
        &mut self,
        _graph: &mut Graph,
        accessor: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let mut updated_accessor = None;

        {
            let accessor_ref = accessor.as_accessor().unwrap();
            let indexable = accessor_ref.indexable.clone();
            let access_type = accessor_ref.access_type.clone();
            match access_type {
                AccessType::Default => {
                    // Check that the child node is a scalar, raise diag otherwise
                    if indexable.clone().as_vector().is_some() {
                        unreachable!(); // raise diag
                    }
                    if indexable.clone().as_matrix().is_some() {
                        unreachable!(); // raise diag
                    }
                }
                AccessType::Index(index) => {
                    // Check that the child node is a vector, raise diag otherwise
                    // Replace the current node by the index-th element of the vector
                    // Raise diag if index is out of bounds

                    if let Op::Vector(indexable_vector) = indexable.borrow().deref() {
                        let indexable_vec = indexable_vector.children().borrow().deref().clone();
                        let child_accessed = match indexable_vec.get(index) {
                            Some(child_accessed) => child_accessed,
                            None => unreachable!(), // raise diag
                        };
                        updated_accessor = Some(child_accessed.clone());
                    } else {
                        unreachable!(); // raise diag
                    };
                }
                AccessType::Matrix(row, col) => {
                    // Check that the child node is a matrix, raise diag otherwise
                    // Replace the current node by the index-th element of the vector
                    // Raise diag if index is out of bounds

                    if let Op::Vector(indexable_vector) = indexable.borrow().deref() {
                        let indexable_vec = indexable_vector.children().borrow().deref().clone();
                        let row_accessed = match indexable_vec.get(row) {
                            Some(row_accessed) => row_accessed,
                            None => unreachable!(), // raise diag
                        };

                        if let Op::Vector(row_accessed_vector) = row_accessed.borrow().deref() {
                            let row_accessed_vec =
                                row_accessed_vector.children().borrow().deref().clone();
                            let child_accessed = match row_accessed_vec.get(col) {
                                Some(child_accessed) => child_accessed,
                                None => unreachable!(), // raise diag
                            };
                            updated_accessor = Some(child_accessed.clone());
                        } else {
                            unreachable!(); // raise diag
                        };
                    } else {
                        if let Op::Matrix(indexable_matrix) = indexable.borrow().deref() {
                            let indexable_vec =
                                indexable_matrix.children().borrow().deref().clone();
                            let row_accessed = match indexable_vec.get(row) {
                                Some(row_accessed) => row_accessed,
                                None => unreachable!(), // raise diag
                            };

                            if let Op::Vector(row_accessed_vector) = row_accessed.borrow().deref() {
                                let row_accessed_vec =
                                    row_accessed_vector.children().borrow().deref().clone();
                                let child_accessed = match row_accessed_vec.get(col) {
                                    Some(child_accessed) => child_accessed,
                                    None => unreachable!(), // raise diag
                                };
                                updated_accessor = Some(child_accessed.clone());
                            } else {
                                unreachable!(); // raise diag
                            };
                        }
                    };
                }

                AccessType::Slice(_range_expr) => {
                    unreachable!(); // Slices are not scalar, raise diag
                }
            }
        }

        Ok(updated_accessor)
    }

    fn visit_for_bis(
        &mut self,
        _graph: &mut Graph,
        for_node: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        let mut updated_for = None;

        {
            // For each value produced by the iterators, we need to:
            // - Duplicate the body
            // - Visit the body and replace the Variables with the value (with the correct index depending on the binding)
            // If there is a selector, we need to enforce the selector on the body through an if node ?

            let for_node_clone = for_node.clone();
            let for_ref = for_node_clone.as_for().unwrap();
            let iterators_ref = for_ref.iterators.borrow();
            let iterators = iterators_ref.deref();
            let expr = for_ref.expr.clone();
            let selector = for_ref.selector.clone();

            // Check iterator lengths
            if iterators.is_empty() {
                unreachable!(); // Raise diag
            }
            let iterator_expected_len = iterators[0]
                .clone()
                .as_vector()
                .unwrap_or_else(|| {
                    unreachable!("Iterators should be vectors, got {:?}", iterators[0])
                })
                .children()
                .borrow()
                .len();

            for iterator in iterators.iter().skip(1) {
                if iterator
                    .clone()
                    .as_vector()
                    .unwrap_or_else(|| {
                        unreachable!("Iterators should be vectors, got {:?}", iterator)
                    })
                    .children()
                    .borrow()
                    .len()
                    != iterator_expected_len
                {
                    unreachable!(); // Raise diag
                }
            }

            let mut new_vec = vec![];
            for i in 0..iterator_expected_len {
                let new_node = Parameter::create(i, MirType::Felt);
                new_vec.push(new_node.clone());

                let iterators_i = iterators
                    .iter()
                    .map(|op| match op.clone().as_vector() {
                        Some(vec) => vec.children().borrow()[i].clone(),
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>();
                let selector = if let Op::None = selector.borrow().deref() {
                    None
                } else {
                    Some(selector.clone())
                };

                self.bodies_to_inline.push((
                    new_node.clone(),
                    ForInliningContext {
                        body: expr.clone(),
                        iterators: iterators_i,
                        selector,
                        ref_node: for_node.clone(),
                    },
                ));
            }

            let new_vec_op = Vector::create(new_vec.clone());
            for param in new_vec {
                param
                    .as_parameter_mut()
                    .unwrap()
                    .set_ref_node(new_vec_op.as_owner().unwrap());
            }
            updated_for = Some(new_vec_op);
        }

        Ok(updated_for)
    }

    fn visit_call_bis(
        &mut self,
        _graph: &mut Graph,
        _call: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        unreachable!("Calls should have been inlined before this pass");
    }

    fn visit_function_bis(
        &mut self,
        _graph: &mut Graph,
        _function: Link<Root>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        unreachable!("Functions should have been inlined before this pass");
    }

    fn visit_evaluator_bis(
        &mut self,
        _graph: &mut Graph,
        _evaluator: Link<Root>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        unreachable!("Evaluators should have been inlined before this pass");
    }
    fn visit_vector_bis(
        &mut self,
        _graph: &mut Graph,
        _vector: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        /*let mut updated_vector = None;

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

        Ok(updated_vector)*/
        Ok(None)
    }
    fn visit_matrix_bis(
        &mut self,
        _graph: &mut Graph,
        _matrix: Link<Op>,
    ) -> Result<Option<Link<Op>>, CompileError> {
        Ok(None)
    }
}

impl Visitor for UnrollingFirstPass<'_> {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }
    fn root_nodes_to_visit(&self, graph: &Graph) -> Vec<Link<Node>> {
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

    fn visit_node(&mut self, graph: &mut Graph, node: Link<Node>) -> Result<(), CompileError> {
        let updated_op = match node.borrow().deref() {
            Node::Function(f) => self.visit_function_bis(graph, f.clone().into()),
            Node::Evaluator(e) => self.visit_evaluator_bis(graph, e.clone().into()),
            Node::Enf(e) => self.visit_enf_bis(graph, e.clone().into()),
            Node::Boundary(b) => self.visit_boundary_bis(graph, b.clone().into()),
            Node::Add(a) => self.visit_add_bis(graph, a.clone().into()),
            Node::Sub(s) => self.visit_sub_bis(graph, s.clone().into()),
            Node::Mul(m) => self.visit_mul_bis(graph, m.clone().into()),
            Node::If(i) => self.visit_if_bis(graph, i.clone().into()),
            Node::For(f) => self.visit_for_bis(graph, f.clone().into()),
            Node::Call(c) => self.visit_call_bis(graph, c.clone().into()),
            Node::Fold(f) => self.visit_fold_bis(graph, f.clone().into()),
            Node::Vector(v) => self.visit_vector_bis(graph, v.clone().into()),
            Node::Matrix(m) => self.visit_matrix_bis(graph, m.clone().into()),
            Node::Accessor(a) => self.visit_accessor_bis(graph, a.clone().into()),
            Node::Parameter(p) => self.visit_parameter_bis(graph, p.clone().into()),
            Node::Value(v) => self.visit_value_bis(graph, v.clone().into()),
            Node::None => Ok(None),
        };

        if let Some(updated_op) = updated_op? {
            node.as_op().unwrap().set(&updated_op);
        }

        Ok(())
    }
}

impl Visitor for UnrollingSecondPass<'_> {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }
    fn run(&mut self, graph: &mut Graph) -> Result<(), CompileError> {
        for root in self.root_nodes_to_visit(graph).iter() {
            /*println!("Visiting root node: {idx} - {:?}", root);
            println!("");*/

            // Set context to inline the body for this index
            let for_inlining_context = self.bodies_to_inline.iter().find_map(|(node, context)| {
                if Rc::ptr_eq(&node.clone().as_node().link, &root.link) {
                    Some(context.clone())
                } else {
                    None
                }
            });

            //println!("SET NEW CONTEXT: {:?}", for_inlining_context);

            self.for_inlining_context = for_inlining_context;
            self.nodes_to_replace.clear();

            self.scan_node(
                graph,
                self.for_inlining_context.clone().unwrap().body.as_node(),
            )?;

            let mut ind = 0;
            while let Some(node) = self.work_stack().pop() {
                ind += 1;
                self.visit_node(graph, node)?;
                if ind > 500 {
                    unreachable!("UnrollingSecondPass::run: too many iterations");
                }
            }

            //println!("END Visiting root node: {idx} - {:?}", root);

            // We have finished inlining the body, we can now replace the Root node with the body

            let body = self.for_inlining_context.clone().unwrap().body;
            let new_node = self
                .nodes_to_replace
                .get(&body.get_ptr())
                .unwrap()
                .1
                .clone();

            let new_node_with_selector_if_needed =
                if let Some(selector) = self.for_inlining_context.clone().unwrap().selector {
                    let zero_node = Value::create(SpannedMirValue {
                        span: Default::default(),
                        value: MirValue::Constant(ConstantValue::Felt(0)),
                    });
                    Sub::create(Mul::create(selector, new_node), zero_node)
                } else {
                    new_node
                };

            root.as_op().unwrap().set(&new_node_with_selector_if_needed);
            /*println!("");
            println!("Updated child of For node: {:?}", root);
            println!("");*/

            // Reset context to None
            self.for_inlining_context = None;
        }

        Ok(())
    }
    fn root_nodes_to_visit(&self, _graph: &Graph) -> Vec<Link<Node>> {
        self.bodies_to_inline
            .iter()
            .map(|(k, _v)| k)
            .cloned()
            .map(|op| op.as_node())
            .collect::<Vec<_>>()
    }
    fn visit_node(&mut self, _graph: &mut Graph, node: Link<Node>) -> Result<(), CompileError> {
        if node.is_stale() {
            return Ok(());
        }
        if let Some(op) = node.clone().as_op() {
            duplicate_node_or_replace(
                &mut self.nodes_to_replace,
                op,
                self.for_inlining_context.clone().unwrap().iterators.clone(),
                self.for_inlining_context
                    .clone()
                    .unwrap()
                    .ref_node
                    .as_node(),
            );
        } else {
            unreachable!(
                "UnrollingSecondPass::visit_node on a non-Op node: {:?}",
                node
            );
        }
        Ok(())
    }
}
