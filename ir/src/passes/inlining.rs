use std::{
    collections::{BTreeMap, HashMap, HashSet},
    ops::{ControlFlow, Deref, DerefMut},
    vec,
};

use air_pass::Pass;
//use miden_diagnostics::DiagnosticsHandler;

use crate::{
    ir2::{Graph, Link, MiddleNode, Mir, NodeType},
    CompileError,
};

use super::{duplicate_node_or_replace, visitor::VisitDefault, Visit, VisitContext, VisitOrder};

//pub struct Inlining<'a> {
//     #[allow(unused)]
//     diagnostics: &'a DiagnosticsHandler,
//}

#[derive(Clone)]
pub struct CallInliningContext {
    body: Link<NodeType>,
    arguments: Vec<Link<NodeType>>,
    call_node: Link<NodeType>,
}

impl CallInliningContext {}

pub struct Inlining {
    // general context
    work_stack: Vec<Link<NodeType>>,
    during_first_pass: bool,

    // context for first pass
    currently_in_body_of: Option<Link<NodeType>>,
    // HashMap<Definition, Functions_where_called>
    func_eval_dependency_graph: HashMap<Link<NodeType>, Vec<Link<NodeType>>>,

    // context for both passes
    func_eval_inlining_order: Vec<Link<NodeType>>,
    // HashMap<Definition, Call nodes where called, along with context>
    func_eval_nodes_where_called: HashMap<Link<NodeType>, Vec<CallInliningContext>>,

    // context for second pass
    call_inlining_context: Option<CallInliningContext>,
    nodes_to_replace: HashMap<Link<NodeType>, Link<NodeType>>,
}

impl VisitContext for Inlining {
    type Graph = Graph;
    fn visit(&mut self, graph: &mut Graph, node: Link<NodeType>) {
        if self.during_first_pass {
            self.visit_first_pass(graph, node);
        } else {
            self.visit_second_pass(graph, node);
        }
    }
    fn as_stack_mut(&mut self) -> &mut Vec<Link<NodeType>> {
        &mut self.work_stack
    }

    // FIXME: Clean this up
    // Maybe go for an approach for "root_nodes_to_visit", to keep a consistent context
    fn boundary_roots(&self, graph: &Graph) -> Link<Vec<Link<NodeType>>> {
        if self.during_first_pass {
            return Link::new(
                graph
                    .get_function_nodes()
                    .iter()
                    .chain(graph.get_evaluator_nodes().iter())
                    .chain(graph.boundary_constraints_roots.borrow().deref().iter())
                    .cloned()
                    .collect(),
            );
        } else {
            let mut callee_nodes_to_inline_in_order = Vec::new();
            for callee in self.func_eval_inlining_order.iter() {
                if let Some(nodes_with_context) = self.func_eval_nodes_where_called.get(callee) {
                    callee_nodes_to_inline_in_order.extend(
                        nodes_with_context
                            .iter()
                            .map(|context| context.call_node.clone()),
                    );
                }
            }
            return Link::new(callee_nodes_to_inline_in_order);
        }
    }
    fn integrity_roots(&self, graph: &Graph) -> Link<Vec<Link<NodeType>>> {
        if self.during_first_pass {
            return graph.integrity_constraints_roots.clone();
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

impl VisitDefault for Inlining {}

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

    fn visit_first_pass(&mut self, graph: &Graph, node: Link<NodeType>) {
        let funcs_and_evaluators: Vec<_> = graph
            .get_function_nodes()
            .iter()
            .chain(graph.get_evaluator_nodes().iter())
            .cloned()
            .collect();
        let boundary_and_integrity_roots: Vec<_> = graph
            .boundary_constraints_roots
            .borrow()
            .deref()
            .iter()
            .chain(graph.integrity_constraints_roots.borrow().deref().iter())
            .cloned()
            .collect();
        if funcs_and_evaluators.contains(&node) {
            self.currently_in_body_of = Some(node);
        }
        if boundary_and_integrity_roots.contains(&node) {
            self.currently_in_body_of = None;
        }

        if let NodeType::MiddleNode(MiddleNode::Call(call)) = node.borrow().deref() {
            // /!\ callee should not be in children to avoid loops IMO
            let callee = call.function();
            let args = call.arguments();

            if let Some(body_of) = self.currently_in_body_of {
                // The current function's body has a call to callee
                self.func_eval_dependency_graph
                    .entry(body_of)
                    .and_modify(|v| v.push(callee))
                    .or_insert(vec![callee]);
            }

            let NodeType::MiddleNode(MiddleNode::Function(func)) = callee.borrow().deref() else {
                unreachable!();
            };

            let context = CallInliningContext {
                body: func.body(),
                arguments: args.clone(),
                call_node: node.clone(),
            };
            self.func_eval_nodes_where_called
                .entry(callee)
                .and_modify(|v| v.push(context))
                .or_insert(vec![context]);
        }
    }

    fn visit_second_pass(&mut self, graph: &Graph, node: Link<NodeType>) {
        // First, check if it's a known Call to inline,
        // if so, set the context and visit the body
        let mut new_call_inlining_context = None;
        for (callee, calls) in self.func_eval_nodes_where_called.iter_mut() {
            calls.retain_mut(|context| {
                if context.call_node == node {
                    new_call_inlining_context = Some(context.clone());
                    false
                } else {
                    true
                }
            });
        }

        match new_call_inlining_context {
            Some(context) => {
                self.call_inlining_context = Some(context);
                self.nodes_to_replace.clear();
                self.visit_later(context.body);
            }
            None => {
                // Normal visit, insert in the graph the same instruction
                duplicate_node_or_replace(
                    &mut self.nodes_to_replace,
                    node,
                    self.call_inlining_context.unwrap().arguments,
                );

                if node == self.call_inlining_context.unwrap().body {
                    // We have finished inlining the body, we can now replace the node in the current index of the parent For
                    let new_node = self.nodes_to_replace.get(&node).unwrap().clone();
                    *self
                        .call_inlining_context
                        .unwrap()
                        .call_node
                        .borrow_mut()
                        .deref_mut() = new_node;
                }
            }
        }
    }
}
