use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use air_pass::Pass;
//use miden_diagnostics::DiagnosticsHandler;

use crate::{
    ir::{Graph, Link, Mir, Node, Op, Parent, Root, Vector},
    CompileError,
};

use super::{duplicate_node_or_replace, visitor2::Visitor};

/// This pass handles inlining of Call nodes at there call sites.
///
/// It works in three steps:
/// * Firstly, we visit the graph to build the call dependency graph.
/// * This dependency graph is then used to compute the wanted inlining order
///   (we first replace calls to callees that do not have Calls in their body).
///   If it is not possible create this order, this means there is a circular dependency.
/// * Then, we visit the graph again at each Call nodes, building a duplicate of the body
///   (with Parameter replaced by call arguments), and replacing the Call node by this duplicate body.
///
/// TODO:
/// - [ ] Implement diagnostics for better error handling
///  
#[derive(Clone)]
pub struct CallInliningContext {
    body: Link<Vec<Link<Op>>>,
    arguments: Vec<Link<Op>>,
    call_node: Link<Op>,
    pure_function: bool,
}
impl CallInliningContext {}

pub struct Inlining {}
impl Inlining {
    pub fn new() -> Self {
        Self {}
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
    pub fn new(
        func_eval_inlining_order: Vec<Link<Root>>,
        func_eval_nodes_where_called: HashMap<Link<Root>, Vec<CallInliningContext>>,
    ) -> Self {
        Self {
            work_stack: vec![],
            call_inlining_context: None,
            nodes_to_replace: HashMap::new(),
            func_eval_nodes_where_called,
            func_eval_inlining_order,
        }
    }
}

impl Pass for Inlining {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut first_pass = InliningFirstPass::new();

        // The first pass only identifies the call graph dependencies and the needed calls to inline
        Visitor::run(&mut first_pass, ir.constraint_graph_mut());

        let func_eval_inlining_order =
            create_inlining_order(first_pass.func_eval_dependency_graph.clone());

        println!("func_eval_inlining_order: {:?}", func_eval_inlining_order);

        let mut second_pass = InliningSecondPass::new(
            func_eval_inlining_order.clone(),
            first_pass.func_eval_nodes_where_called.clone(),
        );

        // The second pass actually inlines the calls
        Visitor::run(&mut second_pass, ir.constraint_graph_mut());
        Ok(ir)
    }
}

fn create_inlining_order(
    mut func_eval_dependency_graph: HashMap<Link<Root>, Vec<Link<Root>>>,
) -> Vec<Link<Root>> {
    let mut func_eval_inlining_order = Vec::new();

    // Note: we remove an element at each iteration (or raise diag), so this will terminate
    while !func_eval_dependency_graph.is_empty() {
        //println!("Current dependancy graph has len: {:?}", func_eval_dependency_graph.len());

        /*for (k,v) in func_eval_dependency_graph.iter() {
            self.print_function(k, v);
        }*/

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
                panic!("Circular dependency detected!"); // Circular dep?, raise diag
            }
        }

        let removed_fn = func_eval_inlining_order.last().unwrap();

        /*if let Some(f) = removed_fn.clone().as_function() {
            println!("Removing function from dependency graph: Function of {:?} parameters, Value len", f.borrow().parameters.len());
        } else if let Some(e) = removed_fn.clone().as_evaluator() {
            println!("Removing evaluator from dependency graph: Evaluator of {:?} parameters, Value len", e.borrow().parameters.len());
        }*/

        // Remove the function from the dependency graph
        func_eval_dependency_graph.iter_mut().for_each(|(_k, v)| {
            v.retain(|x| x != removed_fn);
        });
    }
    func_eval_inlining_order
}

impl Inlining {
    /*fn print_function(&self, k: &Link<Root>, v: &Vec<Link<Root>>) {
        if let Some(f) = k.clone().as_function() {
            println!("Function of {:?} parameters, Value len: {:?}", f.borrow().parameters.len(), v.len());

            if f.borrow().parameters.len() == 12 {
                println!("Function of 12 parameters : {:?}", f);
            }
        } else if let Some(e) = k.clone().as_evaluator() {
            println!("Evaluator of {:?} parameters, Value len: {:?}", e.borrow().parameters.len(), v.len());
        }
        for callee in v {
            if let Some(f) = callee.clone().as_function() {
                println!("    Function of {:?} parameters", f.borrow().parameters.len());

                if f.borrow().parameters.len() == 12 {
                    println!("    Function of 12 parameters : {:?}", f);
                }
            } else if let Some(e) = callee.clone().as_evaluator() {
                println!("    Evaluator of {:?} parameters", e.borrow().parameters.len());
            }
        }
    }*/
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

        let combined_roots = boundary_constraints_roots_ref
            .clone()
            .into_iter()
            .map(|bc| bc.as_node())
            .chain(
                integrity_constraints_roots_ref
                    .clone()
                    .into_iter()
                    .map(|ic| ic.as_node()),
            )
            .chain(evaluators.into_iter().map(|e| e.as_node()))
            .chain(functions.into_iter().map(|f| f.as_node()));
        combined_roots.collect()
    }
    fn visit_function(&mut self, _graph: &mut Graph, function: Link<crate::ir::Function>) {
        println!("Currently in NEW body of function");
        self.func_eval_dependency_graph
            .insert(function.as_root(), self.current_callees_encountered.clone());
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
                    body: ev.borrow().body.clone(),
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
                    body: func.borrow().body.clone(),
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

    fn scan_node(&mut self, _graph: &Graph, node: Link<Node>) {
        // INLINING TODO:
        // - If we scan a Call node, set the context
        // - Check assumptions (e.g. we should never encounter a new Call node before fully finishing the current call's inlining)

        self.work_stack().push(node.clone());
        if let Some(_owner) = node.clone().as_owner() {
            if let Some(_call) = _owner.as_call() {
                return;
            }
            for child in node.children().borrow().iter() {
                self.scan_node(_graph, child.clone().as_node());
            }
        }
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
                    self.scan_node(
                        graph,
                        context.body.borrow().last().unwrap().clone().as_node(),
                    );
                } else {
                    // We scan all the nodes related to the body
                    for body_node in context.body.borrow().iter() {
                        self.scan_node(graph, body_node.clone().as_node());
                    }
                }
            }
            None => {
                // Normal visit, insert in the graph the same instruction

                // INLINING TODO:
                // - For evaluators, we need to flatten the arguments for main and aux to construct the replace_parameters_list,
                // see below
                /*if !context.pure_function {
                    let mut args = [];
                    for args in self.call_inlining_context.clone().unwrap().arguments.iter() {
                        args.push(args.clone().as_node());
                    }
                }*/

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
                        .borrow()
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
                            for body_node in self
                                .call_inlining_context
                                .clone()
                                .unwrap()
                                .body
                                .borrow()
                                .iter()
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
        //        core::cell::RefMut::deref_mut(&mut Link::new(0).borrow_mut());
        //        Link::new(0).borrow_mut().deref_mut();
        let obj = Link::new(vec![1, 2, 3]);
        core::cell::RefMut::deref_mut(&mut obj.borrow_mut());
    }
}
