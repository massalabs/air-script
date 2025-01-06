use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use air_parser::ast::AccessType;
use air_pass::Pass;
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

#[derive(Clone)]
pub struct ForInliningContext {
    body: Link<Op>,
    iterators: Vec<Link<Op>>,
    selector: Option<Link<Op>>,
    index: usize,
    parent_for: Link<Op>,
}

impl ForInliningContext {}

pub struct Unrolling {}
impl Unrolling {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct UnrollingFirstPass {
    // general context
    work_stack: Vec<Link<Node>>,

    bodies_to_inline: Vec<(Link<Op>, ForInliningContext)>,
}

impl UnrollingFirstPass {
    pub fn new() -> Self {
        Self {
            work_stack: vec![],
            bodies_to_inline: vec![],
        }
    }
}

pub struct UnrollingSecondPass {
    // general context
    work_stack: Vec<Link<Node>>,

    bodies_to_inline: Vec<(Link<Op>, ForInliningContext)>,
    for_inlining_context: Option<ForInliningContext>,
    nodes_to_replace: HashMap<Link<Op>, Link<Op>>,
}
impl UnrollingSecondPass {
    pub fn new(bodies_to_inline: Vec<(Link<Op>, ForInliningContext)>) -> Self {
        Self {
            work_stack: vec![],
            bodies_to_inline,
            for_inlining_context: None,
            nodes_to_replace: HashMap::new(),
        }
    }
}

impl Pass for Unrolling {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {

        let graph = ir.constraint_graph();
        let functions = graph.get_function_nodes();
        let evaluators = graph.get_evaluator_nodes();
        let bc = graph.boundary_constraints_roots.borrow().deref().clone();
        let ic = graph.integrity_constraints_roots.borrow().deref().clone();

        // The first pass unrolls all nodes fully, except for For nodes
        let mut first_pass = UnrollingFirstPass::new();
        Visitor::run(&mut first_pass, ir.constraint_graph_mut());

        // The second pass actually inlines the For nodes
        let mut second_pass = UnrollingSecondPass::new(first_pass.bodies_to_inline.clone());
        Visitor::run(&mut second_pass, ir.constraint_graph_mut());
        Ok(ir)
    }
}

impl Visitor for UnrollingFirstPass {
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
    fn visit_value(&mut self, _graph: &mut Graph, value: Link<Value>) {
        let mir_value = value.borrow().value.value.clone();
        match mir_value {
            MirValue::Constant(c) => match c {
                ConstantValue::Felt(_) => {}
                ConstantValue::Vector(v) => {
                    let mut vec = vec![];
                    for val in v {
                        let val = Value::create(SpannedMirValue {
                            span: value.borrow().value.span.clone(),
                            value: MirValue::Constant(ConstantValue::Felt(val)),
                        })
                        .as_op()
                        .into();
                        vec.push(val);
                    }
                    *value.as_node().borrow_mut().deref_mut() =
                        Vector::create(vec).as_node().borrow().clone();
                }
                ConstantValue::Matrix(m) => {
                    let mut res_m = vec![];
                    for row in m {
                        let mut res_row = vec![];
                        for val in row {
                            let val = Value::create(SpannedMirValue {
                                span: value.borrow().value.span.clone(),
                                value: MirValue::Constant(ConstantValue::Felt(val)),
                            })
                            .as_op()
                            .into();
                            res_row.push(val);
                        }
                        let res_row_vec = Vector::create(res_row).into();
                        res_m.push(res_row_vec);
                    }
                    *value.as_node().borrow_mut().deref_mut() =
                        Matrix::create(res_m).as_node().borrow().clone();
                }
            },
            MirValue::TraceAccess(_) => {}
            MirValue::PeriodicColumn(_) => {}
            MirValue::PublicInput(_) => {}
            MirValue::RandomValue(_) => {}
            MirValue::TraceAccessBinding(trace_access_binding) => {
                // Create Trace Access based on this binding
                let mut vec = vec![];
                for index in 0..trace_access_binding.size {
                    let val = Value::create(SpannedMirValue {
                        span: value.borrow().value.span.clone(),
                        value: MirValue::TraceAccess(TraceAccess {
                            segment: trace_access_binding.segment,
                            column: trace_access_binding.offset + index,
                            row_offset: 0, // ???
                        }),
                    })
                    .as_op()
                    .into();
                    vec.push(val);
                }
                *value.as_node().borrow_mut().deref_mut() =
                    Vector::create(vec).as_node().borrow().clone();
            }
            MirValue::RandomValueBinding(random_value_binding) => {
                let mut vec = vec![];
                for index in 0..random_value_binding.size {
                    let val = Value::create(SpannedMirValue {
                        span: value.borrow().value.span.clone(),
                        value: MirValue::RandomValue(random_value_binding.offset + index),
                    })
                    .as_op()
                    .into();
                    vec.push(val);
                }
                *value.as_node().borrow_mut().deref_mut() =
                    Vector::create(vec).as_node().borrow().clone();
            }
        }
    }

    fn visit_add(&mut self, _graph: &mut Graph, add: Link<Add>) {
        let lhs = add.borrow().lhs.clone();
        let rhs = add.borrow().rhs.clone();

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
                    let new_node = Add::create(lhs.clone(), rhs.clone()).as_op().into();
                    new_vec.push(new_node);
                }
                *add.as_node().borrow_mut().deref_mut() =
                    Vector::create(new_vec).as_node().borrow().clone();
            }
        };
    }

    fn visit_sub(&mut self, _graph: &mut Graph, sub: Link<Sub>) {
        let lhs = sub.borrow().lhs.clone();
        let rhs = sub.borrow().rhs.clone();

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
                    let new_node = Sub::create(lhs.clone(), rhs.clone()).as_op().into();
                    new_vec.push(new_node);
                }
                *sub.as_node().borrow_mut().deref_mut() =
                    Vector::create(new_vec).as_node().borrow().clone();
            }
        };
    }

    fn visit_mul(&mut self, _graph: &mut Graph, mul: Link<Mul>) {
        let lhs = mul.borrow().lhs.clone();
        let rhs = mul.borrow().rhs.clone();

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
                    let new_node = Mul::create(lhs.clone(), rhs.clone()).as_op().into();
                    new_vec.push(new_node);
                }
                *mul.as_node().borrow_mut().deref_mut() =
                    Vector::create(new_vec).as_node().borrow().clone();
            }
        };
    }

    fn visit_enf(&mut self, _graph: &mut Graph, enf: Link<Enf>) {
        let expr = enf.borrow().expr.clone();
        if let Op::Vector(vec) = expr.borrow().deref() {
            let ops = vec.children().borrow().deref().clone();
            let mut new_vec = vec![];
            for op in ops.iter() {
                let new_node = Enf::create(op.clone()).as_op().into();
                new_vec.push(new_node);
            }
            *enf.as_node().borrow_mut().deref_mut() =
                Vector::create(new_vec).as_node().borrow().clone();
        };
    }

    fn visit_fold(&mut self, _graph: &mut Graph, fold: Link<Fold>) {
        let iterator = fold.borrow().iterator.clone();
        let operator = fold.borrow().operator.clone();
        let initial_value = fold.borrow().initial_value.clone();

        let iterator_ref = iterator.borrow();
        let Op::Vector(iterator_vector) = iterator_ref.deref() else {
            unreachable!();
        };
        let iterator_nodes = iterator_vector.children().borrow().deref().clone();

        let mut acc_node = initial_value;
        match operator {
            FoldOperator::Add => {
                for iterator_node in iterator_nodes {
                    let new_acc_node = Add::create(acc_node, iterator_node).as_op().into();
                    acc_node = new_acc_node;
                }
            }
            FoldOperator::Mul => {
                for iterator_node in iterator_nodes {
                    let new_acc_node = Mul::create(acc_node, iterator_node).as_op().into();
                    acc_node = new_acc_node;
                }
            }
            FoldOperator::None => {}
        }

        // Finally, replace the Fold with the expanded expression
        *fold.as_node().borrow_mut().deref_mut() = acc_node.as_node().borrow().clone();
    }

    fn visit_parameter(&mut self, _graph: &mut Graph, _parameter: Link<Parameter>) {
        // FIXME: Just check that the parameter is a scalar, raise diag otherwise
        // List comprehension bodies should only be scalar expressions
    }

    fn visit_if(&mut self, _graph: &mut Graph, if_node: Link<If>) {
        let condition = if_node.borrow().condition.clone();
        let then_branch = if_node.borrow().then_branch.clone();
        let else_branch = if_node.borrow().else_branch.clone();

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
                        If::create(condition.clone(), then_branch.clone(), else_branch.clone())
                            .as_op()
                            .into();
                    new_vec.push(new_node);
                }
                *if_node.as_node().borrow_mut().deref_mut() =
                    Vector::create(new_vec).as_node().borrow().clone();
            }
        };
    }

    fn visit_boundary(&mut self, _graph: &mut Graph, boundary: Link<Boundary>) {
        let expr = boundary.borrow().expr.clone();
        let kind = boundary.borrow().kind.clone();

        if let Op::Vector(vec) = expr.borrow().deref() {
            let expr_vec = vec.children().borrow().deref().clone();
            let mut new_vec = vec![];
            for expr in expr_vec.iter() {
                let new_node = Boundary::create(expr.clone(), kind).as_op().into();
                new_vec.push(new_node);
            }
            *boundary.as_node().borrow_mut().deref_mut() =
                Vector::create(new_vec).as_node().borrow().clone();
        };
    }

    fn visit_accessor(&mut self, _graph: &mut Graph, accessor: Link<Accessor>) {
        let indexable = accessor.borrow().indexable.clone();
        let access_type = accessor.borrow().access_type.clone();
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
                    *accessor.as_node().borrow_mut().deref_mut() =
                        child_accessed.clone().as_node().borrow().clone();
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
                        *accessor.as_node().borrow_mut().deref_mut() =
                            child_accessed.clone().as_node().borrow().clone();
                    } else {
                        unreachable!(); // raise diag
                    };
                } else {
                    unreachable!(); // raise diag
                };
            }

            AccessType::Slice(_range_expr) => {
                unreachable!(); // Slices are not scalar, raise diag
            }
        }
    }

    fn visit_for(&mut self, _graph: &mut Graph, for_node: Link<For>) {
        // For each value produced by the iterators, we need to:
        // - Duplicate the body
        // - Visit the body and replace the Variables with the value (with the correct index depending on the binding)
        // If there is a selector, we need to enforce the selector on the body through an if node ?

        let for_node_clone = for_node.clone();
        let for_ref = for_node_clone.borrow();
        let iterators_ref = for_ref.iterators.borrow();
        let iterators = iterators_ref.deref();
        let expr = for_node.borrow().expr.clone();
        let selector = for_node.borrow().selector.clone();

        // Check iterator lengths
        if iterators.is_empty() {
            unreachable!(); // Raise diag
        }
        let iterator_expected_len = iterators[0]
            .clone()
            .as_vector()
            .expect("Iterators should be vectors")
            .borrow()
            .children()
            .borrow()
            .len();

        for iterator in iterators.iter().skip(1) {
            if iterator
                .clone()
                .as_vector()
                .expect("Iterators should be vectors")
                .borrow()
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
            let new_node = Link::new(Op::None);
            new_vec.push(new_node.clone());

            let iterators_i = iterators
                .iter()
                .map(|op| match op.clone().as_vector() {
                    Some(vec) => vec.borrow().children().borrow()[i].clone(),
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>();
            let selector = if let Op::None = selector.borrow().deref() {
                None
            } else {
                Some(selector.clone())
            };

            self.bodies_to_inline.push((
                for_node.clone().as_op(),
                ForInliningContext {
                    body: expr.clone(),
                    iterators: iterators_i,
                    selector,
                    index: i,
                    parent_for: for_node.clone().as_op(),
                },
            ));
        }
        *for_node.as_node().borrow_mut().deref_mut() =
            Vector::create(new_vec).as_node().borrow().clone();
    }

    fn visit_call(&mut self, _graph: &mut Graph, _call: Link<Call>) {
        unreachable!("Calls should have been inlined before this pass");
    }

    fn visit_function(&mut self, _graph: &mut Graph, _function: Link<Function>) {
        unreachable!("Functions should have been inlined before this pass");
    }

    fn visit_evaluator(&mut self, _graph: &mut Graph, _evaluator: Link<Evaluator>) {
        unreachable!("Evaluators should have been inlined before this pass");
    }
}

impl Visitor for UnrollingSecondPass {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }
    fn root_nodes_to_visit(&self, _graph: &Graph) -> Vec<Link<Node>> {
        self.bodies_to_inline
            .iter()
            .map(|(k, _v)| k)
            .cloned()
            .map(|op| op.as_node())
            .collect::<Vec<_>>()
            .into()
    }
    fn visit_node(&mut self, graph: &mut Graph, node: Link<Node>) {
        if node.borrow().deref() == &Node::None {
            return;
        }
        let node_index = self
            .bodies_to_inline
            .iter()
            .position(|(n, _)| n.clone().as_node() == node);
        match node_index {
            Some(index) => {
                // A new body to inline, we should replace the op with the corresponding iteration in the body
                self.for_inlining_context =
                    Some(self.bodies_to_inline.remove(index).clone().1);
                self.nodes_to_replace.clear();
                self.scan_node(
                    graph,
                    self.for_inlining_context
                        .clone()
                        .unwrap()
                        .body
                        .clone()
                        .as_node(),
                );
                
            }
            None => {
                // Normal visit, insert in the graph the same instruction
                duplicate_node_or_replace(
                    &mut self.nodes_to_replace,
                    node.clone().as_op().unwrap(),
                    self.for_inlining_context.clone().unwrap().iterators,
                );

                if node
                    == self
                        .for_inlining_context
                        .clone()
                        .unwrap()
                        .body
                        .clone()
                        .as_node()
                {
                    // We have finished inlining the body, we can now replace the node in the current index of the parent For
                    let new_node = self
                        .nodes_to_replace
                        .get(&node.clone().as_op().unwrap())
                        .unwrap()
                        .clone();

                    let parent_for: Link<Op> =
                        self.for_inlining_context.clone().unwrap().parent_for;
                    let mut parent_for_mut_ref = parent_for.borrow_mut();
                    let Op::Vector(vector) = parent_for_mut_ref.deref_mut() else {
                        unreachable!();
                    };

                    let new_node_to_update_at = if let Some(selector) =
                        self.for_inlining_context.clone().unwrap().selector
                    {
                        let zero_node = Value::create(SpannedMirValue {
                            span: Default::default(),
                            value: MirValue::Constant(ConstantValue::Felt(0)),
                        })
                        .as_op()
                        .into();
                        let if_node = If::create(selector, new_node, zero_node).as_op().into();
                        if_node
                    } else {
                        new_node
                    };

                    let children = vector.children();
                    let mut children_mut_ref = children.borrow_mut();
                    let children_mut = children_mut_ref.deref_mut();
                    let child_to_update = children_mut
                        .get_mut(self.for_inlining_context.clone().unwrap().index)
                        .unwrap();
                    *child_to_update = new_node_to_update_at;
                }
            }
        }
    }
}
