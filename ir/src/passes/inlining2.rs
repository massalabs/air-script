use std::{collections::HashMap, ops::{Deref, DerefMut}};

use air_pass::Pass;
//use miden_diagnostics::DiagnosticsHandler;

use crate::{
    ir::{Graph, Link, Mir, Node, Op, Root, Vector},
    CompileError,
};

use super::{duplicate_node_or_replace, visitor2::Visitor};

#[derive(Clone)]
pub struct CallInliningContext {
    body: Vec<Link<Op>>,
    arguments: Vec<Link<Op>>,
    call_node: Link<Op>,
    pure_function: bool,
}
impl CallInliningContext {}

pub struct Inlining {
    // context for each pass
    first_pass: InliningFirstPass,
    second_pass: InliningSecondPass,
}
impl Inlining {
    pub fn new() -> Self {
        Self {
            first_pass: InliningFirstPass::new(),
            second_pass: InliningSecondPass::new(),
        }
    }
}

pub struct InliningFirstPass {
    // general context
    work_stack: Vec<Link<Node>>,
    // When encountering a call, we store it here to construct the dependency graph once reaching the root node
    current_callees_encountered: Vec<Link<Root>>,
    // HashMap<Definition, Functions_where_called>
    func_eval_dependency_graph: HashMap<Link<Root>, Vec<Link<Root>>>,
    // HashMap<Definition, Call nodes where called, along with context>
    func_eval_nodes_where_called: HashMap<Link<Root>, Vec<CallInliningContext>>,

}
impl InliningFirstPass {
    pub fn new() -> Self {
        Self {
            work_stack: vec![],
            current_callees_encountered: Vec::new(),
            func_eval_dependency_graph: HashMap::new(),
            func_eval_nodes_where_called: HashMap::new(),
        }
    }
}

pub struct InliningSecondPass {
    // general context
    work_stack: Vec<Link<Node>>,
    // context for both passes
    func_eval_inlining_order: Vec<Link<Root>>,
    // context for second pass
    call_inlining_context: Option<CallInliningContext>,
    nodes_to_replace: HashMap<Link<Op>, Link<Op>>,
    
    // HashMap<Definition, Call nodes where called, along with context>
    func_eval_nodes_where_called: HashMap<Link<Root>, Vec<CallInliningContext>>,
}
impl InliningSecondPass {
    pub fn new() -> Self {
        Self {
            work_stack: vec![],
            call_inlining_context: None,
            nodes_to_replace: HashMap::new(),
            func_eval_nodes_where_called: HashMap::new(),
            func_eval_inlining_order: Vec::new(),
        }
    }
}

impl Pass for Inlining {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut pass = Inlining::new();
        // The first pass only identifies the call graph dependencies and the needed calls to inline
        Visitor::run(&mut pass.first_pass, ir.constraint_graph_mut());

        let func_eval_inlining_order = self.create_inlining_order();
        pass.second_pass.func_eval_inlining_order = func_eval_inlining_order.clone();
        println!("func_eval_inlining_order: {:?}", func_eval_inlining_order);

        // The second pass actually inlines the calls
        Visitor::run(&mut pass.second_pass, ir.constraint_graph_mut());
        Ok(ir)
    }
}

impl Inlining {

    fn create_inlining_order(&mut self) -> Vec<Link<Root>> {
        let mut func_eval_inlining_order = Vec::new();
        let mut func_eval_dependency_graph = self.first_pass.func_eval_dependency_graph.clone();
        // Note: we remove an element at each iteration (or raise diag), so this will terminate
        while !func_eval_dependency_graph.is_empty() {
            println!("dependancy graph has len: {:?}", func_eval_dependency_graph.len());
            println!("dependancy graph: {:?}", func_eval_dependency_graph);
            // Find a function without dependency
            match func_eval_dependency_graph
                .clone()
                .iter()
                .find(|(_k, v)| v.is_empty())
            {
                Some((f, _)) => {
                    func_eval_inlining_order.push(f.clone());
                    func_eval_dependency_graph.remove(f);
                }
                _ => {
                    println!("Circular dependency detected!");
                    // Circular dep?, raise diag
                }
            }

            let removed_fn = func_eval_inlining_order.last().unwrap();

            // Remove the function from the dependancy graph
            func_eval_dependency_graph
                .iter_mut()
                .for_each(|(_k, v)| {
                    v.retain(|x| x != removed_fn);
                });
        }
        func_eval_inlining_order
    }
}

impl Visitor for InliningFirstPass {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }
    fn root_nodes_to_visit(&self, graph: &Graph) -> Vec<Link<Node>> {
        let functions = graph.get_function_nodes();
        let evaluators = graph.get_evaluator_nodes();
        let boundary_constraints_roots_ref = graph.boundary_constraints_roots.borrow();
        let integrity_constraints_roots_ref = graph.integrity_constraints_roots.borrow();

        let combined_roots = functions
            .into_iter()
            .map(|f| f.as_node())
            .chain(evaluators.into_iter().map(|e| e.as_node()))
            .chain(boundary_constraints_roots_ref.clone().into_iter().map(|bc| bc.as_node()))
            .chain(integrity_constraints_roots_ref.clone().into_iter().map(|ic| ic.as_node()));
        combined_roots.collect()
    }
    fn visit_function(&mut self, _graph: &mut Graph, function: Link<crate::ir::Function>) {
        println!("Currently in NEW body of function");
        self.func_eval_dependency_graph.insert(
            function.as_root(),
            self.current_callees_encountered.clone(),
        );
        self.current_callees_encountered.clear();
    }
    fn visit_evaluator(&mut self, _graph: &mut Graph, evaluator: Link<crate::ir::Evaluator>) {
        println!("Currently in NEW body of evaluator");
        self.func_eval_dependency_graph.insert(
            evaluator.as_root(),
            self.current_callees_encountered.clone(),
        );
        self.current_callees_encountered.clear();
    }
    fn visit_call(&mut self, _graph: &mut Graph, call: Link<crate::ir::Call>) {
        println!("Currently in call!");
        let callee = call.borrow().function.clone();
        let args = call.borrow().arguments.clone();

        self.current_callees_encountered.push(callee.clone());
        let callee_ref = callee.borrow();

        match callee_ref.deref() {
            Root::Evaluator(ev) => {
                let context = CallInliningContext {
                    body: ev.borrow().body.borrow().deref().clone(),
                    arguments: args.borrow().deref().clone(),
                    call_node: call.as_op(),
                    pure_function: false,
                };
                self.func_eval_nodes_where_called
                    .entry(callee.clone())
                    .and_modify(|v| v.push(context.clone()))
                    .or_insert(vec![context]);
            }
            Root::Function(func) => {
                let context = CallInliningContext {
                    body: func.borrow().body.borrow().deref().clone(),
                    arguments: args.borrow().deref().clone(),
                    call_node: call.as_op(),
                    pure_function: true,
                };
                self.func_eval_nodes_where_called
                    .entry(callee.clone())
                    .and_modify(|v| v.push(context.clone()))
                    .or_insert(vec![context]);
            }
            _ => unreachable!(),
        }
    }
}

impl Visitor for InliningSecondPass {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }
    fn root_nodes_to_visit(&self, _graph: &Graph) -> Vec<Link<Node>> {
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
        return callee_nodes_to_inline_in_order;
    }
    fn visit_node(&mut self, graph: &mut Graph, node: Link<Node>) {

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
                // New context, scan the body
                self.call_inlining_context = Some(context.clone());
                self.nodes_to_replace.clear();
                if context.pure_function {
                    // Instead of scanning all the body, we only scan the last node,
                    // which represents the return value of the function
                    self.scan_node(graph, context.body.last().unwrap().clone().as_node());
                } else {
                    // We scan all the nodes related to the body
                    for body_node in context.body.iter() {
                        self.scan_node(graph, body_node.clone().as_node());
                    }
                }
            }
            None => {
                // Normal visit, insert in the graph the same instruction
                duplicate_node_or_replace(
                    &mut self.nodes_to_replace,
                    node.clone().as_op().unwrap(),
                    self.call_inlining_context.clone().unwrap().arguments,
                );

                if let Some(op) = node.clone().as_op() {
                    // If we are at the last node of the body, we have visited all the body
                    // (for pure functions, we only visit the last node, and for evaluators, we visit all the nodes)
                    if self
                        .call_inlining_context
                        .clone()
                        .unwrap()
                        .body
                        .last()
                        .unwrap()
                        == &op
                    {
                        if self.call_inlining_context.clone().unwrap().pure_function {
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
                        } else {
                            // We have finished inlining the body, we can now replace the Call node with all the body
                            let mut new_nodes = Vec::new();
                            for body_node in self.call_inlining_context.clone().unwrap().body.iter()
                            {
                                // FIXME: Maybe we should only push nodes that are Enf()?
                                // Depends if additional nodes change things (e.g. the Vector size..)
                                // For now I think we can keep all nodes, and just ignore the non-Enf nodes
                                // When building the constraints during lowering Mir -> Air
                                new_nodes
                                    .push(self.nodes_to_replace.get(&body_node).unwrap().clone());
                            }
                            let new_nodes_vector = Vector::create(new_nodes).as_op();
                            *self
                                .call_inlining_context
                                .as_mut()
                                .unwrap()
                                .call_node
                                .borrow_mut()
                                .deref_mut() = new_nodes_vector.borrow().deref().clone();
                        }
                    }
                }
            }
        }
    }
}
