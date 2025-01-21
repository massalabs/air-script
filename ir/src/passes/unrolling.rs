use std::{
    collections::HashMap,
    ops::{ControlFlow, Deref, DerefMut},
};

use air_parser::ast::AccessType;
use air_pass::Pass;
//use miden_diagnostics::DiagnosticsHandler;

use crate::{ir::*, CompileError};

use super::{duplicate_node_or_replace, Visit, VisitContext};

//pub struct Unrolling<'a> {
//     #[allow(unused)]
//     diagnostics: &'a DiagnosticsHandler,
//}

#[derive(Clone)]
pub struct ForInliningContext {
    body: Link<Op>,
    iterators: Vec<Link<Op>>,
    selector: Option<Link<Op>>,
    index: usize,
    parent_for: Link<Op>,
}

impl ForInliningContext {}

pub struct Unrolling {
    // general context
    work_stack: Vec<Link<Node>>,
    during_first_pass: bool,

    // context for both passes
    bodies_to_inline: Vec<(Link<Op>, ForInliningContext)>,

    // context for second pass
    for_inlining_context: Option<ForInliningContext>,
    nodes_to_replace: HashMap<Link<Op>, Link<Op>>,
}

impl VisitContext for Unrolling {
    #[allow(unused)]
    fn visit(&mut self, graph: &mut Graph, node: Link<Node>) {
        if self.during_first_pass {
            self.visit_first_pass(node);
        } else {
            self.visit_second_pass(node);
        }
    }

    fn as_stack_mut(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }

    type Graph = Graph;

    fn root_nodes_to_visit(&self, graph: &Self::Graph) -> Link<Vec<Link<Node>>> {
        if self.during_first_pass {
            return graph
                .boundary_constraints_roots
                .borrow()
                .deref()
                .iter()
                .cloned()
                .map(|bc| bc.as_node())
                .chain(graph
                    .integrity_constraints_roots
                    .borrow()
                    .deref()
                    .iter()
                    .cloned()
                    .map(|ic| ic.as_node())
                ).collect::<Vec<_>>()
                .into();
        } else {
            return self
                .bodies_to_inline
                .iter()
                .map(|(k, _v)| k)
                .cloned()
                .map(|op| op.as_node())
                .collect::<Vec<_>>()
                .into();
        }
    }
}

//impl<'p> Pass for Unrolling<'p> {}
impl Pass for Unrolling {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        match self.run_visitor(&mut ir.constraint_graph_mut()) {
            ControlFlow::Continue(()) => Ok(ir),
            ControlFlow::Break(_err) => Err(CompileError::Failed),
        }
    }
}

impl Visit for Unrolling {
    fn run(&mut self, graph: &mut Self::Graph) {
        // First pass, unroll all nodes fully, except for For nodes
        self.during_first_pass = true;
        self.visit_postorder(graph);
        while let Some(node) = self.next_node() {
            self.visit(graph, node);
        }

        // Second pass, inline For nodes
        self.during_first_pass = false;
        self.visit_postorder(graph);
        while let Some(node) = self.next_node() {
            self.visit(graph, node);
        }
    }
}

// impl<'a> Unrolling<'a> {
//     pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
//         Self { diagnostics }
//         Self {}
//     }
// }
impl Unrolling {
    pub fn new() -> Self {
        Self {
            work_stack: vec![],
            during_first_pass: true,
            bodies_to_inline: Vec::new(),
            for_inlining_context: None,
            nodes_to_replace: HashMap::new(),
        }
    }
    //TODO MIR: Implement inlining pass on MIR
    // 1. Understand the basics of the previous inlining process
    // 2. Remove what is done during lowering from AST to MIR (unroll, ...)
    // 3. Check how it translates to the MIR structure
    fn run_visitor(&mut self, ir: &mut Graph) -> ControlFlow<()> {
        Visit::run(self, ir);
        ControlFlow::Continue(())
    }
}

impl Unrolling {
    fn visit_value(&mut self, value: Link<Value>) -> Option<Link<Op>> {
        match value.borrow().value.value.clone() {
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
                    return Some(Vector::create(vec).as_op());
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
                    return Some(Matrix::create(res_m).as_op());
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
                return Some(Vector::create(vec).as_op());
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
                return Some(Vector::create(vec).as_op());
            }
        }
        None
    }

    fn visit_add(&mut self, add: Link<Add>) -> Option<Link<Op>> {
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
                return Some(Vector::create(new_vec).as_op());
            }
        };
        None
    }

    fn visit_sub(&mut self, sub: Link<Sub>) -> Option<Link<Op>> {
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
                return Some(Vector::create(new_vec).as_op());
            }
        };
        None
    }

    fn visit_mul(&mut self, mul: Link<Mul>) -> Option<Link<Op>> {
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
                return Some(Vector::create(new_vec).as_op());
            }
        };
        None
    }

    fn visit_enf(&mut self, enf: Link<Enf>) -> Option<Link<Op>> {
        let expr = enf.borrow().expr.clone();
        if let Op::Vector(vec) = expr.borrow().deref() {
            let ops = vec.children().borrow().deref().clone();
            let mut new_vec = vec![];
            for op in ops.iter() {
                let new_node = Enf::create(op.clone()).as_op().into();
                new_vec.push(new_node);
            }
            return Some(Vector::create(new_vec).as_op());
        };
        None
    }

    fn visit_fold(&mut self, fold: Link<Fold>) -> Option<Link<Op>> {
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
        return Some(acc_node.clone());
    }

    fn visit_parameter(&mut self, _parameter: Link<Parameter>) -> Option<Link<Op>> {
        // FIXME: Just check that the parameter is a scalar, raise diag otherwise
        // List comprehension bodies should only be scalar expressions
        None
    }

    fn visit_if(&mut self, if_node: Link<If>) -> Option<Link<Op>> {
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
                return Some(Vector::create(new_vec).as_op());
            }
        };
        None
    }

    fn visit_boundary(&mut self, boundary: Link<Boundary>) -> Option<Link<Op>> {
        let expr = boundary.borrow().expr.clone();
        let kind = boundary.borrow().kind.clone();

        if let Op::Vector(vec) = expr.borrow().deref() {
            let expr_vec = vec.children().borrow().deref().clone();
            let mut new_vec = vec![];
            for expr in expr_vec.iter() {
                let new_node = Boundary::create(expr.clone(), kind).as_op().into();
                new_vec.push(new_node);
            }
            return Some(Vector::create(new_vec).as_op());
        };
        None
    }

    fn visit_accessor(&mut self, accessor: Link<Accessor>) -> Option<Link<Op>> {
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

                    return Some(child_accessed.clone());
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

                        return Some(child_accessed.clone());
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
        None
    }

    fn visit_for(&mut self, node: Link<Op>, for_node: Link<For>) -> Option<Link<Op>> {
        // For each value produced by the iterators, we need to:
        // - Duplicate the body
        // - Visit the body and replace the Variables with the value (with the correct index depending on the binding)
        // If there is a selector, we need to enforce the selector on the body through an if node ?

        let for_ref = for_node.borrow();
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
                new_node,
                ForInliningContext {
                    body: expr.clone(),
                    iterators: iterators_i,
                    selector,
                    index: i,
                    parent_for: node.clone(),
                },
            ));
        }
        Some(Vector::create(new_vec).as_op())
    }

    fn visit_first_pass(&mut self, node: Link<Node>) {
        let new_op: Option<Link<Op>> = match node.clone().borrow().deref() {
            Node::Enf(enf) => self.visit_enf(enf.clone()),
            Node::Boundary(boundary) => self.visit_boundary(boundary.clone()),
            Node::Add(add) => self.visit_add(add.clone()),
            Node::Sub(sub) => self.visit_sub(sub.clone()),
            Node::Mul(mul) => self.visit_mul(mul.clone()),
            Node::If(if_node) => self.visit_if(if_node.clone()),
            Node::For(for_node) => {
                // For each value produced by the iterators, we need to:
                // - Duplicate the body
                // - Visit the body and replace the Variables with the value (with the correct index depending on the binding)
                // We then have a vector, that we can either fold up or enforce on each value
                self.visit_for(node.clone().as_op().unwrap(), for_node.clone())
            }
            Node::Fold(fold) => self.visit_fold(fold.clone()),
            Node::Accessor(accessor) => self.visit_accessor(accessor.clone()),
            Node::Parameter(parameter) => self.visit_parameter(parameter.clone()),
            Node::Value(value) => self.visit_value(value.clone()),

            // These should not exist / be accessible from roots after inlining
            Node::Call(_call) => unreachable!(),
            Node::Function(_function) => unreachable!(),
            Node::Evaluator(_evaluator) => unreachable!(),

            // Already unrolled
            Node::Vector(_vector) => None,
            Node::Matrix(_matrix) => None,
            Node::None => None,
        };
        if let Some(new_op) = new_op {
            node.set(new_op);
        }
    }

    fn visit_second_pass(&mut self, node: Link<Node>) {
        let node_index = self
            .bodies_to_inline
            .iter()
            .position(|(n, _)| n.clone().as_node() == node);
        match node_index {
            Some(index) => {
                // A new body to inline, we should replace the op with the corresponding iteration in the body
                self.for_inlining_context =
                    Some(self.bodies_to_inline.get(index).unwrap().clone().1);
                self.nodes_to_replace.clear();
                self.visit_later(
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
