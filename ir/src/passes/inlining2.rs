use std::{collections::HashMap, ops::Deref};

use air_pass::Pass;
use miden_diagnostics::{DiagnosticsHandler, Severity, SourceSpan};

use crate::{
    ir::{
        Graph, Link, Mir, MirType, MirValue, Node, Op, Parameter, Parent, Root, SpannedMirValue,
        TraceAccessBinding, Value, Vector,
    },
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
    ref_node: Link<Node>,
}
impl CallInliningContext {}

pub struct Inlining<'a> {
    diagnostics: &'a DiagnosticsHandler,
}
impl<'a> Inlining<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}

pub struct InliningFirstPass<'a> {
    diagnostics: &'a DiagnosticsHandler,

    // general context
    work_stack: Vec<Link<Node>>,
    in_func_or_eval: bool,
    // When encountering a call, we store it here to construct the dependency graph once reaching the root node
    current_callees_encountered: Vec<Link<Root>>,
    // HashMap<FunctionPtr, (Function, Functions where it is called)>
    func_eval_dependency_graph: HashMap<usize, (Link<Root>, Vec<Link<Root>>)>,
    // HashMap<CaleePtr, Callee, Vec<Call nodes where called>>
    func_eval_nodes_where_called: HashMap<usize, (Link<Root>, Vec<Link<Op>>)>, // Op is a Call here
}
impl<'a> InliningFirstPass<'a> {
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self {
            diagnostics,
            work_stack: vec![],
            in_func_or_eval: false,
            current_callees_encountered: Vec::new(),
            func_eval_dependency_graph: HashMap::new(),
            func_eval_nodes_where_called: HashMap::new(),
        }
    }
}

pub struct InliningSecondPass<'a> {
    diagnostics: &'a DiagnosticsHandler,

    // general context
    work_stack: Vec<Link<Node>>,
    // context for both passes
    func_eval_inlining_order: Vec<Link<Root>>,
    // context for second pass
    call_inlining_context: Option<CallInliningContext>,
    // HashMap<KeyPtr, (Key, Value)>
    nodes_to_replace: HashMap<usize, (Link<Op>, Link<Op>)>,

    // HashMap<CaleePtr, (Callee, Vec<Call nodes where called>)>
    func_eval_nodes_where_called: HashMap<usize, (Link<Root>, Vec<Link<Op>>)>, // Op is a Call here
}
impl<'a> InliningSecondPass<'a> {
    pub fn new(
        diagnostics: &'a DiagnosticsHandler,
        func_eval_inlining_order: Vec<Link<Root>>,
        func_eval_nodes_where_called: HashMap<usize, (Link<Root>, Vec<Link<Op>>)>,
    ) -> Self {
        Self {
            diagnostics,
            work_stack: vec![],
            call_inlining_context: None,
            nodes_to_replace: HashMap::new(),
            func_eval_nodes_where_called,
            func_eval_inlining_order,
        }
    }
}

impl Pass for Inlining<'_> {
    type Input<'a> = Mir;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, mut ir: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut first_pass = InliningFirstPass::new(self.diagnostics);

        /*println!("****************************");
        println!("Starting first INLINING pass");
        println!("****************************");*/

        // The first pass only identifies the call graph dependencies and the needed calls to inline
        Visitor::run(&mut first_pass, ir.constraint_graph_mut())?;

        let func_eval_inlining_order = create_inlining_order(
            self.diagnostics,
            first_pass.func_eval_dependency_graph.clone(),
        )?;

        //println!("func_eval_inlining_order: {:?}", func_eval_inlining_order);

        let mut second_pass = InliningSecondPass::new(
            self.diagnostics,
            func_eval_inlining_order.clone(),
            first_pass.func_eval_nodes_where_called.clone(),
        );

        /*println!("****************************");
        println!("Starting second INLINING pass");
        println!("****************************");*/

        // The second pass actually inlines the calls
        Visitor::run(&mut second_pass, ir.constraint_graph_mut())?;
        Ok(ir)
    }
}

fn create_inlining_order(
    diagnostics: &DiagnosticsHandler,
    mut func_eval_dependency_graph: HashMap<usize, (Link<Root>, Vec<Link<Root>>)>,
) -> Result<Vec<Link<Root>>, CompileError> {
    let mut func_eval_inlining_order = Vec::new();

    // Note: we remove an element at each iteration (or raise diag), so this will terminate
    while !func_eval_dependency_graph.is_empty() {
        // Find a function without dependency
        match func_eval_dependency_graph
            .clone()
            .iter()
            .find(|(_, (k, v))| v.is_empty())
        {
            Some((f_ptr, (f, _))) => {
                func_eval_inlining_order.push(f.clone());
                func_eval_dependency_graph.remove(f_ptr);
            }
            _ => {
                //panic!("Circular dependency detected!"); // Circular dep?, raise diag
                diagnostics
                    .diagnostic(Severity::Error)
                    .with_message("argument count mismatch")
                    .emit();
                return Err(CompileError::Failed);
            }
        }

        let removed_fn = func_eval_inlining_order.last().unwrap();

        // Remove the function from the dependency graph
        func_eval_dependency_graph
            .iter_mut()
            .for_each(|(_, (_, v))| {
                v.retain(|x| x != removed_fn);
            });
    }
    Ok(func_eval_inlining_order)
}

impl Visitor for InliningFirstPass<'_> {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }
    fn run(&mut self, graph: &mut Graph) -> Result<(), CompileError> {
        for root_node in self.root_nodes_to_visit(graph) {
            if let Some(root) = root_node.as_root() {
                if let Some(_function) = root.clone().as_function() {
                    self.in_func_or_eval = true;
                } else if let Some(_evaluator) = root.clone().as_evaluator() {
                    self.in_func_or_eval = true;
                } else {
                    unreachable!("Encountered a root node that is not a Function or an Evaluator");
                }
            } else {
                self.in_func_or_eval = false;
            }

            self.scan_node(graph, root_node.clone())?;
            while let Some(node) = self.work_stack().pop() {
                self.visit_node(graph, node)?;
            }
        }
        Ok(())
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
    fn visit_function(
        &mut self,
        _graph: &mut Graph,
        function: Link<Root>,
    ) -> Result<(), CompileError> {
        self.func_eval_dependency_graph.insert(
            function.get_ptr(),
            (function, self.current_callees_encountered.clone()),
        );
        self.current_callees_encountered.clear();
        Ok(())
    }
    fn visit_evaluator(
        &mut self,
        _graph: &mut Graph,
        evaluator: Link<Root>,
    ) -> Result<(), CompileError> {
        self.func_eval_dependency_graph.insert(
            evaluator.get_ptr(),
            (evaluator.clone(), self.current_callees_encountered.clone()),
        );
        self.current_callees_encountered.clear();
        Ok(())
    }
    fn visit_call(&mut self, _graph: &mut Graph, call: Link<Op>) -> Result<(), CompileError> {
        // safe to unwrap because we just dispatched on it
        let callee = &call.as_call().unwrap().function;
        if self.in_func_or_eval {
            self.current_callees_encountered.push(callee.clone());
        }
        self.func_eval_nodes_where_called
            .entry(callee.get_ptr())
            .and_modify(|(_, v)| v.push(call.clone()))
            .or_insert((callee.clone(), vec![call.clone()]));
        Ok(())
    }
}

impl Visitor for InliningSecondPass<'_> {
    fn work_stack(&mut self) -> &mut Vec<Link<Node>> {
        &mut self.work_stack
    }
    fn root_nodes_to_visit(&self, _graph: &Graph) -> Vec<Link<Node>> {
        let mut callee_nodes_to_inline_in_order = Vec::new();
        for callee in self.func_eval_inlining_order.iter() {
            if let Some((_, nodes_with_context)) =
                self.func_eval_nodes_where_called.get(&callee.get_ptr())
            {
                callee_nodes_to_inline_in_order
                    .extend(nodes_with_context.iter().map(|call| call.clone().as_node()));
            }
        }
        callee_nodes_to_inline_in_order
    }
    fn run(&mut self, graph: &mut Graph) -> Result<(), CompileError> {
        for root_node in self.root_nodes_to_visit(graph).iter() {
            //println!("Visiting root node: {idx} - {:?}", root_node);
            let mut updated_op = None;

            if let Some(op) = root_node.as_op() {
                // Set context for inlining this call
                let Some(call_node) = op.as_call() else {
                    return Ok(());
                };

                let callee = call_node.function.clone();
                let arguments = call_node.arguments.clone();
                let (pure_function, body) = if let Some(f) = callee.clone().as_function() {
                    (true, f.body.clone())
                } else if let Some(ev) = callee.clone().as_evaluator() {
                    (false, ev.body.clone())
                } else {
                    unreachable!(
                        "InliningSecondPass::run: callee is not a Function or an Evaluator node"
                    );
                };

                let context = CallInliningContext {
                    body,
                    arguments,
                    pure_function,
                    ref_node: callee.as_node(),
                };

                //println!("SET NEW CONTEXT: {:?}", context);

                self.call_inlining_context = Some(context.clone());
                self.nodes_to_replace.clear();

                self.scan_node(graph, root_node.clone())?;

                let mut ind = 0;

                while let Some(node) = self.work_stack().pop() {
                    ind += 1;
                    self.visit_node(graph, node)?;
                    if ind > 50 {
                        unreachable!("InliningSecondPass::run: too many iterations");
                    }
                }

                //println!("END Visiting root node: {idx} - {:?}", root_node);

                if context.pure_function {
                    // We have finished inlining the body, we can now replace the Call node with the last expression of the body
                    let last_child_of_body = context.body.borrow().last().unwrap().clone();

                    //println!("BEFORE update call node: {:?}", root_node);
                    let (_, new_node) = self
                        .nodes_to_replace
                        .get(&last_child_of_body.get_ptr())
                        .unwrap()
                        .clone();

                    updated_op = Some(new_node);

                    //println!("Updated call node: {:?}", root_node);
                } else {
                    // We have finished inlining the body, we can now replace the Call node with all the body
                    let mut new_nodes = Vec::new();
                    for body_node in context.body.borrow().iter() {
                        // FIXME: Maybe we should only push nodes that are Enf()?
                        // Depends if additional nodes change things (e.g. the Vector size..)
                        // For now I think we can keep all nodes, and just ignore the non-Enf nodes
                        // When building the constraints during lowering Mir -> Air
                        new_nodes.push(
                            self.nodes_to_replace
                                .get(&body_node.get_ptr())
                                .unwrap()
                                .1
                                .clone(),
                        );
                    }
                    let new_nodes_vector = Vector::create(new_nodes);

                    updated_op = Some(new_nodes_vector);

                    //println!("Updated call node: {:?}", root_node);
                }

                // Reset context to None
                self.call_inlining_context = None;
            }

            if let Some(updated_op) = updated_op {
                root_node.as_op().unwrap().set(&updated_op);
            }
        }
        Ok(())
    }
    fn scan_node(&mut self, _graph: &Graph, node: Link<Node>) -> Result<(), CompileError> {
        self.work_stack().push(node.clone());
        if let Some(op) = node.clone().as_op() {
            // If we visit a Call, do not visit the children (the call's arguments)
            // TODO INLINING: Check whether we should instead
            if op.as_call().is_some() {
                return Ok(());
            };
            for child in node.children().borrow().iter() {
                self.scan_node(_graph, child.clone().as_node())?;
            }
        }
        Ok(())
    }

    fn visit_call(&mut self, _graph: &mut Graph, _call: Link<Op>) -> Result<(), CompileError> {
        let Some(context) = self.call_inlining_context.clone() else {
            unreachable!("InliningSecondPass::visit_node: call_inlining_context is None");
        };
        if context.pure_function {
            // Instead of scanning all the body, we only scan the last node,
            // which represents the return value of the function

            /*println!("    Visiting call node: {:?}", _call);
            println!("    Context in visit_call: {:?}", context.clone());
            println!(
                "    Scaning body: {:?}",
                context.body.borrow().last().unwrap().clone().as_node()
            );*/

            self.scan_node(
                _graph,
                context.body.borrow().last().unwrap().clone().as_node(),
            )?;
        } else {
            // We scan all the nodes related to the body
            for body_node in context.body.borrow().iter() {
                self.scan_node(_graph, body_node.clone().as_node())?;
            }
        }
        Ok(())
    }

    fn visit_node(&mut self, graph: &mut Graph, node: Link<Node>) -> Result<(), CompileError> {
        /*if self.updated_nodes.contains(&node) {
            println!("        encountering a node we've updated! {:?}", node);
        } else {
            println!("        encountering a node we haven't updated! {:?}", node);
        }
        println!("");*/
        if node.is_stale() {
            return Ok(());
        }
        // First, check if it's a known Call to inline,
        // if so, set the context and visit the body
        let call_op = node.clone().as_op().unwrap_or_else(|| {
            panic!(
                "InliningSecondPass::visit_node on a non-Op node: {:?}",
                node
            )
        });
        if call_op.clone().as_call().is_some() {
            self.visit_call(graph, call_op.clone())?;
        } else {
            /*println!("Visiting node: {:?}", node);
            println!("Nodes to replace: {:?}", self.nodes_to_replace);*/

            if self.call_inlining_context.clone().unwrap().pure_function {
                duplicate_node_or_replace(
                    &mut self.nodes_to_replace,
                    call_op,
                    self.call_inlining_context
                        .clone()
                        .unwrap()
                        .arguments
                        .borrow()
                        .clone(),
                    self.call_inlining_context.clone().unwrap().ref_node,
                );
            } else {
                // We unpack the arguments for all trace_segments first
                let args = self
                    .call_inlining_context
                    .clone()
                    .unwrap()
                    .arguments
                    .borrow()
                    .clone();

                let callee_params = self
                    .call_inlining_context
                    .clone()
                    .unwrap()
                    .ref_node
                    .as_root()
                    .unwrap()
                    .as_evaluator()
                    .unwrap()
                    .parameters
                    .clone();
                for ((trace_segment_id, trace_segments_params), trace_segments_arg) in
                    callee_params.iter().enumerate().zip(args.iter())
                {
                    let Some(trace_segments_arg_vector) = trace_segments_arg.as_vector() else {
                        unreachable!("expected vector, got {:?}", trace_segments_arg);
                    };
                    let children = trace_segments_arg_vector.children();
                    let mut trace_segments_arg_vector_len = 0;
                    for child in children.borrow().deref() {
                        if let Some(value) = child.as_value() {
                            let Value {
                                value: SpannedMirValue { value, .. },
                                ..
                            } = value.deref();

                            let param_size = match value {
                                MirValue::TraceAccessBinding(tab) => tab.size,
                                MirValue::TraceAccess(_) => 1,
                                _ => unreachable!("expected trace access binding, got {:?}", value),
                            };
                            trace_segments_arg_vector_len += param_size;
                        } else if let Some(parameter) = child.as_parameter() {
                            let Parameter {
                                ty,
                                position,
                                ref_node,
                                ..
                            } = parameter.deref();
                            let size = match ty {
                                MirType::Felt => 1,
                                MirType::Vector(len) => *len,
                                _ => unreachable!("expected felt or vector, got {:?}", ty),
                            };
                            trace_segments_arg_vector_len += size;
                        } else {
                            unreachable!("expected value or parameter, got {:?}", child);
                        }
                    }

                    if trace_segments_params.len() != trace_segments_arg_vector_len {
                        self.diagnostics
                            .diagnostic(Severity::Error)
                            .with_message("argument count mismatch")
                            .with_primary_label(
                                SourceSpan::UNKNOWN,
                                format!(
                                    "expected call to have {} arguments in trace segment {}, but got {}",
                                    trace_segments_params.len(),
                                    trace_segment_id,
                                    trace_segments_arg_vector_len
                                ),
                            )
                            .with_secondary_label(
                                SourceSpan::UNKNOWN,
                                format!(
                                    "this functions has {} parameters in trace segment {}",
                                    trace_segments_params.len(),
                                    trace_segment_id
                                ),
                            )
                            .emit();
                        return Err(CompileError::Failed);
                    }
                }

                let mut args_unpacked = Vec::new();
                for args_for_trace_segment in args.iter() {
                    let Some(trace_segment_vec) = args_for_trace_segment.as_vector() else {
                        unreachable!("Arguments of a Call node to Evaluator should be a Vectors for each trace segment");
                    };
                    let children = trace_segment_vec.children();
                    for arg in children.borrow().deref() {
                        if let Some(value) = arg.as_value() {
                            let Value {
                                value: SpannedMirValue { span, value, .. },
                                ..
                            } = value.deref();

                            match value {
                                MirValue::TraceAccessBinding(tab) => {
                                    if tab.size > 1 {
                                        for index in 0..tab.size {
                                            let new_arg = Value::create(SpannedMirValue {
                                                value: MirValue::TraceAccessBinding(
                                                    TraceAccessBinding {
                                                        size: 1,
                                                        segment: tab.segment,
                                                        offset: tab.offset + index,
                                                    },
                                                ),
                                                span: *span,
                                            });
                                            args_unpacked.push(new_arg);
                                        }
                                    } else {
                                        args_unpacked.push(arg.clone());
                                    }
                                }
                                MirValue::TraceAccess(_ta) => {
                                    args_unpacked.push(arg.clone());
                                }
                                _ => unreachable!(
                                    "expected trace access binding or trace access, got {:?}",
                                    value
                                ),
                            };
                        } else if let Some(parameter) = arg.as_parameter() {
                            let Parameter {
                                ty,
                                position,
                                ref_node,
                                ..
                            } = parameter.deref();

                            args_unpacked.push(arg.clone());
                        } else {
                            unreachable!("expected value or parameter, got {:?}", arg);
                        }
                    }
                }

                println!("args_unpacked: {:?}", args_unpacked);

                duplicate_node_or_replace(
                    &mut self.nodes_to_replace,
                    call_op,
                    args_unpacked,
                    self.call_inlining_context.clone().unwrap().ref_node,
                );
            }
        }
        Ok(())
    }
}
