use core::panic;
use std::collections::HashMap;

use air_pass::Pass;
//use miden_diagnostics::DiagnosticsHandler;

use crate::{
    ir::{Call, Graph, Link, Mir, Node, Op, Parent, Root, Vector},
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
#[derive(Clone, Debug)]
pub struct CallInliningContext {
    body: Link<Vec<Link<Op>>>,
    arguments: Link<Vec<Link<Op>>>,
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
    in_func_or_eval: bool,
    // When encountering a call, we store it here to construct the dependency graph once reaching the root node
    current_callees_encountered: Vec<Link<Root>>,
    // HashMap<Function, Functions where it is called>
    func_eval_dependency_graph: HashMap<Link<Root>, Vec<Link<Root>>>,
    // HashMap<Callee, Vec<Call nodes where called>>
    func_eval_nodes_where_called: HashMap<Link<Root>, Vec<Link<Call>>>,
}
impl InliningFirstPass {
    pub fn new() -> Self {
        Self {
            work_stack: vec![],
            in_func_or_eval: false,
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

    // HashMap<Callee, Vec<Call nodes where called>>
    func_eval_nodes_where_called: HashMap<Link<Root>, Vec<Link<Call>>>,
}
impl InliningSecondPass {
    pub fn new(
        func_eval_inlining_order: Vec<Link<Root>>,
        func_eval_nodes_where_called: HashMap<Link<Root>, Vec<Link<Call>>>,
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

        println!("****************************");
        println!("Starting first INLINING pass");
        println!("****************************");
        println!("");

        // The first pass only identifies the call graph dependencies and the needed calls to inline
        Visitor::run(&mut first_pass, ir.constraint_graph_mut());

        let func_eval_inlining_order =
            create_inlining_order(first_pass.func_eval_dependency_graph.clone());

        println!("");
        println!("func_eval_inlining_order: {:?}", func_eval_inlining_order);
        println!("");

        let mut second_pass = InliningSecondPass::new(
            func_eval_inlining_order.clone(),
            first_pass.func_eval_nodes_where_called.clone(),
        );

        println!("****************************");
        println!("Starting second INLINING pass");
        println!("****************************");
        println!("");

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

        // Remove the function from the dependency graph
        func_eval_dependency_graph.iter_mut().for_each(|(_k, v)| {
            v.retain(|x| x != removed_fn);
        });
    }
    func_eval_inlining_order
}

impl Visitor for InliningFirstPass {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }
    fn run(&mut self, graph: &mut Graph) {
        for root in self.root_nodes_to_visit(graph) {
            if let Some(_function) = root.clone().as_function() {
                self.in_func_or_eval = true;
            } else if let Some(_evaluator) = root.clone().as_evaluator() {
                self.in_func_or_eval = true;
            } else {
                self.in_func_or_eval = false;
            }

            self.scan_node(graph, root.clone());
            while let Some(node) = self.work_stack().pop() {
                self.visit_node(graph, node);
            }
        }
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
        self.func_eval_dependency_graph
            .insert(function.as_root(), self.current_callees_encountered.clone());
        self.current_callees_encountered.clear();
    }
    fn visit_evaluator(&mut self, _graph: &mut Graph, evaluator: Link<crate::ir::Evaluator>) {
        self.func_eval_dependency_graph.insert(
            evaluator.as_root(),
            self.current_callees_encountered.clone(),
        );
        self.current_callees_encountered.clear();
    }
    fn visit_call(&mut self, _graph: &mut Graph, call: Link<crate::ir::Call>) {
        let callee = call.borrow().function.clone();

        if self.in_func_or_eval {
            self.current_callees_encountered.push(callee.clone());
        }

        self.func_eval_nodes_where_called
            .entry(callee.clone())
            .and_modify(|v| v.push(call.clone()))
            .or_insert(vec![call.clone()]);
    }
}

impl InliningSecondPass {}

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
                        .into_iter()
                        .map(|call| call.clone().as_node()),
                );
            }
        }
        return callee_nodes_to_inline_in_order;
    }
    fn run(&mut self, graph: &mut Graph) {
        for (idx, root) in self.root_nodes_to_visit(graph).iter().enumerate() {
            println!("Visiting root node: {idx} - {:?}", root);
            println!("");

            // Set context for inlining this call
            let Some(call_node) = root.clone().as_call() else {
                unreachable!("InliningSecondPass::run: root node is not a Call node");
            };

            let callee = call_node.borrow().function.clone();
            let arguments = call_node.borrow().arguments.clone();
            let (pure_function, body) = if let Some(f) = callee.clone().as_function() {
                (true, f.borrow().body.clone())
            } else if let Some(ev) = callee.clone().as_evaluator() {
                (false, ev.borrow().body.clone())
            } else {
                unreachable!(
                    "InliningSecondPass::run: callee is not a Function or an Evaluator node"
                );
            };

            let context = CallInliningContext {
                body,
                arguments,
                pure_function,
            };

            println!("SET NEW CONTEXT: {:?}", context);
            println!("");

            self.call_inlining_context = Some(context.clone());
            self.nodes_to_replace.clear();

            self.scan_node(graph, root.clone());

            let mut ind = 0;

            while let Some(node) = self.work_stack().pop() {
                ind += 1;
                self.visit_node(graph, node);
                if ind > 500 {
                    unreachable!("InliningSecondPass::run: too many iterations");
                }
            }

            println!("END Visiting root node: {idx} - {:?}", root);

            if context.pure_function {
                // We have finished inlining the body, we can now replace the Call node with the last expression of the body
                let last_child_of_body = context.body.borrow().last().unwrap().clone();

                println!("BEFORE update call node: {:?}", root);
                let new_node = self
                    .nodes_to_replace
                    .get(&last_child_of_body)
                    .unwrap()
                    .clone();

                *root.borrow_mut() = new_node.as_node().borrow().clone();

                println!("Updated call node: {:?}", root);
                println!("");
            } else {
                // We have finished inlining the body, we can now replace the Call node with all the body
                let mut new_nodes = Vec::new();
                for body_node in context.body.borrow().iter() {
                    // FIXME: Maybe we should only push nodes that are Enf()?
                    // Depends if additional nodes change things (e.g. the Vector size..)
                    // For now I think we can keep all nodes, and just ignore the non-Enf nodes
                    // When building the constraints during lowering Mir -> Air
                    new_nodes.push(self.nodes_to_replace.get(&body_node).unwrap().clone());
                }
                let new_nodes_vector = Vector::create(new_nodes).as_op();

                *root.clone().borrow_mut() = new_nodes_vector.as_node().borrow().clone();

                println!("");
                println!("Updated call node: {:?}", root);
                println!("");
            }

            // Reset context to None
            self.call_inlining_context = None;
        }
    }
    fn scan_node(&mut self, _graph: &Graph, node: Link<Node>) {
        self.work_stack().push(node.clone());
        if let Some(_owner) = node.clone().as_owner() {
            // If we visit a Call, do not visit the children (the call's arguments)
            // TODO INLINING: Check whether we should instead
            if let Some(_call) = node.clone().as_call() {
                return;
            }
            for child in node.children().borrow().iter() {
                self.scan_node(_graph, child.clone().as_node());
            }
        }
    }

    fn visit_call(&mut self, _graph: &mut Graph, _call: Link<Call>) {
        let Some(context) = self.call_inlining_context.clone() else {
            unreachable!("InliningSecondPass::visit_node: call_inlining_context is None");
        };
        if context.pure_function {
            // Instead of scanning all the body, we only scan the last node,
            // which represents the return value of the function

            println!("    Visiting call node: {:?}", _call);
            println!("");
            println!("    Context in visit_call: {:?}", context.clone());
            println!("");
            println!(
                "    Scaning body: {:?}",
                context.body.borrow().last().unwrap().clone().as_node()
            );
            println!("");

            self.scan_node(
                _graph,
                context.body.borrow().last().unwrap().clone().as_node(),
            );
        } else {
            // We scan all the nodes related to the body
            for body_node in context.body.borrow().iter() {
                self.scan_node(_graph, body_node.clone().as_node());
            }
        }
    }

    fn visit_node(&mut self, graph: &mut Graph, node: Link<Node>) {
        /*if self.updated_nodes.contains(&node) {
            println!("        encountering a node we've updated! {:?}", node);
        } else {
            println!("        encountering a node we haven't updated! {:?}", node);
        }
        println!("");*/

        // First, check if it's a known Call to inline,
        // if so, set the context and visit the body

        if let Some(call_node) = node.clone().as_call() {
            self.visit_call(graph, call_node.clone());
        } else {
            if let Some(op) = node.clone().as_op() {
                duplicate_node_or_replace(
                    &mut self.nodes_to_replace,
                    op,
                    self.call_inlining_context
                        .clone()
                        .unwrap()
                        .arguments
                        .borrow()
                        .clone(),
                );
            } else {
                unreachable!(
                    "InliningSecondPass::visit_node on a non-Op node: {:?}",
                    node
                );
            }
        }
    }
}
