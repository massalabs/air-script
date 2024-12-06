use std::ops::{Deref, DerefMut};

use air_parser::{ast::{self, AccessType, Identifier, QualifiedIdentifier}, symbols, LexicalScope, SemanticAnalysisError};
use air_pass::Pass;

use miden_diagnostics::{DiagnosticsHandler, SourceSpan, Spanned};

use crate::{ir2::*, CompileError, FoldOperator};

pub struct AstToMir<'a> {
    diagnostics: &'a DiagnosticsHandler,
}
impl<'a> AstToMir<'a> {
    /// Create a new instance of this pass
    #[inline]
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}
impl<'p> Pass for AstToMir<'p> {
    type Input<'a> = ast::Program;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, program: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut mir = Mir::new(program.name);

        //TODO MIR: Implement AST > MIR lowering
        // 1. Start from the previous lowering from AST to AIR
        // 2. Understand what changes when starting from an unoptimized AST
        // (with no constant prop and no inlining)
        // 3. Implement the needed changes

        let random_values = program.random_values;
        let trace_columns = program.trace_columns;
        let boundary_constraints = program.boundary_constraints;
        let integrity_constraints = program.integrity_constraints;

        mir.trace_columns = trace_columns.clone();
        mir.num_random_values = random_values.as_ref().map(|rv| rv.size as u16).unwrap_or(0);
        mir.periodic_columns = program.periodic_columns;
        mir.public_inputs = program.public_inputs;

        let mut builder = MirBuilder {
            diagnostics: self.diagnostics,
            mir: &mut mir,
            random_values,
            trace_columns,
            bindings: Default::default(),
        };

        Function::new()

        for (ident, func) in program.functions.iter() {
            builder.insert_function_body(ident, func)?;
        }

        for (ident, func) in program.evaluators.iter() {
            builder.insert_evaluator_function_body(ident, func)?;
        }

        for bc in boundary_constraints.iter() {
            builder.build_boundary_constraint(bc)?;
        }

        for ic in integrity_constraints.iter() {
            builder.build_integrity_constraint(ic)?;
        }

        Ok(mir)
    }
}

struct MirBuilder<'a> {
    #[allow(unused)]
    diagnostics: &'a DiagnosticsHandler,
    mir: &'a mut Mir,
    random_values: Option<ast::RandomValues>,
    trace_columns: Vec<ast::TraceSegment>,
    bindings: LexicalScope<Identifier, Link<NodeType>>,
}
impl<'a> MirBuilder<'a> {
    fn insert_evaluator_function_body(
        &mut self,
        ident: &QualifiedIdentifier,
        func: &ast::EvaluatorFunction,
    ) -> Result<(), CompileError> {

        let mut evaluator = Evaluator::default();

        let body = &func.body;
        let params = &func.params;

        for trace_segment in params.iter() {
            for param in trace_segment.bindings.iter() {
                println!("param: {:?}", param.name);
                println!("param offset: {:?}", param.offset);
            }
        }
        self.bindings.enter();
        for trace_segment in params.iter() {
            for binding in trace_segment.bindings.iter() {
                let spanned_mir_value = SpannedMirValue {
                    span: binding.span(),
                    value: MirValue::TraceAccessBinding(TraceAccessBinding {
                        segment: trace_segment.id,
                        offset: binding.offset,
                        size: binding.size,
                    }),
                };
                let spanned_mir_value_node: Link<NodeType> = spanned_mir_value.into();
                self.bindings.insert(binding.name.unwrap(), spanned_mir_value_node.clone());
                evaluator.add_arg(spanned_mir_value_node);
            }
        }

        // Insert the function body
        for stmt in body.iter() {
            self.build_function_body_statement(evaluator.clone().into(), stmt)?;
        }

        self.mir
            .constraint_graph_mut()
            .add_child(evaluator.clone().into());

        self.constraint_graph_mut().insert_evaluator(*ident, evaluator.into());

        self.bindings.exit();

        Ok(())
    }

    fn insert_function_body(
        &mut self,
        ident: &QualifiedIdentifier,
        func: &ast::Function,
    ) -> Result<(), CompileError> {

        let mut function = Function::default();

        let body = &func.body;
        let params = &func.params;

        self.bindings.enter();
        for (index, (ident, ty)) in params.iter().enumerate() {
            let param_node: Link<NodeType> = Parameter::new(ident.span(), (*ty).into(), index).into();
            self.bindings.insert(*ident, param_node.clone());
            function.add_arg(param_node);
        }

        let return_variable_node = Parameter::new(ident.span(), func.return_type.into(), 0);
        function.add_ret(return_variable_node.into());

        // Insert the function body
        for stmt in body.iter() {
            self.build_function_body_statement(function.clone().into(), stmt)?;
        }

        self.mir
            .constraint_graph_mut()
            .add_child(function.clone().into());

        self.constraint_graph_mut().insert_function(*ident, function.into());

        self.bindings.exit();

        Ok(())
    }

    fn build_boundary_constraint(&mut self, bc: &ast::Statement) -> Result<(), CompileError> {
        let parent = self.mir.constraint_graph().clone();
        self.build_statement(parent.into(), bc, true)
    }

    fn build_integrity_constraint(&mut self, ic: &ast::Statement) -> Result<(), CompileError> {
        let parent = self.mir.constraint_graph().clone();
        self.build_statement(parent.into(), ic, false)
    }

    fn build_function_body_statement(&mut self, parent: Link<NodeType>, s: &ast::Statement) -> Result<(), CompileError> {
        self.build_statement(parent, s, false)
    }

    fn build_statement(&mut self, parent: Link<NodeType>, c: &ast::Statement, in_boundary: bool) -> Result<(), CompileError> {
        match c {
            // If we have a let, update scoping and insert the body
            ast::Statement::Let(expr) => {
                self.build_let(expr, |bldr, stmt| bldr.build_statement(parent.clone(), stmt, in_boundary))
            }
            // Depending on the expression, we can have different types of operations in the
            // If we have a symbol access, we have to get it depending on the scope and add the
            // identifier to the graph nodes (SSA)
            ast::Statement::Expr(expr) => {
                let expr_node = self.insert_expr(expr)?;
                match parent.borrow_mut().deref_mut() {
                    NodeType::RootNode(root_node) => root_node.add_child(expr_node.into()),
                    NodeType::MiddleNode(middle_node) => middle_node.add_child(expr_node.into()),
                    NodeType::LeafNode(leaf_node) => leaf_node.add_child(expr_node.into()),
                };
                Ok(())
            }
            // Enforce statements can be translated to Enf operations in the MIR on scalar expressions
            ast::Statement::Enforce(scalar_expr) => {
                let scalar_expr_node = self.insert_scalar_expr(scalar_expr)?;

                // TODO: 
                match scalar_expr_node.clone().borrow().deref() {
                    NodeType::MiddleNode(MiddleNode::Enf(_enf)) => {
                        match in_boundary {
                            true => self.mir.constraint_graph_mut().insert_boundary_constraints_root(scalar_expr_node),
                            false => {
                                match parent.borrow_mut().deref_mut() {
                                    NodeType::RootNode(root_node) => root_node.add_child(scalar_expr_node.clone().into()),
                                    NodeType::MiddleNode(middle_node) => middle_node.add_child(scalar_expr_node.clone().into()),
                                    NodeType::LeafNode(leaf_node) => leaf_node.add_child(scalar_expr_node.clone().into()),
                                };
                                if parent == self.mir.constraint_graph().clone().into() {
                                    self.mir.constraint_graph_mut().insert_integrity_constraints_root(scalar_expr_node);
                                }
                            }
                        }
                    },
                    _ => {
                        let enf_node = Enf::new(scalar_expr_node).into();
                        match in_boundary {
                            true => self.mir.constraint_graph_mut().insert_boundary_constraints_root(enf_node),
                            false => {
                                match parent.borrow_mut().deref_mut() {
                                    NodeType::RootNode(root_node) => root_node.add_child(enf_node.clone().into()),
                                    NodeType::MiddleNode(middle_node) => middle_node.add_child(enf_node.clone().into()),
                                    NodeType::LeafNode(leaf_node) => leaf_node.add_child(enf_node.clone().into()),
                                };
                                if parent == self.mir.constraint_graph().clone().into() {
                                    self.mir.constraint_graph_mut().insert_integrity_constraints_root(enf_node);
                                }
                            }
                        }
                    }
                };
                Ok(())
            }
            ast::Statement::EnforceIf(_, _) => unreachable!(), // This variant was only available after AST's inlining, we should handle EnforceAll instead
            ast::Statement::EnforceAll(list_comprehension) => {
                self.bindings.enter();
                for (index, binding) in list_comprehension.bindings.iter().enumerate() {
                    let binding_node = Parameter::new(binding.span(), ast::Type::Felt.into(), index);
                    self.bindings.insert(*binding, binding_node.into());
                }

                let mut iterator_nodes = Vec::new();
                for iterator in list_comprehension.iterables.iter() {
                    let iterator_node = self.insert_expr(iterator)?;
                    iterator_nodes.push(iterator_node);
                }

                let selector_node = if let Some(selector) = &list_comprehension.selector {
                    Some(self.insert_scalar_expr(selector)?)
                } else {
                    None
                };
                let body_node = self.insert_scalar_expr(&list_comprehension.body)?;

                let for_node = For::new(
                    iterator_nodes,
                    body_node,
                    selector_node,
                );

                let enf_node = Enf::new(for_node.into());

                match in_boundary {
                    true => self.mir.constraint_graph_mut().insert_boundary_constraints_root(enf_node.into()),
                    false => {
                        match parent.borrow_mut().deref_mut() {
                            NodeType::RootNode(root_node) => root_node.add_child(enf_node.clone().into()),
                            NodeType::MiddleNode(middle_node) => middle_node.add_child(enf_node.clone().into()),
                            NodeType::LeafNode(leaf_node) => leaf_node.add_child(enf_node.clone().into()),
                        };
                        self.mir.constraint_graph_mut().insert_integrity_constraints_root(enf_node.into())
                    }
                }

                self.bindings.exit();
                Ok(())
            }
        }
    }

    fn build_let<F>(
        &mut self,
        expr: &ast::Let,
        mut statement_builder: F,
    ) -> Result<(), CompileError>
    where
        F: FnMut(&mut MirBuilder, &ast::Statement) -> Result<(), CompileError>,
    {
        let bound = self.insert_expr(&expr.value)?;
        self.bindings.enter();
        self.bindings.insert(expr.name, bound);
        for stmt in expr.body.iter() {
            statement_builder(self, stmt)?;
        }
        self.bindings.exit();
        Ok(())
    }

    fn insert_expr(&mut self, expr: &ast::Expr) -> Result<Link<NodeType>, CompileError> {
        match expr {
            ast::Expr::Const(span) => {
                let node = self.insert_typed_constant(Some(span.span()), span.item.clone());
                Ok(node)
            }
            ast::Expr::Range(range_expr) => {
                let values = range_expr.to_slice_range();
                let const_expr = ast::ConstantExpr::Vector(values.map(|v| v as u64).collect());
                let node = self.insert_typed_constant(Some(range_expr.span()), const_expr);
                Ok(node)
            }
            ast::Expr::Vector(spanned_vec) => {
                //let span = spanned_vec.span();
                if spanned_vec.len() == 0 {
                    return Ok(self.insert_typed_constant(None, ast::ConstantExpr::Vector(vec![])));
                }
                match spanned_vec.item[0].ty().unwrap() {
                    ast::Type::Felt => {
                        let mut nodes = vec![];
                        for value in spanned_vec.iter().cloned() {
                            let value = value.try_into().unwrap();
                            nodes.push(self.insert_scalar_expr(&value)?);
                        }
                        let node = Vector::new(nodes).into();
                        Ok(node)
                    }
                    /*ast::Type::Vector(n) => {
                        let mut nodes = vec![];
                        for row in spanned_vec.iter().cloned() {
                            nodes.push(self.insert_expr(&row)?);
                        }
                        let node = self.insert_op(Operation::Vector(nodes));
                        Ok(node)
                    }*/
                    ast::Type::Vector(n) => {
                        let mut nodes = vec![];
                        for row in spanned_vec.iter().cloned() {
                            match row {
                                ast::Expr::Const(const_expr) => {
                                    self.insert_typed_constant(
                                        Some(const_expr.span()),
                                        const_expr.item,
                                    );
                                }
                                // Rework based on Continuous Symbol Access in the MIR ?
                                ast::Expr::SymbolAccess(access) => {
                                    let mut cols = vec![];
                                    for i in 0..n {
                                        let node = match access.access_type {
                                            AccessType::Index(i) => {
                                                let access = ast::ScalarExpr::SymbolAccess(
                                                    access.access(AccessType::Index(i)).unwrap(),
                                                );
                                                self.insert_scalar_expr(&access)?
                                            }
                                            AccessType::Default => {
                                                let access = ast::ScalarExpr::SymbolAccess(
                                                    access.access(AccessType::Index(i)).unwrap(),
                                                );
                                                self.insert_scalar_expr(&access)?
                                            }
                                            AccessType::Slice(_range_expr) => todo!(),
                                            AccessType::Matrix(_, _) => todo!(),
                                        };

                                        cols.push(node);
                                    }
                                    nodes.push(cols);
                                }
                                ast::Expr::Vector(ref elems) => {
                                    let mut cols = vec![];
                                    for elem in elems.iter().cloned() {
                                        let elem: ast::ScalarExpr = elem.try_into().unwrap();
                                        let node = self.insert_scalar_expr(&elem)?;
                                        cols.push(node);
                                    }
                                    nodes.push(cols);
                                }
                                _ => unreachable!(),
                            }
                        }
                        let node = Matrix::new(nodes).into();
                        Ok(node)
                    }
                    _ => unreachable!(),
                }
            }
            ast::Expr::Matrix(values) => {
                let mut rows = Vec::with_capacity(values.len());
                for vs in values.iter() {
                    let mut cols = Vec::with_capacity(vs.len());
                    for value in vs {
                        cols.push(self.insert_scalar_expr(value)?);
                    }
                    rows.push(cols);
                }
                let node = Matrix::new(rows).into();
                Ok(node)
            }
            ast::Expr::SymbolAccess(access) => {
                // Should resolve the identifier depending on the scope, and add the access to the graph once it's resolved

                match self.bindings.get(access.name.as_ref()) {
                    None => {
                        // Must be a reference to a declaration
                        let node = self.insert_symbol_access(access);
                        Ok(node)
                    }
                    // Otherwise, this has been added to the bindings (function and list comprehensions params, let expr...)
                    Some(node) => Ok(node.clone()), /*Some(MemoizedBinding::Vector(nodes)) => {
                                                             let value = match &access.access_type {
                                                                 AccessType::Default => MemoizedBinding::Vector(nodes.clone()),
                                                                 AccessType::Index(idx) => MemoizedBinding::Scalar(nodes[*idx]),
                                                                 AccessType::Slice(range) => {
                                                                     MemoizedBinding::Vector(nodes[range.to_slice_range()].to_vec())
                                                                 }
                                                                 AccessType::Matrix(_, _) => unreachable!(),
                                                             };
                                                             Ok(value)
                                                         }
                                                         Some(MemoizedBinding::Matrix(nodes)) => {
                                                             let value = match &access.access_type {
                                                                 AccessType::Default => MemoizedBinding::Matrix(nodes.clone()),
                                                                 AccessType::Index(idx) => MemoizedBinding::Vector(nodes[*idx].clone()),
                                                                 AccessType::Slice(range) => {
                                                                     MemoizedBinding::Matrix(nodes[range.to_slice_range()].to_vec())
                                                                 }
                                                                 AccessType::Matrix(row, col) => {
                                                                     MemoizedBinding::Scalar(nodes[*row][*col])
                                                                 }
                                                             };
                                                             Ok(value)
                                                         }*/
                }
            }
            ast::Expr::Binary(binary_expr) => self.insert_binary_expr(binary_expr),
            ast::Expr::Call(call) => {
                // First, resolve the callee, panic if it's not resolved
                let resolved_callee = call.callee.resolved().unwrap();

                if call.is_builtin() {
                    // If it's a fold operator (Sum / Prod), handle it
                    match call.callee.as_ref().name() {
                        symbols::Sum => {
                            assert_eq!(call.args.len(), 1);
                            let iterator_node =
                                self.insert_expr(call.args.first().unwrap()).unwrap();
                            let accumulator_node =
                                self.insert_typed_constant(None, ast::ConstantExpr::Scalar(0));
                            let node = Fold::new(
                                iterator_node,
                                FoldOperator::Add,
                                accumulator_node,
                            ).into();
                            Ok(node)
                        }
                        symbols::Prod => {
                            assert_eq!(call.args.len(), 1);
                            let iterator_node =
                                self.insert_expr(call.args.first().unwrap()).unwrap();
                            let accumulator_node =
                                self.insert_typed_constant(None, ast::ConstantExpr::Scalar(1));
                            let node = Fold::new(
                                iterator_node,
                                FoldOperator::Mul,
                                accumulator_node,
                            ).into();
                            Ok(node)
                        }
                        other => unimplemented!("unhandled builtin: {}", other),
                    }
                } else {
                    let args_node: Vec<_> = call
                        .args
                        .iter()
                        .map(|arg| self.insert_expr(arg).unwrap())
                        .collect();

                    // Get the known callee in the functions hashmap
                    // Then, get the node index of the function definition
                    let callee_node = self
                        .mir
                        .constraint_graph()
                        .get_function(&resolved_callee)
                        .unwrap()
                        .clone();

                    let call_node = Call::new(callee_node, args_node).into();
                    Ok(call_node)
                }
            }
            ast::Expr::ListComprehension(list_comprehension) => {
                self.bindings.enter();
                for (index, binding) in list_comprehension.bindings.iter().enumerate() {
                    let binding_node = Parameter::new(binding.span(), ast::Type::Felt.into(), index);
                    self.bindings.insert(*binding, binding_node.into());
                }

                let mut iterator_nodes = Vec::new();
                for iterator in list_comprehension.iterables.iter() {
                    let iterator_node = self.insert_expr(iterator)?;
                    iterator_nodes.push(iterator_node);
                }

                let selector_node = if let Some(selector) = &list_comprehension.selector {
                    Some(self.insert_scalar_expr(selector)?)
                } else {
                    None
                };
                let body_node = self.insert_scalar_expr(&list_comprehension.body)?;

                let for_node = For::new(
                    iterator_nodes,
                    body_node,
                    selector_node,
                ).into();

                self.bindings.exit();
                Ok(for_node)
            }
            ast::Expr::Let(expr) => self.expand_let_expr(expr),
        }
    }

    fn expand_let_expr(&mut self, expr: &ast::Let) -> Result<Link<NodeType>, CompileError> {
        let mut next_let = Some(expr);
        let snapshot = self.bindings.clone();
        loop {
            let let_expr = next_let.take().expect("invalid empty let body");
            let bound = self.insert_expr(&let_expr.value)?;
            self.bindings.enter();
            self.bindings.insert(let_expr.name, bound);
            match let_expr.body.last().unwrap() {
                ast::Statement::Let(ref inner_let) => {
                    next_let = Some(inner_let);
                }
                ast::Statement::Expr(ref expr) => {
                    let value = self.insert_expr(expr);
                    self.bindings = snapshot;
                    break value;
                }
                ast::Statement::Enforce(_)
                | ast::Statement::EnforceIf(_, _)
                | ast::Statement::EnforceAll(_) => {
                    unreachable!()
                }
            }
        }
    }

    fn insert_scalar_expr(&mut self, expr: &ast::ScalarExpr) -> Result<Link<NodeType>, CompileError> {
        match expr {
            ast::ScalarExpr::Const(value) => {
                Ok(SpannedMirValue {
                    span: value.span(),
                    value: MirValue::Constant(ConstantValue::Felt(value.item)),
                }.into())
            }
            ast::ScalarExpr::SymbolAccess(access) => Ok(self.insert_symbol_access(access)),
            ast::ScalarExpr::Binary(expr) => self.insert_binary_expr(expr),
            ast::ScalarExpr::Let(ref let_expr) => {
                let index = self.expand_let_expr(let_expr)?;

                // TODO: Check that the resulting expr is a scalar expr
                Ok(index)
            }
            ast::ScalarExpr::Call(call) => {
                // First, resolve the callee, panic if it's not resolved
                let resolved_callee = call.callee.resolved().unwrap();

                if call.is_builtin() {
                    // If it's a fold operator (Sum / Prod), handle it
                    match call.callee.as_ref().name() {
                        symbols::Sum => {
                            assert_eq!(call.args.len(), 1);
                            let iterator_node =
                                self.insert_expr(call.args.first().unwrap()).unwrap();
                            let accumulator_node =
                                self.insert_typed_constant(None, ast::ConstantExpr::Scalar(0));
                            let node = Fold::new(
                                iterator_node,
                                FoldOperator::Add,
                                accumulator_node,
                            ).into();
                            Ok(node)
                        }
                        symbols::Prod => {
                            assert_eq!(call.args.len(), 1);
                            let iterator_node =
                                self.insert_expr(call.args.first().unwrap()).unwrap();
                            let accumulator_node =
                                self.insert_typed_constant(None, ast::ConstantExpr::Scalar(1));
                            let node = Fold::new(
                                iterator_node,
                                FoldOperator::Mul,
                                accumulator_node,
                            ).into();
                            Ok(node)
                        }
                        other => unimplemented!("unhandled builtin: {}", other),
                    }
                } else {
                    // If not, check if evaluator or function
                    let is_pure_function = self
                        .mir
                        .constraint_graph()
                        .get_function(&resolved_callee)
                        .is_some();

                    if is_pure_function {
                        let args_node: Vec<_> = call
                            .args
                            .iter()
                            .map(|arg| self.insert_expr(arg).unwrap())
                            .collect();
                        let callee_node = self
                            .mir
                            .constraint_graph()
                            .get_function(&resolved_callee)
                            .unwrap()
                            .clone();

                        // We can only check this once all bodies have been inserted
                        /*match self.mir.constraint_graph().node(&callee_node).op() {
                            Operation::Definition(_, Some(return_node), _) => {
                                match self.mir.constraint_graph().node(&return_node).op() {
                                    Operation::Variable(var) => {
                                        assert_eq!(
                                            var.ty,
                                            MirType::Felt,
                                            "Call to a function that does not return a scalar value"
                                        );
                                    }
                                    _ => unreachable!(),
                                }
                            },
                            _ => unreachable!(),
                        };*/
                        let call_node = Call::new(callee_node, args_node).into();
                        Ok(call_node)
                    } else {
                        let mut args_node = Vec::new();

                        for arg in call.args.iter() {
                            match arg {
                                ast::Expr::Vector(spanned_vec) => {
                                    let mut arg_node = Vec::new();
                                    for expr in spanned_vec.iter() {
                                        let expr_node = self.insert_expr(expr).unwrap();
                                        arg_node.push(expr_node);
                                    }
                                    let arg_node = Vector::new(arg_node).into();
                                    args_node.push(arg_node);
                                }
                                _ => unreachable!(),
                            }
                        }

                        let callee_node = self
                            .mir
                            .constraint_graph()
                            .get_evaluator(&resolved_callee)
                            .unwrap()
                            .clone();

                        // We can only check this once all bodies have been inserted
                        /*match self.mir.constraint_graph().node(&callee_node).op() {
                            Operation::Definition(_, None, _) => {},
                            op => {
                                println!("op: {:?}", op);
                                unreachable!();
                            },
                        };*/
                        let call_node = Call::new(callee_node, args_node).into();
                        Ok(call_node)
                    }
                }
            }
            ast::ScalarExpr::BoundedSymbolAccess(bsa) => Ok(self.insert_bounded_symbol_access(bsa)),
        }
    }

    // Use square and multiply algorithm to expand the exp into a series of multiplications
    fn expand_exp(&mut self, lhs: Link<NodeType>, rhs: u64, span: SourceSpan) -> Link<NodeType> {
        match rhs {
            0 => self.insert_typed_constant(Some(span), ast::ConstantExpr::Scalar(1)),
            1 => lhs,
            n if n % 2 == 0 => {
                let square = Mul::new(lhs.clone(), lhs).into();
                self.expand_exp(square, n / 2, span)
            }
            n => {
                let square = Mul::new(lhs.clone(), lhs.clone()).into();
                let rec = self.expand_exp(square, (n - 1) / 2, span);
                Mul::new(lhs, rec).into()
            }
        }
    }

    fn insert_binary_expr(&mut self, expr: &ast::BinaryExpr) -> Result<Link<NodeType>, CompileError> {
        if expr.op == ast::BinaryOp::Exp {
            let lhs = self.insert_scalar_expr(expr.lhs.as_ref())?;
            let ast::ScalarExpr::Const(rhs) = expr.rhs.as_ref() else {
                return Err(CompileError::SemanticAnalysis(
                    SemanticAnalysisError::InvalidExpr(ast::InvalidExprError::NonConstantExponent(
                        expr.rhs.span(),
                    )),
                ));
            };
            return Ok(self.expand_exp(lhs, rhs.item, expr.span()));
        }

        let lhs = self.insert_scalar_expr(expr.lhs.as_ref())?;
        let rhs = self.insert_scalar_expr(expr.rhs.as_ref())?;
        Ok(match expr.op {
            ast::BinaryOp::Add => Add::new(lhs, rhs).into(),
            ast::BinaryOp::Sub => Sub::new(lhs, rhs).into(),
            ast::BinaryOp::Mul => Mul::new(lhs, rhs).into(),
            ast::BinaryOp::Eq => {
                let sub_node = Sub::new(lhs, rhs).into();
                Enf::new(sub_node).into()
            }
            _ => unreachable!(),
        })
    }

    fn insert_bounded_symbol_access(&mut self, bsa: &ast::BoundedSymbolAccess) -> Link<NodeType> {
        let access_node = self.insert_symbol_access(&bsa.column);
        Boundary::new(access_node, bsa.boundary).into()
    }

    // Assumed inlining was done, to update
    fn insert_symbol_access(&mut self, access: &ast::SymbolAccess) -> Link<NodeType> {
        use air_parser::ast::ResolvableIdentifier;
        match access.name {
            // At this point during compilation, fully-qualified identifiers can only possibly refer
            // to a periodic column, as all functions have been inlined, and constants propagated.
            ResolvableIdentifier::Resolved(ref qid) => {
                if let Some(pc) = self.mir.periodic_columns.get(qid).cloned() {
                    SpannedMirValue {
                        span: qid.span(),
                        value: MirValue::PeriodicColumn(PeriodicColumnAccess::new(
                            *qid,
                            pc.period(),
                        )),
                    }.into()
                } else {
                    // This is a qualified reference that should have been eliminated
                    // during inlining or constant propagation, but somehow slipped through.
                    unreachable!(
                        "expected reference to periodic column, got `{:?}` instead",
                        qid
                    );
                }
            }
            // This must be one of public inputs, random values, or trace columns
            ResolvableIdentifier::Global(id) | ResolvableIdentifier::Local(id) => {
                // Special identifiers are those which are `$`-prefixed, and must refer to
                // the random values array (generally the case), or the names of trace segments (e.g. `$main`)
                if id.is_special() {
                    if let Some(rv) = self.random_value_access(access) {
                        return SpannedMirValue {
                            span: id.span(),
                            value: MirValue::RandomValue(rv),
                        }.into();
                    }

                    if let Some(tab) = self.trace_access_binding(access) {
                        return SpannedMirValue {
                            span: id.span(),
                            value: MirValue::TraceAccessBinding(tab),
                        }.into();
                    }

                    // Must be a trace segment name
                    if let Some(ta) = self.trace_access(access) {
                        return SpannedMirValue {
                            span: id.span(),
                            value: MirValue::TraceAccess(ta),
                        }.into();
                    }

                    // It should never be possible to reach this point - semantic analysis
                    // would have caught that this identifier is undefined.
                    unreachable!(
                        "expected reference to random values array or trace segment: {:#?}",
                        access
                    );
                }

                // Otherwise, we check the trace bindings, random value bindings, and public inputs, in that order
                if let Some(tab) = self.trace_access_binding(access) {
                    return SpannedMirValue {
                        span: id.span(),
                        value: MirValue::TraceAccessBinding(tab),
                    }.into();
                }

                if let Some(trace_access) = self.trace_access(access) {
                    return SpannedMirValue {
                        span: id.span(),
                        value: MirValue::TraceAccess(trace_access),
                    }.into();
                }

                if let Some(random_value) = self.random_value_access(access) {
                    return SpannedMirValue {
                        span: id.span(),
                        value: MirValue::RandomValue(random_value),
                    }.into();
                }

                if let Some(public_input) = self.public_input_access(access) {
                    return SpannedMirValue {
                        span: id.span(),
                        value: MirValue::PublicInput(public_input),
                    }.into();
                }

                // If we reach here, this must be a let-bound variable
                return self
                    .bindings
                    .get(access.name.as_ref())
                    .expect("undefined variable")
                    .clone();
            }
            // These should have been eliminated by previous compiler passes
            ResolvableIdentifier::Unresolved(_) => {
                unreachable!(
                    "expected fully-qualified or global reference, got `{:?}` instead",
                    &access.name
                );
            }
        }
    }

    // Check assumptions, probably this assumed that the inlining pass did some work
    fn random_value_access(&self, access: &ast::SymbolAccess) -> Option<usize> {
        let rv = self.random_values.as_ref()?;
        let id = access.name.as_ref();
        if rv.name == id {
            if let AccessType::Index(index) = access.access_type {
                assert!(index < rv.size);
                return Some(index);
            } else {
                // This should have been caught earlier during compilation
                unreachable!("invalid access to random values array: {:#?}", access);
            }
        }

        // This must be a reference to a binding, if it is a random value access
        let binding = rv.bindings.iter().find(|rb| rb.name == id)?;

        match access.access_type {
            AccessType::Default if binding.size == 1 => Some(binding.offset),
            AccessType::Index(extra) if binding.size > 1 => Some(binding.offset + extra),
            // This should have been caught earlier during compilation
            _ => unreachable!(
                "unexpected random value access type encountered during lowering: {:#?}",
                access
            ),
        }
    }

    // Check assumptions, probably this assumed that the inlining pass did some work
    fn public_input_access(&self, access: &ast::SymbolAccess) -> Option<PublicInputAccess> {
        let public_input = self.mir.public_inputs.get(access.name.as_ref())?;
        if let AccessType::Index(index) = access.access_type {
            Some(PublicInputAccess::new(public_input.name, index))
        } else {
            // This should have been caught earlier during compilation
            unreachable!(
                "unexpected public input access type encountered during lowering: {:#?}",
                access
            )
        }
    }

    // Check assumptions, probably this assumed that the inlining pass did some work
    fn trace_access_binding(&self, access: &ast::SymbolAccess) -> Option<TraceAccessBinding> {
        let id = access.name.as_ref();
        for segment in self.trace_columns.iter() {
            if let Some(binding) = segment
                .bindings
                .iter()
                .find(|tb| tb.name.as_ref() == Some(id))
            {
                return match &access.access_type {
                    AccessType::Default => Some(TraceAccessBinding {
                        segment: binding.segment,
                        offset: binding.offset,
                        size: binding.size,
                    }),
                    AccessType::Slice(range_expr) => Some(TraceAccessBinding {
                        segment: binding.segment,
                        offset: binding.offset,
                        size: range_expr.to_slice_range().count(),
                    }),
                    _ => None,
                };
            }
        }

        None
    }

    // Check assumptions, probably this assumed that the inlining pass did some work
    fn trace_access(&self, access: &ast::SymbolAccess) -> Option<TraceAccess> {
        let id = access.name.as_ref();
        for (i, segment) in self.trace_columns.iter().enumerate() {
            if segment.name == id {
                if let AccessType::Index(column) = access.access_type {
                    return Some(TraceAccess::new(i, column, access.offset));
                } else {
                    // This should have been caught earlier during compilation
                    unreachable!(
                        "unexpected trace access type encountered during lowering: {:#?}",
                        &access
                    );
                }
            }

            if let Some(binding) = segment
                .bindings
                .iter()
                .find(|tb| tb.name.as_ref() == Some(id))
            {
                return match access.access_type {
                    AccessType::Default if binding.size == 1 => Some(TraceAccess::new(
                        binding.segment,
                        binding.offset,
                        access.offset,
                    )),
                    AccessType::Index(extra_offset) if binding.size > 1 => Some(TraceAccess::new(
                        binding.segment,
                        binding.offset + extra_offset,
                        access.offset,
                    )),
                    // This should have been caught earlier during compilation
                    _ => unreachable!(
                        "unexpected trace access type encountered during lowering: {:#?}",
                        access
                    ),
                };
            }
        }

        None
    }

    /*/// Adds the specified operation to the graph and returns the index of its node.
    #[inline]
    fn insert_op(&mut self, op: Operation) -> Link<NodeType> {
        self.mir.constraint_graph_mut().insert_node(op)
    }*/

    fn constraint_graph_mut(&mut self) -> &mut Graph {
        self.mir.constraint_graph_mut()
    }

    fn insert_typed_constant(
        &mut self,
        span: Option<SourceSpan>,
        value: ast::ConstantExpr,
    ) -> Link<NodeType> {
        let mir_value = match value {
            ast::ConstantExpr::Scalar(val) => ConstantValue::Felt(val),
            ast::ConstantExpr::Vector(val) => ConstantValue::Vector(val),
            ast::ConstantExpr::Matrix(val) => ConstantValue::Matrix(val),
        };
        SpannedMirValue {
            span: span.unwrap_or_default(),
            value: MirValue::Constant(mir_value),
        }.into()
    }
}

