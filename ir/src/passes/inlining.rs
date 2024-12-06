use std::{collections::{BTreeMap, HashSet}, ops::Deref};

use air_pass::Pass;
//use miden_diagnostics::DiagnosticsHandler;

use crate::{ir2::{Graph, Link, MiddleNode, Mir, NodeType}, CompileError};

use super::{visitor::VisitDefault, Visit, VisitContext, VisitOrder};

//pub struct Inlining<'a> {
//     #[allow(unused)]
//     diagnostics: &'a DiagnosticsHandler,
//}

pub struct Inlining {
    work_stack: Vec<Link<NodeType>>,
}

impl VisitContext for Inlining {
    type Graph = Graph;
    fn visit(&mut self, graph: &mut Graph, node: Link<NodeType>) {
        match node.clone().borrow().deref() {
            NodeType::MiddleNode(MiddleNode::Function(_)) => self.visit_body(graph, node),
            NodeType::MiddleNode(MiddleNode::Evaluator(_)) => self.visit_body(graph, node),
            _ => {}
        }
    }
    fn as_stack_mut(&mut self) -> &mut Vec<Link<NodeType>> {
        &mut self.work_stack
    }
    fn boundary_roots(&self, graph: &Graph) -> Link<Vec<Link<NodeType>>> {
        graph.boundary_constraints_roots.clone()
    }
    fn integrity_roots(&self, graph: &Graph) -> Link<Vec<Link<NodeType>>> {
        graph.integrity_constraints_roots.clone()
    }
    fn visit_order(&self) -> VisitOrder {
        VisitOrder::Manual
    }
}

//impl<'p> Pass for Inlining<'p> {}
impl Pass for Inlining {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut context = Inlining::new();
        Visit::run(&mut context, &mut ir.constraint_graph_mut());
        Ok(ir)
    }
}

impl VisitDefault for Inlining {}

// impl<'a> Inlining<'a> {
//     pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
//         Self { diagnostics }
//         Self {}
//     }
// }
impl Inlining {
    pub fn new() -> Self {
        Self { work_stack: vec![] }
    }
    fn visit_body(&mut self, ir: &mut Graph, node: Link<NodeType>) {

        match node.clone().borrow().deref() {
            NodeType::MiddleNode(MiddleNode::Function(f)) => {
                // FIXME: .body() should return a Vec<>
                let body = f.body();

                // Find all calls in the body
                for (index_in_body, call) in body.iter().enumerate() {
                    self.inline_call(ir, call, &node, index_in_body);
                }
            },
            NodeType::MiddleNode(MiddleNode::Evaluator(ev)) => {
                // FIXME: .body() should return a Vec<>
                let body = ev.body();

                // Find all calls in the body
                for (index_in_body, call) in body.iter().enumerate() {
                    self.inline_call(ir, call, &node, index_in_body);
                }
            },
            _ => {}
        }
    }

    fn inline_call(
        &mut self,
        ir: &mut Graph,
        call: &Link<NodeType>,
        outer_def: &Link<NodeType>,
        index_in_body: usize,
    ) {
        let call_node = ir.node(call).clone();
        if let Operation::Call(def, arg_valuees) = &call_node.op {
            let mut body_map = BTreeMap::new();
            // Inline the body of the called function
            let new_nodes = self.inline_body(ir, &mut body_map, def, arg_valuees);
            let outer_def_node = ir.node(outer_def).clone();
            if let Operation::Definition(outer_func_arges, outer_func_ret, outer_body) =
                &outer_def_node.op
            {
                // Edit the body of the outer function
                // body.last: swap the call with the last node
                let mut new_body = outer_body.clone();
                new_body[index_in_body] = *new_nodes.last().unwrap();
                // body[..body.last]: insert the new nodes in reverse order
                for op_idx in new_nodes.iter().rev().skip(1) {
                    new_body.insert(index_in_body, *op_idx);
                }
                ir.update_node(
                    outer_def,
                    Operation::Definition(
                        outer_func_arges.clone(),
                        *outer_func_ret,
                        new_body,
                    ),
                );
                self.visit_later(*outer_def);
            }
        }
    }

    fn inline_body(
        &mut self,
        ir: &mut Graph,
        body_map: &mut BTreeMap<Link<NodeType>, Link<NodeType>>,
        def: &Link<NodeType>,
        arg_valuees: &[Link<NodeType>],
    ) -> Vec<Link<NodeType>> {
        let def_node = ir.node(def).clone();
        let mut new_body = vec![];
        if let Operation::Definition(arges, _, body) = &def_node.op {
            // map the arguments to the values of the call
            for (arg, arg_value) in arges.iter().zip(arg_valuees) {
                body_map.insert(*arg, *arg_value);
            }
            // Inline the body of the called function
            for node in body {
                self.inline_op(ir, body_map, node, &mut new_body);
            }
        }
        new_body
    }

    fn inline_op(
        &mut self,
        ir: &mut Graph,
        body_map: &mut BTreeMap<Link<NodeType>, Link<NodeType>>,
        op: &Link<NodeType>,
        new_body: &mut Vec<Link<NodeType>>,
    ) {
        // Clone the operation and insert it in the new body
        let new_node = ir.insert_op_placeholder();
        body_map.insert(*op, new_node);
        let op_node = ir.node(op).clone();
        // Update the operation with the new indexes
        let op = match op_node.op.clone() {
            Operation::Value(value) => Operation::Value(value),
            Operation::Add(lhs, rhs) => Operation::Add(
                *body_map.get(&lhs).expect("Add lhs not found"),
                *body_map.get(&rhs).expect("Add rhs not found"),
            ),
            Operation::Sub(lhs, rhs) => Operation::Sub(
                *body_map.get(&lhs).expect("Sub lhs not found"),
                *body_map.get(&rhs).expect("Sub rhs not found"),
            ),
            Operation::Mul(lhs, rhs) => Operation::Mul(
                *body_map.get(&lhs).expect("Mul lhs not found"),
                *body_map.get(&rhs).expect("Mul rhs not found"),
            ),
            Operation::Vector(values) => Operation::Vector(
                values
                    .iter()
                    .map(|value| {
                        *body_map
                            .get(value)
                            .expect("Vector value not found")
                    })
                    .collect(),
            ),
            Operation::Matrix(rows) => Operation::Matrix(
                rows.iter()
                    .map(|row| {
                        row.iter()
                            .map(|value| {
                                *body_map
                                    .get(value)
                                    .expect("Matrix value not found")
                            })
                            .collect()
                    })
                    .collect(),
            ),
            Operation::Call(def, arg_valuees) => Operation::Call(
                def,
                arg_valuees
                    .iter()
                    .map(|arg_value| {
                        *body_map
                            .get(arg_value)
                            .unwrap_or(arg_value)
                    })
                    .collect(),
            ),
            Operation::If(cond, then_branch, else_branch) => Operation::If(
                *body_map.get(&cond).unwrap_or(&cond),
                *body_map.get(&then_branch).unwrap_or(&then_branch),
                *body_map.get(&else_branch).unwrap_or(&else_branch),
            ),
            Operation::For(iterators, body, opt_selector) => Operation::For(
                iterators
                    .iter()
                    .map(|iterator| {
                        *body_map.get(iterator).unwrap_or(iterator)
                    })
                    .collect(),
                *body_map.get(&body).unwrap_or(&body),
                opt_selector.map(|selector| {
                    *body_map
                        .get(&selector)
                        .unwrap_or(&selector)
                }),
            ),
            Operation::Fold(iterator, fold_op, init) => Operation::Fold(
                *body_map
                    .get(&iterator)
                    .unwrap_or(&iterator),
                fold_op,
                *body_map.get(&init).unwrap_or(&init),
            ),
            Operation::Enf(value) => {
                Operation::Enf(*body_map.get(&value).unwrap_or(&value))
            }
            Operation::Boundary(boundary, value) => Operation::Boundary(
                boundary,
                *body_map.get(&value).unwrap_or(&value),
            ),
            Operation::Variable(var) => Operation::Variable(var),
            Operation::Definition(_, _, _) => unreachable!(),
            Operation::Placeholder => Operation::Placeholder,
        };
        ir.update_node(&new_node, op);
        new_body.push(new_node);
    }
}
