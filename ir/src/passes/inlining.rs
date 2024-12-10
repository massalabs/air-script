use std::{
    collections::HashMap,
    ops::{ControlFlow, Deref, DerefMut},
    vec,
};

use air_pass::Pass;
//use miden_diagnostics::DiagnosticsHandler;

use crate::{
    ir3::{Graph, Link, Mir, Node, Op, Root},
    CompileError,
};

use super::{duplicate_node_or_replace, Visit, VisitContext, VisitOrder};

//pub struct Inlining<'a> {
//     #[allow(unused)]
//     diagnostics: &'a DiagnosticsHandler,
//}

#[derive(Clone)]
pub struct CallInliningContext {
    body: Vec<Link<Op>>,
    arguments: Vec<Link<Op>>,
    call_node: Link<Op>,
}

impl CallInliningContext {}

pub struct Inlining {
    // general context
    work_stack: Vec<Link<Node>>,
    during_first_pass: bool,

    // context for first pass
    currently_in_body_of: Option<Link<Root>>,
    // HashMap<Definition, Functions_where_called>
    func_eval_dependency_graph: HashMap<Link<Root>, Vec<Link<Root>>>,

    // context for both passes
    func_eval_inlining_order: Vec<Link<Root>>,
    // HashMap<Definition, Call nodes where called, along with context>
    func_eval_nodes_where_called: HashMap<Link<Root>, Vec<CallInliningContext>>,

    // context for second pass
    call_inlining_context: Option<CallInliningContext>,
    nodes_to_replace: HashMap<Link<Op>, Link<Op>>,
}

impl VisitContext for Inlining {
    type Graph = Graph;
    fn visit(&mut self, graph: &mut Graph, node: Link<Node>) {
        if self.during_first_pass {
            self.visit_first_pass(graph, node);
        } else {
            self.visit_second_pass(node);
        }
    }
    fn as_stack_mut(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }

    // FIXME: Clean this up
    // Maybe go for an approach for "root_nodes_to_visit", to keep a consistent context
    fn boundary_roots(&self, graph: &Graph) -> Link<Vec<Link<Node>>> {
        if self.during_first_pass {
            return Link::new(
                graph
                    .get_function_nodes()
                    .iter()
                    .cloned()
                    .map(|f| f.as_node())
                    .chain(
                        graph
                            .get_evaluator_nodes()
                            .iter()
                            .cloned()
                            .map(|ev| ev.as_node()),
                    )
                    .chain(
                        graph
                            .boundary_constraints_roots
                            .borrow()
                            .deref()
                            .iter()
                            .cloned()
                            .map(|bc| bc.as_node()),
                    )
                    .collect(),
            );
        } else {
            let mut callee_nodes_to_inline_in_order = Vec::new();
            for callee in self.func_eval_inlining_order.iter() {
                if let Some(nodes_with_context) = self.func_eval_nodes_where_called.get(callee) {
                    callee_nodes_to_inline_in_order.extend(
                        nodes_with_context
                            .iter()
                            .map(|context| context.call_node.clone().as_node()),
                    );
                }
            }
            return Link::new(callee_nodes_to_inline_in_order);
        }
    }
    fn integrity_roots(&self, graph: &Graph) -> Link<Vec<Link<Node>>> {
        if self.during_first_pass {
            return graph
                .boundary_constraints_roots
                .borrow()
                .deref()
                .iter()
                .cloned()
                .map(|ic| ic.as_node())
                .collect::<Vec<_>>()
                .into();
        } else {
            return Link::new(vec![]);
        }
    }
    fn visit_order(&self) -> VisitOrder {
        if self.during_first_pass {
            return super::VisitOrder::DepthFirst;
        } else {
            return super::VisitOrder::PostOrder;
        }
    }
}

//impl<'p> Pass for Inlining<'p> {}
impl Pass for Inlining {
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

impl Visit for Inlining {
    fn run(&mut self, graph: &mut Self::Graph) {
        // First pass, build the dependency graph
        self.during_first_pass = true;
        match self.visit_order() {
            VisitOrder::Manual => self.visit_manual(graph),
            VisitOrder::PostOrder => self.visit_postorder(graph),
            VisitOrder::DepthFirst => self.visit_depthfirst(graph),
        }
        while let Some(node) = self.next_node() {
            self.visit(graph, node);
        }

        let mut func_eval_dependancy_graph_clone = self.func_eval_dependency_graph.clone();
        // Note: we remove an element at each iteration (or raise diag), so this will terminate
        while !func_eval_dependancy_graph_clone.is_empty() {
            // Find a function without dependency
            match func_eval_dependancy_graph_clone
                .clone()
                .iter()
                .find(|(_k, v)| v.is_empty())
            {
                Some((f, _)) => {
                    self.func_eval_inlining_order.push(f.clone());
                    func_eval_dependancy_graph_clone.remove(f);
                }
                _ => {
                    // Circular dep, raise diag
                }
            }

            let removed_fn = self.func_eval_inlining_order.last().unwrap();

            // Remove the function from the dependancy graph
            func_eval_dependancy_graph_clone
                .iter_mut()
                .for_each(|(_k, v)| {
                    v.retain(|x| x != removed_fn);
                });
        }

        // Second pass, inline all Calls
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

//impl VisitDefault for Inlining {}

// impl<'a> Inlining<'a> {
//     pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
//         Self { diagnostics }
//         Self {}
//     }
// }
impl Inlining {
    pub fn new() -> Self {
        Self {
            work_stack: vec![],
            during_first_pass: true,
            func_eval_dependency_graph: HashMap::new(),
            nodes_to_replace: HashMap::new(),
            currently_in_body_of: None,
            func_eval_inlining_order: Vec::new(),
            func_eval_nodes_where_called: HashMap::new(),
            call_inlining_context: None,
        }
    }

    fn run_visitor(&mut self, ir: &mut Graph) -> ControlFlow<()> {
        Visit::run(self, ir);
        ControlFlow::Continue(())
    }

    fn visit_first_pass(&mut self, graph: &Graph, node: Link<Node>) {
        let funcs_and_evaluators: Vec<_> = graph
            .get_function_nodes()
            .iter()
            .cloned()
            .map(|f| f.as_node())
            .chain(
                graph
                    .get_evaluator_nodes()
                    .iter()
                    .cloned()
                    .map(|ev| ev.as_node()),
            )
            .collect();
        let boundary_and_integrity_roots: Vec<_> = graph
            .boundary_constraints_roots
            .borrow()
            .deref()
            .iter()
            .cloned()
            .map(|bc| bc.as_node())
            .chain(
                graph
                    .integrity_constraints_roots
                    .borrow()
                    .deref()
                    .iter()
                    .cloned()
                    .map(|ic| ic.as_node()),
            )
            .collect();
        if funcs_and_evaluators.contains(&node) {
            self.currently_in_body_of = Some(node.clone().as_root().unwrap());
        }
        if boundary_and_integrity_roots.contains(&node) {
            self.currently_in_body_of = None;
        }

        if let Some(call) = node.clone().as_call() {
            let call_ref = call.borrow();
            let call = call_ref.deref();

            // /!\ callee should not be in children to avoid loops IMO
            let callee = call.function.clone();
            let args = call.arguments.clone();

            if let Some(body_of) = self.currently_in_body_of.clone() {
                // The current function's body has a call to callee
                self.func_eval_dependency_graph
                    .entry(body_of)
                    .and_modify(|v| v.push(callee.clone()))
                    .or_insert(vec![callee.clone()]);
            }

            let callee_ref = callee.borrow();
            let Root::Function(func) = callee_ref.deref() else {
                unreachable!();
            };

            let context = CallInliningContext {
                body: func.body.borrow().deref().clone(),
                arguments: args.borrow().deref().clone(),
                call_node: node.as_op().unwrap(),
            };

            self.func_eval_nodes_where_called
                .entry(callee.clone())
                .and_modify(|v| v.push(context.clone()))
                .or_insert(vec![context]);
        }
    }

    fn visit_second_pass(&mut self, node: Link<Node>) {
        // First, check if it's a known Call to inline,
        // if so, set the context and visit the body
        let mut new_call_inlining_context = None;
        for (_callee, calls) in self.func_eval_nodes_where_called.iter_mut() {
            calls.retain_mut(|context| {
                if context.call_node == node.clone().as_op().unwrap() {
                    new_call_inlining_context = Some(context.clone());
                    false
                } else {
                    true
                }
            });
        }

        match new_call_inlining_context {
            Some(context) => {
                self.call_inlining_context = Some(context.clone());
                self.nodes_to_replace.clear();
                /*for body_node in context.body.iter() {
                    self.visit_later(body_node.as_node());
                }*/
                // Instead of visiting all the body, we only visit the last node,
                // which represents the return value of the function
                self.visit_later(context.body.last().unwrap().clone().as_node());
            }
            None => {
                // Normal visit, insert in the graph the same instruction
                duplicate_node_or_replace(
                    &mut self.nodes_to_replace,
                    node.clone().as_op().unwrap(),
                    self.call_inlining_context.clone().unwrap().arguments,
                );

                if let Some(op) = node.clone().as_op() {
                    if self
                        .call_inlining_context
                        .clone()
                        .unwrap()
                        .body
                        .contains(&op)
                    {
                        // We have finished inlining the body, we can now replace the Call node with the last expression of the body
                        let new_node = self
                            .nodes_to_replace
                            .get(&op)
                            .unwrap()
                            .borrow()
                            .deref()
                            .clone();
                        *self
                            .call_inlining_context
                            .as_mut()
                            .unwrap()
                            .call_node
                            .borrow_mut()
                            .deref_mut() = new_node;
                    }
                }
            }
        }
    }
}
