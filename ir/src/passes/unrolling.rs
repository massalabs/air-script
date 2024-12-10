use std::{
    collections::HashMap,
    ops::{ControlFlow, Deref, DerefMut},
};

use air_parser::ast::{AccessType, Boundary as BoundaryKind};
use air_pass::Pass;
//use miden_diagnostics::DiagnosticsHandler;

use crate::{ir3::*, CompileError};

use super::{duplicate_node_or_replace, Visit, VisitContext, VisitOrder};

//pub struct Unrolling<'a> {
//     #[allow(unused)]
//     diagnostics: &'a DiagnosticsHandler,
//}

#[derive(Clone)]
pub struct ForInliningContext {
    body: Link<NodeType>,
    iterators: Vec<Link<NodeType>>,
    selector: Option<Link<NodeType>>,
    index: usize,
    parent_for: Link<NodeType>,
}

impl ForInliningContext {}

pub struct Unrolling {
    // general context
    work_stack: Vec<Link<NodeType>>,
    during_first_pass: bool,

    // context for both passes
    bodies_to_inline: Vec<(Link<NodeType>, ForInliningContext)>,

    // context for second pass
    for_inlining_context: Option<ForInliningContext>,
    nodes_to_replace: HashMap<Link<NodeType>, Link<NodeType>>,
}

impl VisitContext for Unrolling {
    #[allow(unused)]
    fn visit(&mut self, graph: &mut Graph, node: Link<NodeType>) {
        if self.during_first_pass {
            self.visit_first_pass(node);
        } else {
            self.visit_second_pass(node);
        }
    }

    fn as_stack_mut(&mut self) -> &mut Vec<Link<NodeType>> {
        &mut self.work_stack
    }

    type Graph = Graph;

    fn boundary_roots(&self, graph: &Self::Graph) -> Link<Vec<Link<NodeType>>> {
        if self.during_first_pass {
            return graph.boundary_constraints_roots.clone();
        } else {
            return Link::new(
                self.bodies_to_inline
                    .iter()
                    .map(|(k, _v)| k)
                    .cloned()
                    .collect(),
            );
        }
    }

    fn integrity_roots(&self, graph: &Self::Graph) -> Link<Vec<Link<NodeType>>> {
        if self.during_first_pass {
            return graph.integrity_constraints_roots.clone();
        } else {
            return Link::new(vec![]);
        }
    }

    fn visit_order(&self) -> super::VisitOrder {
        if self.during_first_pass {
            return super::VisitOrder::PostOrder;
        } else {
            return super::VisitOrder::PostOrder;
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
        match self.visit_order() {
            VisitOrder::Manual => self.visit_manual(graph),
            VisitOrder::PostOrder => self.visit_postorder(graph),
            VisitOrder::DepthFirst => self.visit_depthfirst(graph),
        }
        while let Some(node) = self.next_node() {
            self.visit(graph, node);
        }

        // Second pass, inline For nodes
        self.during_first_pass = false;
        match self.visit_order() {
            VisitOrder::Manual => self.visit_manual(graph),
            VisitOrder::PostOrder => self.visit_postorder(graph),
            VisitOrder::DepthFirst => self.visit_depthfirst(graph),
        }
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

enum BinaryOp {
    Add,
    Sub,
    Mul,
}

impl Unrolling {
    fn visit_value(&mut self, node: Link<NodeType>) {
        match node.borrow().deref() {
            NodeType::LeafNode(LeafNode::Value(leaf)) => {
                match &leaf.data {
                    SpannedMirValue {
                        span: span,
                        value: value,
                    } => {
                        match value {
                            MirValue::Constant(c) => match c {
                                ConstantValue::Felt(_) => {}
                                ConstantValue::Vector(v) => {
                                    let mut vec = vec![];
                                    for val in v {
                                        let val = SpannedMirValue {
                                            span: span.clone(),
                                            value: MirValue::Constant(ConstantValue::Felt(*val)),
                                        }
                                        .into();
                                        vec.push(val);
                                    }
                                    let new_node = Vector::new(vec);
                                    *node.borrow_mut().deref_mut() =
                                        NodeType::MiddleNode(MiddleNode::Vector(new_node));
                                }
                                ConstantValue::Matrix(m) => {
                                    let mut res_m = vec![];
                                    for row in m {
                                        let mut res_row = vec![];
                                        for val in row {
                                            let val = SpannedMirValue {
                                                span: span.clone(),
                                                value: MirValue::Constant(ConstantValue::Felt(
                                                    *val,
                                                )),
                                            }
                                            .into();
                                            res_row.push(val);
                                        }
                                        res_m.push(res_row);
                                    }
                                    let new_node = Matrix::new(res_m);
                                    *node.borrow_mut().deref_mut() =
                                        NodeType::MiddleNode(MiddleNode::Matrix(new_node));
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
                                    let val = SpannedMirValue {
                                        span: span.clone(),
                                        value: MirValue::TraceAccess(TraceAccess {
                                            segment: trace_access_binding.segment,
                                            column: trace_access_binding.offset + index,
                                            row_offset: 0, // ???
                                        }),
                                    }
                                    .into();
                                    vec.push(val);
                                }
                                let new_node = Vector::new(vec);
                                *node.borrow_mut().deref_mut() =
                                    NodeType::MiddleNode(MiddleNode::Vector(new_node));
                            }
                            MirValue::RandomValueBinding(random_value_binding) => {
                                let mut vec = vec![];
                                for index in 0..random_value_binding.size {
                                    let val = SpannedMirValue {
                                        span: span.clone(),
                                        value: MirValue::RandomValue(
                                            random_value_binding.offset + index,
                                        ),
                                    }
                                    .into();
                                    vec.push(val);
                                }
                                let new_node = Vector::new(vec);
                                *node.borrow_mut().deref_mut() =
                                    NodeType::MiddleNode(MiddleNode::Vector(new_node));
                            }
                        }
                    }
                }
            }
            _ => unreachable!(),
        };
    }

    fn visit_binary_op(
        &mut self,
        node: Link<NodeType>,
        lhs: Link<NodeType>,
        rhs: Link<NodeType>,
        binary_op: BinaryOp,
    ) {
        match (lhs.borrow().deref(), rhs.borrow().deref()) {
            (
                NodeType::MiddleNode(MiddleNode::Vector(lhs_vec)),
                NodeType::MiddleNode(MiddleNode::Vector(rhs_vec)),
            ) => {
                let lhs_vec = lhs_vec.get_children().borrow().deref().clone();
                let rhs_vec = rhs_vec.get_children().borrow().deref().clone();
                if lhs_vec.len() != rhs_vec.len() {
                    // Raise diag
                } else {
                    let mut new_vec = vec![];
                    for (lhs, rhs) in lhs_vec.iter().zip(rhs_vec.iter()) {
                        let new_node = match binary_op {
                            BinaryOp::Add => Add::new(lhs.clone(), rhs.clone()).into(),
                            BinaryOp::Sub => Sub::new(lhs.clone(), rhs.clone()).into(),
                            BinaryOp::Mul => Mul::new(lhs.clone(), rhs.clone()).into(),
                        };
                        new_vec.push(new_node);
                    }
                    *node.borrow_mut().deref_mut() =
                        NodeType::MiddleNode(MiddleNode::Vector(Vector::new(new_vec)));
                }
            }
            _ => {}
        }
    }

    fn visit_enf(&mut self, node: Link<NodeType>, child_node: Link<NodeType>) {
        match child_node.borrow().deref() {
            NodeType::MiddleNode(MiddleNode::Vector(child_vec)) => {
                let child_vec = child_vec.get_children().borrow().deref().clone();
                let mut new_vec = vec![];
                for child in child_vec.iter() {
                    let new_node = Enf::new(child.clone()).into();
                    new_vec.push(new_node);
                }
                *node.borrow_mut().deref_mut() =
                    NodeType::MiddleNode(MiddleNode::Vector(Vector::new(new_vec)));
            }
            _ => {}
        }
    }

    fn visit_scope(&mut self, node: Link<NodeType>) {
        match node.borrow().deref() {
            NodeType::MiddleNode(MiddleNode::Scope(_scope)) => {
                todo!();
                /*let child_vec = child_vec.get_children().borrow().deref().clone();
                let mut new_vec = vec![];
                for child in child_vec.iter() {
                    let new_node = Enf::new(child.clone()).into();
                    new_vec.push(new_node);
                }
                *node.borrow_mut().deref_mut() = NodeType::MiddleNode(MiddleNode::Vector(Vector::new(new_vec)));*/
            }
            _ => {}
        }
    }

    fn visit_fold(
        &mut self,
        node: Link<NodeType>,
        iterator: Link<NodeType>,
        fold_operator: FoldOperator,
        accumulator: Link<NodeType>,
    ) {
        // We need to expand this Fold into a nested sequence of binary expressions (add or mul depending on fold_operator)
        let iterator_nodes = match iterator.borrow().deref() {
            NodeType::MiddleNode(MiddleNode::Vector(vec)) => {
                vec.get_children().borrow().deref().clone()
            }
            _ => unreachable!(),
        };

        let mut acc_node = accumulator;
        match fold_operator {
            FoldOperator::Add => {
                for iterator_node in iterator_nodes {
                    let new_acc_node = Add::new(acc_node, iterator_node).into();
                    acc_node = new_acc_node;
                }
            }
            FoldOperator::Mul => {
                for iterator_node in iterator_nodes {
                    let new_acc_node = Mul::new(acc_node, iterator_node).into();
                    acc_node = new_acc_node;
                }
            }
        }

        // Finally, replace the Fold with the expanded expression
        *node.borrow_mut().deref_mut() = acc_node.borrow().deref().clone();
    }

    fn visit_parameter(&mut self, node: Link<NodeType>) {
        // Just check that the variable is a scalar, raise diag otherwise
        // List comprehension bodies should only be scalar expressions

        match node.borrow().deref() {
            NodeType::LeafNode(LeafNode::Parameter(parameter)) => match &parameter.data.ty {
                MirType::Felt => {}
                MirType::Vector(_size) => unreachable!(),
                MirType::Matrix(_rows, _cols) => unreachable!(),
                MirType::Definition(_vec, _) => todo!(),
            },
            _ => unreachable!(),
        }
    }

    fn visit_if(
        &mut self,
        node: Link<NodeType>,
        cond_node: Link<NodeType>,
        then_node: Link<NodeType>,
        else_node: Link<NodeType>,
    ) {
        match (
            cond_node.borrow().deref(),
            then_node.borrow().deref(),
            else_node.borrow().deref(),
        ) {
            (
                NodeType::LeafNode(LeafNode::Value(_cond_leaf)),
                NodeType::LeafNode(LeafNode::Value(_then_leaf)),
                NodeType::LeafNode(LeafNode::Value(_else_leaf)),
            ) => {
                // Check value types to ensure scalar, raise diag otherwise
            }
            (
                NodeType::MiddleNode(MiddleNode::Vector(cond_vec)),
                NodeType::MiddleNode(MiddleNode::Vector(then_vec)),
                NodeType::MiddleNode(MiddleNode::Vector(else_vec)),
            ) => {
                let cond_vec = cond_vec.get_children().borrow().deref().clone();
                let then_vec = then_vec.get_children().borrow().deref().clone();
                let else_vec = else_vec.get_children().borrow().deref().clone();
                if cond_vec.len() != then_vec.len() || cond_vec.len() != else_vec.len() {
                    // Raise diag
                } else {
                    let mut new_vec = vec![];
                    for ((cond, then), else_) in
                        cond_vec.iter().zip(then_vec.iter()).zip(else_vec.iter())
                    {
                        let new_node = If::new(cond.clone(), then.clone(), else_.clone()).into();
                        new_vec.push(new_node);
                    }
                    *node.borrow_mut().deref_mut() =
                        NodeType::MiddleNode(MiddleNode::Vector(Vector::new(new_vec)));
                }
            }
            _ => unreachable!(),
        }
    }

    fn visit_boundary(
        &mut self,
        node: Link<NodeType>,
        boundary: BoundaryKind,
        child_node: Link<NodeType>,
    ) {
        match child_node.borrow().deref() {
            NodeType::MiddleNode(MiddleNode::Vector(child_vec)) => {
                let child_vec = child_vec.get_children().borrow().deref().clone();
                let mut new_vec = vec![];
                for child in child_vec.iter() {
                    let new_node = Boundary::new(child.clone(), boundary).into();
                    new_vec.push(new_node);
                }
                *node.borrow_mut().deref_mut() =
                    NodeType::MiddleNode(MiddleNode::Vector(Vector::new(new_vec)));
            }
            _ => {}
        }
    }

    fn visit_index_access(
        &mut self,
        node: Link<NodeType>,
        access_type: AccessType,
        child_node: Link<NodeType>,
    ) {
        match access_type {
            AccessType::Default() => {
                // Check that the child node is a scalar, raise diag otherwise
                match child_node.borrow().deref() {
                    NodeType::MiddleNode(MiddleNode::Vector(child_vec)) => {
                        unreachable!(); // raise diag
                    }
                    NodeType::MiddleNode(MiddleNode::Matrix(child_mat)) => {
                        unreachable!(); // raise diag
                    }
                    _ => {}
                };
            }
            AccessType::Index(index) => {
                // Check that the child node is a vector, raise diag otherwise
                // Replace the current node by the index-th element of the vector
                // Raise diag if index is out of bounds
                let NodeType::MiddleNode(MiddleNode::Vector(child_vec)) =
                    child_node.borrow().deref()
                else {
                    unreachable!(); // raise diag
                };

                let child_index = match child_vec.get_children().borrow().deref().get(index) {
                    Some(child_index) => child_index,
                    None => unreachable!(), // raise diag
                };
                *node.borrow_mut().deref_mut() = child_index.borrow().deref().clone().into();
            }
            AccessType::Matrix(row, col) => {
                // Check that the child node is a matrix, raise diag otherwise
                // Replace the current node by the index-th element of the vector
                // Raise diag if index is out of bounds
                let NodeType::MiddleNode(MiddleNode::Matrix(child_mat)) =
                    child_node.borrow().deref()
                else {
                    unreachable!(); // raise diag
                };

                let child_row = match child_mat.get_children().borrow().deref().get(row) {
                    Some(child_row) => child_row,
                    None => unreachable!(), // raise diag
                };

                let NodeType::MiddleNode(MiddleNode::Matrix(child_row)) =
                    child_row.borrow().deref()
                else {
                    unreachable!(); // raise diag
                };

                let child_index = match child_row.get_children().borrow().deref().get(col) {
                    Some(child_index) => child_index,
                    None => unreachable!(), // raise diag
                };

                *node.borrow_mut().deref_mut() = child_index.borrow().deref().clone().into();
            }

            AccessType::Slice(range_expr) => {
                unreachable!(); // Slices are not scalar, raise diag
            }
        }
    }

    fn visit_for(
        &mut self,
        node: Link<NodeType>,
        iterators: Vec<Link<NodeType>>,
        body: Link<NodeType>,
        selector: Option<Link<NodeType>>,
    ) {
        // For each value produced by the iterators, we need to:
        // - Duplicate the body
        // - Visit the body and replace the Variables with the value (with the correct index depending on the binding)
        // If there is a selector, we need to enforce the selector on the body through an if node ?

        // Check iterator lengths
        if iterators.is_empty() {
            unreachable!(); // Raise diag
        }
        let iterator_expected_len = match iterators[0].borrow().deref() {
            NodeType::MiddleNode(MiddleNode::Vector(vec)) => {
                vec.get_children().borrow().deref().len()
            }
            _ => unreachable!(),
        };

        for iterator in iterators.iter().skip(1) {
            match iterator.borrow().deref() {
                NodeType::MiddleNode(MiddleNode::Vector(vec)) => {
                    if vec.get_children().borrow().deref().len() != iterator_expected_len {
                        unreachable!(); // Raise diag
                    }
                }
                _ => unreachable!(),
            }
        }

        let iterator_nodes = iterators
            .iter()
            .map(|iterator| match iterator.borrow().deref() {
                NodeType::MiddleNode(MiddleNode::Vector(vec)) => *vec,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();

        let mut new_vec = vec![];
        for i in 0..iterator_expected_len {
            let new_node = Link::new(NodeType::None);
            new_vec.push(new_node);

            let iterators_i = iterator_nodes
                .iter()
                .map(|vec| vec.get_children().borrow()[i])
                .collect::<Vec<_>>();

            self.bodies_to_inline.push((
                new_node,
                ForInliningContext {
                    body: body,
                    iterators: iterators_i,
                    selector: selector,
                    index: i,
                    parent_for: node,
                },
            ));
        }
        *node.borrow_mut().deref_mut() =
            NodeType::MiddleNode(MiddleNode::Vector(Vector::new(new_vec)));
    }

    fn visit_first_pass(&mut self, node: Link<NodeType>) {
        match node.clone().borrow().deref() {
            NodeType::RootNode(_root_node) => {
                // FIXME: Either unreachable or we should do nothing?
                unreachable!();
            }
            NodeType::LeafNode(leaf_node) => {
                match leaf_node {
                    LeafNode::Value(_leaf) => {
                        // Transform values to scalar nodes (in the case of a vector or matrix, transform into Operation::Vector or Operation::Matrix)
                        self.visit_value(node);
                    }
                    LeafNode::Parameter(_leaf) => {
                        self.visit_parameter(node);
                    }
                }
            }
            NodeType::MiddleNode(middle_node) => {
                match middle_node {
                    MiddleNode::Add(add) => {
                        let lhs = add.lhs();
                        let rhs = add.rhs();
                        self.visit_binary_op(node, lhs, rhs, BinaryOp::Add);
                    }
                    MiddleNode::Sub(sub) => {
                        let lhs = sub.lhs();
                        let rhs = sub.rhs();
                        self.visit_binary_op(node, lhs, rhs, BinaryOp::Sub);
                    }
                    MiddleNode::Mul(mul) => {
                        let lhs = mul.lhs();
                        let rhs = mul.rhs();
                        self.visit_binary_op(node, lhs, rhs, BinaryOp::Mul);
                    }
                    MiddleNode::Scope(_scope) => {
                        self.visit_scope(node);
                    }
                    MiddleNode::If(if_node) => {
                        let cond = if_node.cond();
                        let then_branch = if_node.then_branch();
                        let else_branch = if_node.else_branch();
                        self.visit_if(node, cond, then_branch, else_branch);
                    }
                    MiddleNode::For(for_node) => {
                        // For each value produced by the iterators, we need to:
                        // - Duplicate the body
                        // - Visit the body and replace the Variables with the value (with the correct index depending on the binding)
                        // We then have a vector, that we can either fold up or enforce on each value

                        let iterators = for_node.iterators();
                        let body = for_node.body();
                        let selector = for_node.selector();
                        self.visit_for(node, iterators, body, selector);
                    }
                    MiddleNode::Accessor(access) => {
                        let access_type = access.access_type;
                        let child = access.indexable();
                        self.visit_index_access(node, access_type, child);
                    }
                    MiddleNode::Fold(fold) => {
                        let iterator = fold.iterator();
                        let fold_operator = fold.operator.clone();
                        let accumulator = fold.initial_value();
                        self.visit_fold(node, iterator, fold_operator, accumulator);
                    }

                    MiddleNode::Boundary(boundary) => {
                        let kind = boundary.kind;
                        let child = boundary.expr();
                        self.visit_boundary(node, kind, child);
                    }
                    MiddleNode::Enf(enf) => {
                        let child = enf.expr();
                        self.visit_enf(node, child);
                    }

                    // These are already unrolled
                    MiddleNode::Vector(_vector) => {}
                    MiddleNode::Matrix(_matrix) => {}

                    // These should not exist / be accessible from roots after inlining
                    MiddleNode::Function(_function) => unreachable!(),
                    MiddleNode::Evaluator(_evaluator) => unreachable!(),
                    MiddleNode::Call(_call) => unreachable!(),
                }
            }
        }
    }

    fn visit_second_pass(&mut self, node: Link<NodeType>) {
        let node_index = self.bodies_to_inline.iter().position(|(n, _)| n == &node);
        match node_index {
            Some(index) => {
                // A new body to inline, we should replace the op with the corresponding iteration in the body
                self.for_inlining_context =
                    Some(self.bodies_to_inline.get(index).unwrap().clone().1);
                self.nodes_to_replace.clear();
                self.visit_later(self.for_inlining_context.unwrap().body);
            }
            None => {
                // Normal visit, insert in the graph the same instruction
                duplicate_node_or_replace(
                    &mut self.nodes_to_replace,
                    node,
                    self.for_inlining_context.unwrap().iterators,
                );

                if node == self.for_inlining_context.unwrap().body {
                    // We have finished inlining the body, we can now replace the node in the current index of the parent For
                    let new_node = self.nodes_to_replace.get(&node).unwrap().clone();

                    let parent_for = self.for_inlining_context.unwrap().parent_for;
                    match parent_for.borrow_mut().deref_mut() {
                        NodeType::MiddleNode(MiddleNode::Vector(vec)) => {
                            let new_node_to_update_at = if let Some(selector) =
                                self.for_inlining_context.unwrap().selector
                            {
                                let zero_node = SpannedMirValue {
                                    span: Default::default(),
                                    value: MirValue::Constant(ConstantValue::Felt(0)),
                                }
                                .into();
                                let if_node = If::new(selector, new_node, zero_node).into();
                                if_node
                            } else {
                                new_node
                            };

                            let children = vec.get_children().borrow_mut().deref_mut();
                            let child_to_update = children
                                .get_mut(self.for_inlining_context.unwrap().index)
                                .unwrap();
                            *child_to_update = new_node_to_update_at;
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}
