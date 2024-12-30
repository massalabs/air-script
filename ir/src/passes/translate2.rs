use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use air_parser::ast::AccessType;
use air_parser::{ast, symbols, LexicalScope, SemanticAnalysisError};
use air_pass::Pass;
use miden_diagnostics::{DiagnosticsHandler, Span, Spanned};

use crate::ir::{Add, Boundary, Enf, Evaluator, Matrix, Mul, Root, Sub};
use crate::{
    ir::{
        Builder, Call, ConstantValue, Fold, FoldOperator, For, Function, Link, Mir, MirType,
        MirValue, Op, Parameter, PublicInputAccess, SpannedMirValue, TraceAccess,
        TraceAccessBinding, Value, Vector,
    },
    passes::duplicate_node,
    CompileError,
};

pub struct AstToMir<'a> {
    diagnostics: &'a DiagnosticsHandler,
}

impl<'a> AstToMir<'a> {
    #[inline]
    pub fn new(diagnostics: &'a DiagnosticsHandler) -> Self {
        Self { diagnostics }
    }
}

impl Pass for AstToMir<'_> {
    type Input<'a> = ast::Program;
    type Output<'a> = Mir;
    type Error = CompileError;

    fn run<'a>(&mut self, program: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        let mut builder = MirBuilder::new(&program);
        builder.translate_program()?;
        Ok(builder.mir)
    }
}

pub struct MirBuilder<'a> {
    program: &'a ast::Program,
    mir: Mir,
    random_values: Option<&'a ast::RandomValues>,
    trace_columns: &'a Vec<ast::TraceSegment>,
    bindings: LexicalScope<&'a ast::Identifier, Link<Op>>,
    root: Link<Root>,
    root_name: Option<&'a ast::QualifiedIdentifier>,
    in_boundary: bool,
}

impl<'a> MirBuilder<'a> {
    pub fn new(program: &'a ast::Program) -> Self {
        Self {
            program,
            mir: Mir::default(),
            random_values: program.random_values.as_ref(),
            trace_columns: program.trace_columns.as_ref(),
            bindings: LexicalScope::default(),
            root: Link::default(),
            root_name: None,
            in_boundary: true,
        }
    }

    pub fn translate_program(&mut self) -> Result<(), CompileError> {
        self.mir = Mir::default();
        let random_values = &self.program.random_values;
        let trace_columns = &self.program.trace_columns;
        let boundary_constraints = &self.program.boundary_constraints;
        let integrity_constraints = &self.program.integrity_constraints;

        self.mir.trace_columns.clone_from(trace_columns);
        self.mir.num_random_values = random_values.as_ref().map(|rv| rv.size as u16).unwrap_or(0);
        self.mir.periodic_columns = self.program.periodic_columns.clone();
        self.mir.public_inputs = self.program.public_inputs.clone();

        for (ident, function) in &self.program.functions {
            self.translate_function_signature(ident, function)?;
        }
        for (ident, evaluator) in &self.program.evaluators {
            self.translate_evaluator_signature(ident, evaluator)?;
        }
        for (ident, function) in &self.program.functions {
            self.translate_function(ident, function)?;
        }
        for (ident, evaluator) in &self.program.evaluators {
            self.translate_evaluator(ident, evaluator)?;
        }
        self.root = Link::default();
        self.in_boundary = true;
        for boundary_constraint in boundary_constraints {
            self.translate_statement(boundary_constraint)?;
        }
        self.in_boundary = false;
        for integrity_constraint in integrity_constraints {
            self.translate_statement(integrity_constraint)?;
        }
        Ok(())
    }

    fn translate_evaluator_signature(
        &mut self,
        ident: &'a ast::QualifiedIdentifier,
        ast_eval: &'a ast::EvaluatorFunction,
    ) -> Result<Link<Evaluator>, CompileError> {
        self._translate_evaluator_known(ident, ast_eval, false)
    }

    fn translate_evaluator(
        &mut self,
        ident: &'a ast::QualifiedIdentifier,
        ast_eval: &'a ast::EvaluatorFunction,
    ) -> Result<Link<Evaluator>, CompileError> {
        self._translate_evaluator_known(ident, ast_eval, true)
    }

    fn _translate_evaluator_known(
        &mut self,
        ident: &'a ast::QualifiedIdentifier,
        ast_eval: &'a ast::EvaluatorFunction,
        known_signature: bool,
    ) -> Result<Link<Evaluator>, CompileError> {
        self.bindings.enter();
        self.root_name = Some(ident);
        let mut ev = Evaluator::builder();
        let mut i = 0;
        for trace_segment in &ast_eval.params {
            println!("trace_segment: {:#?}", trace_segment);
            for binding in &trace_segment.bindings {
                println!("binding: {:#?}", binding);
                let params =
                    self.translate_params(ident, binding.name.as_ref(), &binding.ty, &mut i);
                for param in params {
                    ev = ev.parameters(param.clone());
                }
            }
        }
        let ev = Link::new(ev.build());
        if known_signature {
            self.translate_body(ident, ev.clone().as_root(), &ast_eval.body)?;
            let original = self.mir
                .constraint_graph_mut()
                .get_evaluator_mut(ident)
                .unwrap_or_else(||panic!("missing evaluator signature for {:?}\nuse self.translate_evaluator_signature(ident, ast_eval) before self.translate_evaluator(ident, ast_eval)", ident));
            if original.borrow().parameters != ev.borrow().parameters {
                panic!(
                    "evaluator parameter mismatch for {:?}\nexpected: {:#?}\nbut got: {:#?}",
                    ident,
                    original.borrow().parameters,
                    ev.borrow().parameters
                );
            }
            let mut updated = original.clone().borrow().clone().edit();
            for op in ev.clone().borrow().body.borrow().iter() {
                updated = updated.body(op.clone());
            }
            *original = updated.build().into();
        } else {
            self.mir
                .constraint_graph_mut()
                .insert_evaluator(*ident, ev.clone());
        }
        self.bindings.exit();
        Ok(ev)
    }

    fn translate_function_signature(
        &mut self,
        ident: &'a ast::QualifiedIdentifier,
        ast_func: &'a ast::Function,
    ) -> Result<Link<Function>, CompileError> {
        self._translate_function_known(ident, ast_func, false)
    }

    fn translate_function(
        &mut self,
        ident: &'a ast::QualifiedIdentifier,
        ast_func: &'a ast::Function,
    ) -> Result<Link<Function>, CompileError> {
        self._translate_function_known(ident, ast_func, true)
    }

    fn _translate_function_known(
        &mut self,
        ident: &'a ast::QualifiedIdentifier,
        ast_func: &'a ast::Function,
        known_signature: bool,
    ) -> Result<Link<Function>, CompileError> {
        self.bindings.enter();
        self.root_name = Some(ident);
        let mut func = Function::builder();
        let mut i = 0;
        for (param_ident, ty) in ast_func.params.iter() {
            let name = Some(param_ident);
            let params = self.translate_params(ident, name, ty, &mut i);
            for param in params {
                func = func.parameters(param.clone());
            }
        }
        i += 1;
        let ret = Parameter::new(i, self.translate_type(&ast_func.return_type));
        let func = Link::new(func.return_type(ret.into()).build());
        if known_signature {
            self.translate_body(ident, func.clone().as_root(), &ast_func.body)?;
            let original = self.mir
                .constraint_graph_mut()
                .get_function_mut(ident).unwrap_or_else(||panic!("missing function signature for {:?}\nuse self.translate_function_signature(ident, ast_func) before self.translate_function(ident, ast_func)", ident));
            let orig_sig = original.clone().borrow().clone();
            let new_sig = func.borrow();
            if orig_sig.parameters != new_sig.parameters {
                panic!(
                    "function parameter mismatch for {:?}\nexpected: {:#?}\nbut got: {:#?}",
                    ident,
                    original.borrow().deref().parameters,
                    func.borrow().deref().parameters
                );
            } else if orig_sig.return_type != new_sig.return_type {
                panic!(
                    "function return mismatch for {:?}\nexpected: {:#?}\nbut got: {:#?}",
                    ident,
                    original.borrow().deref().return_type,
                    func.borrow().deref().return_type
                );
            }
            let mut updated = original.clone().borrow().clone().edit();
            for op in func.clone().borrow().body.borrow().iter() {
                updated = updated.body(op.clone());
            }
            *original = updated.build().into();
        } else {
            self.mir
                .constraint_graph_mut()
                .insert_function(*ident, func.clone());
        }
        self.bindings.exit();
        Ok(func)
    }

    fn translate_params(
        &mut self,
        func_ident: &'a ast::QualifiedIdentifier,
        name: Option<&'a ast::Identifier>,
        ty: &ast::Type,
        i: &mut usize,
    ) -> Vec<Link<Parameter>> {
        match ty {
            ast::Type::Felt => {
                let param: Link<Parameter> = Parameter::new(*i, MirType::Felt).into();
                *i += 1;
                self.bindings.insert(name.unwrap(), param.clone().as_op());
                vec![param]
            }
            ast::Type::Vector(size) => {
                let mut vector = Vector::builder().size(*size);
                let mut params = Vec::new();
                for _ in 0..*size {
                    let param: Link<Parameter> = Parameter::new(*i, MirType::Felt).into();
                    *i += 1;
                    vector = vector.elements(param.clone().as_op());
                    params.push(param);
                }
                let vector: Link<Op> = vector.build().as_op().into();
                self.bindings.insert(name.unwrap(), vector.clone());
                params
            }
            ast::Type::Matrix(_rows, _cols) => {
                unimplemented!("matrix parameters not supported");
            }
        }
    }

    fn translate_body(
        &mut self,
        ident: &ast::QualifiedIdentifier,
        func: Link<Root>,
        body: &'a Vec<ast::Statement>,
    ) -> Result<Link<Root>, CompileError> {
        self.root = func.clone();
        self.bindings.enter();
        let mut func = func;
        for stmt in body {
            let op = self.translate_statement(stmt)?;
            func = match func.clone().borrow().deref() {
                Root::Function(_) => func
                    .clone()
                    .as_function()
                    .unwrap()
                    .borrow_mut()
                    .clone()
                    .edit()
                    .body(op)
                    .build()
                    .as_root()
                    .into(),
                Root::Evaluator(_) => func
                    .clone()
                    .as_evaluator()
                    .unwrap()
                    .borrow_mut()
                    .clone()
                    .edit()
                    .body(op)
                    .build()
                    .as_root()
                    .into(),
                Root::None => {
                    unreachable!("expected function or evaluator, got None")
                }
            };
            self.root = func.clone();
        }
        self.bindings.exit();
        match func.borrow().deref() {
            Root::Function(_) => self
                .mir
                .constraint_graph_mut()
                .insert_function(*ident, func.clone().as_function().unwrap().clone()),
            Root::Evaluator(_) => self
                .mir
                .constraint_graph_mut()
                .insert_evaluator(*ident, func.clone().as_evaluator().unwrap().clone()),
            Root::None => {
                unreachable!("expected function or evaluator, got None")
            }
        }
        Ok(func)
    }

    fn translate_type(&mut self, ty: &ast::Type) -> MirType {
        match ty {
            ast::Type::Felt => MirType::Felt,
            ast::Type::Vector(size) => MirType::Vector(*size),
            ast::Type::Matrix(rows, cols) => MirType::Matrix(*rows, *cols),
        }
    }

    fn translate_statement(&mut self, stmt: &'a ast::Statement) -> Result<Link<Op>, CompileError> {
        match stmt {
            ast::Statement::Let(let_stmt) => self.translate_let(let_stmt),
            ast::Statement::Expr(expr) => self.translate_expr(expr),
            ast::Statement::Enforce(enf) => self.translate_enforce(enf),
            ast::Statement::EnforceIf(enf, cond) => self.translate_enforce_if(enf, cond),
            ast::Statement::EnforceAll(list_comp) => self.translate_enforce_all(list_comp),
        }
    }
    fn translate_let(&mut self, let_stmt: &'a ast::Let) -> Result<Link<Op>, CompileError> {
        let name = &let_stmt.name;
        let value: Link<Op> = self.translate_expr(&let_stmt.value)?;
        self.bindings.insert(name, value.clone());
        Ok(value)
    }
    fn translate_expr(&mut self, expr: &'a ast::Expr) -> Result<Link<Op>, CompileError> {
        match expr {
            ast::Expr::Const(c) => self.translate_spanned_const(c),
            ast::Expr::Range(r) => self.translate_range(r),
            ast::Expr::Vector(v) => self.translate_vector_expr(&v.item),
            ast::Expr::Matrix(m) => self.translate_matrix(m),
            ast::Expr::SymbolAccess(s) => self.translate_symbol_access(s),
            ast::Expr::Binary(b) => self.translate_binary_op(b),
            ast::Expr::Call(c) => self.translate_call(c),
            ast::Expr::ListComprehension(lc) => self.translate_list_comprehension(lc),
            ast::Expr::Let(l) => self.translate_let(l),
        }
    }

    fn translate_enforce(&mut self, enf: &'a ast::ScalarExpr) -> Result<Link<Op>, CompileError> {
        let node = self.translate_scalar_expr(enf)?;
        self.insert_enforce(node)
    }

    fn translate_enforce_if(
        &mut self,
        enf: &ast::ScalarExpr,
        cond: &ast::ScalarExpr,
    ) -> Result<Link<Op>, CompileError> {
        unreachable!("all EnforceIf should have been transformed into EnforceAll")
    }

    fn translate_enforce_all(
        &mut self,
        list_comp: &'a ast::ListComprehension,
    ) -> Result<Link<Op>, CompileError> {
        self.bindings.enter();
        for (index, binding) in list_comp.bindings.iter().enumerate() {
            let binding_node =
                Parameter::new(/*binding.span(), */ index, ast::Type::Felt.into()).as_op();
            self.bindings.insert(binding, binding_node.into());
        }

        let mut iterator_nodes: Vec<Link<Op>> = Vec::new();
        for iterator in list_comp.iterables.iter() {
            let iterator_node = self.translate_expr(iterator)?;
            iterator_nodes.push(iterator_node);
        }

        let selector_node = if let Some(selector) = &list_comp.selector {
            self.translate_scalar_expr(selector)?
        } else {
            Link::default()
        };
        let body_node = self.translate_scalar_expr(&list_comp.body)?;

        let for_node = For::new(iterator_nodes.into(), body_node, selector_node).as_op();

        let enf_node: Link<Op> = Enf::new(for_node.into()).as_op().into();
        let node = self.insert_enforce(enf_node);
        self.bindings.exit();
        node
    }

    fn insert_enforce(&mut self, node: Link<Op>) -> Result<Link<Op>, CompileError> {
        let node_to_add = if let Op::Enf(_) = node.clone().borrow().deref() {
            node
        } else {
            Enf::builder().expr(node).build().as_op().into()
        };
        match self.in_boundary {
            true => self
                .mir
                .constraint_graph_mut()
                .insert_boundary_constraints_root(node_to_add.clone()),
            false => {
                match self.root.borrow_mut().deref_mut() {
                    Root::Function(ref mut func) => {
                        *func = func.clone().edit().body(node_to_add.clone()).build();
                    }
                    Root::Evaluator(ref mut evaluator) => {
                        *evaluator = evaluator.clone().edit().body(node_to_add.clone()).build();
                    }
                    Root::None => {
                        // Insert in integrity
                        self.mir
                            .constraint_graph_mut()
                            .insert_integrity_constraints_root(node_to_add.clone());
                    }
                };
                /*if parent == Link::new(Owner::default()) {
                    self.mir
                        .constraint_graph_mut()
                        .insert_integrity_constraints_root(node_to_add);
                }*/
            }
        };
        Ok(node_to_add)
    }

    fn translate_spanned_const(
        &mut self,
        c: &Span<ast::ConstantExpr>,
    ) -> Result<Link<Op>, CompileError> {
        self.translate_const(&c.item)
    }

    fn translate_range(&mut self, range_expr: &ast::RangeExpr) -> Result<Link<Op>, CompileError> {
        let values = range_expr.to_slice_range();
        let const_expr = ast::ConstantExpr::Vector(values.map(|v| v as u64).collect());
        self.translate_const(&const_expr)
    }

    fn translate_vector_expr(&mut self, v: &'a Vec<ast::Expr>) -> Result<Link<Op>, CompileError> {
        let mut node = Vector::builder().size(v.len());
        for value in v.iter() {
            let value_node = self.translate_expr(value)?;
            node = node.elements(value_node);
        }
        Ok(node.build().as_op().into())
    }

    fn translate_vector_scalar_expr(
        &mut self,
        v: &'a [ast::ScalarExpr],
    ) -> Result<Link<Op>, CompileError> {
        let mut node = Vector::builder().size(v.len());
        for value in v.iter() {
            let value_node = self.translate_scalar_expr(value)?;
            node = node.elements(value_node);
        }
        Ok(node.build().as_op().into())
    }

    fn translate_matrix(
        &mut self,
        m: &'a Span<Vec<Vec<ast::ScalarExpr>>>,
    ) -> Result<Link<Op>, CompileError> {
        let mut node = Matrix::builder().size(m.len());
        for row in m.iter() {
            let row_node = self.translate_vector_scalar_expr(row)?.as_vector().unwrap();
            node = node.elements(row_node);
        }
        let node = node.build().as_op().into();
        Ok(node)
    }

    fn translate_symbol_access(
        &mut self,
        access: &ast::SymbolAccess,
    ) -> Result<Link<Op>, CompileError> {
        match access.name {
            // At this point during compilation, fully-qualified identifiers can only possibly refer
            // to a periodic column, as all functions have been inlined, and constants propagated.
            ast::ResolvableIdentifier::Resolved(qual_ident) => {
                if let Some(pc) = self.mir.periodic_columns.get(&qual_ident).cloned() {
                    let node = Value::builder()
                        .value(SpannedMirValue {
                            span: Default::default(),
                            value: MirValue::PeriodicColumn(crate::ir::PeriodicColumnAccess::new(
                                qual_ident,
                                pc.period(),
                            )),
                        })
                        .build()
                        .as_op()
                        .into();
                    Ok(node)
                } else {
                    // This is a qualified reference that should have been eliminated
                    // during inlining or constant propagation, but somehow slipped through.
                    unreachable!(
                        "expected reference to periodic column, got `{:?}` instead",
                        qual_ident
                    );
                }
            }
            // This must be one of public inputs, random values, or trace columns
            ast::ResolvableIdentifier::Global(ident) | ast::ResolvableIdentifier::Local(ident) => {
                self.translate_symbol_access_global_or_local(&ident, &access)
            }
            // These should have been eliminated by previous compiler passes
            ast::ResolvableIdentifier::Unresolved(ident) => {
                unreachable!(
                    "expected fully-qualified or global reference, got `{:?}` instead",
                    &access.name
                );
            }
        }
    }

    fn translate_binary_op(
        &mut self,
        bin_op: &'a ast::BinaryExpr,
    ) -> Result<Link<Op>, CompileError> {
        let lhs = self.translate_scalar_expr(&bin_op.lhs)?;
        let rhs = self.translate_scalar_expr(&bin_op.rhs)?;
        match bin_op.op {
            ast::BinaryOp::Add => {
                let node = Add::builder().lhs(lhs).rhs(rhs).build().as_op().into();
                Ok(node)
            }
            ast::BinaryOp::Sub => {
                let node = Sub::builder().lhs(lhs).rhs(rhs).build().as_op().into();
                Ok(node)
            }
            ast::BinaryOp::Mul => {
                let node = Mul::builder().lhs(lhs).rhs(rhs).build().as_op().into();
                Ok(node)
            }
            ast::BinaryOp::Exp => {
                let ast::ScalarExpr::Const(rhs) = bin_op.rhs.as_ref() else {
                    return Err(CompileError::SemanticAnalysis(
                        SemanticAnalysisError::InvalidExpr(
                            ast::InvalidExprError::NonConstantExponent(bin_op.rhs.span()),
                        ),
                    ));
                };
                self.expand_exp(lhs, rhs.item)
            }
            ast::BinaryOp::Eq => {
                let sub_node = Sub::builder().lhs(lhs).rhs(rhs).build().as_op().into();
                Ok(Enf::builder().expr(sub_node).build().as_op().into())
            }
        }
    }

    fn translate_call(&mut self, call: &'a ast::Call) -> Result<Link<Op>, CompileError> {
        // First, resolve the callee, panic if it's not resolved
        let resolved_callee = call.callee.resolved().unwrap();

        if call.is_builtin() {
            // If it's a fold operator (Sum / Prod), handle it
            match call.callee.as_ref().name() {
                symbols::Sum => {
                    assert_eq!(call.args.len(), 1);
                    let iterator_node = self.translate_expr(call.args.first().unwrap())?;
                    let accumulator_node = self.translate_const(&ast::ConstantExpr::Scalar(0))?;
                    let node = Fold::builder()
                        .iterator(iterator_node)
                        .operator(FoldOperator::Add)
                        .initial_value(accumulator_node)
                        .build()
                        .as_op()
                        .into();
                    Ok(node)
                }
                symbols::Prod => {
                    assert_eq!(call.args.len(), 1);
                    let iterator_node = self.translate_expr(call.args.first().unwrap())?;
                    let accumulator_node = self.translate_const(&ast::ConstantExpr::Scalar(1))?;
                    let node = Fold::builder()
                        .iterator(iterator_node)
                        .operator(FoldOperator::Mul)
                        .initial_value(accumulator_node)
                        .build()
                        .as_op()
                        .into();
                    Ok(node)
                }
                other => unimplemented!("unhandled builtin: {}", other),
            }
        } else {
            let mut arg_nodes: Vec<Link<Op>>;

            // Get the known callee in the functions hashmap
            // Then, get the node index of the function definition
            let callee_node;
            if let Some(callee) = self.mir.constraint_graph().get_function(&resolved_callee) {
                callee_node = callee.clone().as_root();
                arg_nodes = call
                    .args
                    .iter()
                    .map(|arg| self.translate_expr(arg).unwrap())
                    .collect()
            } else if let Some(callee) = self.mir.constraint_graph().get_evaluator(&resolved_callee)
            {
                callee_node = callee.clone().as_root();
                arg_nodes = Vec::new();
                for arg in call.args.iter() {
                    let arg_node = self.translate_expr(arg)?;
                    arg_nodes.push(arg_node);
                }
            } else {
                panic!("Unkown function or evaluator: {:?}", resolved_callee);
            }
            let mut call_node = Call::builder().function(callee_node);
            for arg in arg_nodes {
                call_node = call_node.argument(arg);
            }
            let call_node = call_node.build().as_op().into();
            Ok(call_node)
        }
    }

    fn translate_list_comprehension(
        &mut self,
        list_comp: &'a ast::ListComprehension,
    ) -> Result<Link<Op>, CompileError> {
        self.bindings.enter();
        for (index, binding) in list_comp.bindings.iter().enumerate() {
            let binding_node =
                Parameter::new(/*binding.span(), */ index, ast::Type::Felt.into()).as_op();
            self.bindings.insert(binding, binding_node.into());
        }

        let iterator_nodes = Link::new(Vec::new());
        for iterator in list_comp.iterables.iter() {
            let iterator_node = self.translate_expr(iterator)?;
            iterator_nodes.borrow_mut().push(iterator_node);
        }

        let selector_node = if let Some(selector) = &list_comp.selector {
            self.translate_scalar_expr(selector)?
        } else {
            Link::default()
        };
        let body_node = self.translate_scalar_expr(&list_comp.body)?;

        let for_node = For::new(iterator_nodes, body_node, selector_node)
            .as_op()
            .into();

        self.bindings.exit();
        Ok(for_node)
    }

    fn translate_scalar_expr(
        &mut self,
        scalar_expr: &'a ast::ScalarExpr,
    ) -> Result<Link<Op>, CompileError> {
        match scalar_expr {
            ast::ScalarExpr::Const(c) => self.translate_scalar_const(c.item),
            ast::ScalarExpr::SymbolAccess(s) => self.translate_symbol_access(s),
            ast::ScalarExpr::BoundedSymbolAccess(s) => self.translate_bounded_symbol_access(s),
            ast::ScalarExpr::Binary(b) => self.translate_binary_op(b),
            ast::ScalarExpr::Call(c) => self.translate_call(c),
            ast::ScalarExpr::Let(l) => self.translate_let(l),
        }
    }

    fn translate_scalar_const(&mut self, c: u64) -> Result<Link<Op>, CompileError> {
        let value = SpannedMirValue {
            value: MirValue::Constant(ConstantValue::Felt(c)),
            span: Default::default(),
        };
        let node = Value::builder().value(value).build().as_op().into();
        Ok(node)
    }

    fn translate_bounded_symbol_access(
        &mut self,
        access: &ast::BoundedSymbolAccess,
    ) -> Result<Link<Op>, CompileError> {
        let access_node = self.translate_symbol_access(&access.column)?;
        let node = Boundary::builder()
            .kind(access.boundary)
            .expr(access_node)
            .build()
            .as_op()
            .into();
        Ok(node)
    }

    fn translate_const(&mut self, c: &ast::ConstantExpr) -> Result<Link<Op>, CompileError> {
        match c {
            ast::ConstantExpr::Scalar(s) => self.translate_scalar_const(*s),
            ast::ConstantExpr::Vector(v) => self.translate_vector_const(v.clone()),
            ast::ConstantExpr::Matrix(m) => self.translate_matrix_const(m.clone()),
        }
    }

    fn translate_vector_const(&mut self, v: Vec<u64>) -> Result<Link<Op>, CompileError> {
        let mut node = Vector::builder().size(v.len());
        for value in v.iter() {
            let value_node = self.translate_scalar_const(*value)?;
            node = node.elements(value_node);
        }
        Ok(node.build().as_op().into())
    }

    fn translate_matrix_const(&mut self, m: Vec<Vec<u64>>) -> Result<Link<Op>, CompileError> {
        let mut node = Matrix::builder().size(m.len());
        for row in m.iter() {
            let row_node = self
                .translate_vector_const(row.clone())?
                .as_vector()
                .unwrap();
            node = node.elements(row_node);
        }
        let node = node.build().as_op().into();
        Ok(node)
    }

    fn translate_symbol_access_global_or_local(
        &mut self,
        ident: &ast::Identifier,
        access: &ast::SymbolAccess,
    ) -> Result<Link<Op>, CompileError> {
        // Special identifiers are those which are `$`-prefixed, and must refer to
        // the random values array (generally the case), or the names of trace segments (e.g. `$main`)
        if ident.is_special() {
            if let Some(rv) = self.random_value_access(access) {
                return Ok(Value::builder()
                    .value(SpannedMirValue {
                        span: Default::default(),
                        value: MirValue::RandomValue(rv),
                    })
                    .build()
                    .as_op()
                    .into());
            }

            if let Some(tab) = self.trace_access_binding(access) {
                return Ok(Value::builder()
                    .value(SpannedMirValue {
                        span: Default::default(),
                        value: MirValue::TraceAccessBinding(tab),
                    })
                    .build()
                    .as_op()
                    .into());
            }

            // Must be a trace segment name
            if let Some(ta) = self.trace_access(access) {
                return Ok(Value::builder()
                    .value(SpannedMirValue {
                        span: Default::default(),
                        value: MirValue::TraceAccess(ta),
                    })
                    .build()
                    .as_op()
                    .into());
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
            return Ok(Value::builder()
                .value(SpannedMirValue {
                    span: Default::default(),
                    value: MirValue::TraceAccessBinding(tab),
                })
                .build()
                .as_op()
                .into());
        }

        if let Some(trace_access) = self.trace_access(access) {
            return Ok(Value::builder()
                .value(SpannedMirValue {
                    span: Default::default(),
                    value: MirValue::TraceAccess(trace_access),
                })
                .build()
                .as_op()
                .into());
        }

        if let Some(random_value) = self.random_value_access(access) {
            return Ok(Value::builder()
                .value(SpannedMirValue {
                    span: Default::default(),
                    value: MirValue::RandomValue(random_value),
                })
                .build()
                .as_op()
                .into());
        }

        if let Some(public_input) = self.public_input_access(access) {
            return Ok(Value::builder()
                .value(SpannedMirValue {
                    span: Default::default(),
                    value: MirValue::PublicInput(public_input),
                })
                .build()
                .as_op()
                .into());
        }

        let node = self
            .bindings
            .get(access.name.as_ref())
            .unwrap_or_else(|| panic!("undefined variable: {:?}", access))
            .clone();
        Ok(node)
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

    // Use square and multiply algorithm to expand the exp into a series of multiplications
    fn expand_exp(&mut self, lhs: Link<Op>, rhs: u64) -> Result<Link<Op>, CompileError> {
        // 0 -> 1
        // 1 -> lhs
        // n (n pair) ->
        match rhs {
            0 => self.translate_const(&ast::ConstantExpr::Scalar(1)),
            1 => Ok(duplicate_node(lhs.clone())),
            n if n % 2 == 0 => {
                let new_lhs = duplicate_node(lhs.clone());
                let new_rhs = duplicate_node(lhs.clone());
                let square = Mul::new(new_lhs, new_rhs).as_op().into();
                self.expand_exp(square, n / 2)
            }
            n => {
                let new_lhs = duplicate_node(lhs.clone());
                let new_rhs = duplicate_node(lhs.clone());
                let new_lhs_clone = duplicate_node(lhs.clone());
                let square = Mul::new(new_lhs, new_rhs).as_op().into();
                let rec: Link<Op> = self.expand_exp(square, (n - 1) / 2)?;
                let node = Mul::builder()
                    .lhs(new_lhs_clone)
                    .rhs(rec)
                    .build()
                    .as_op()
                    .into();
                Ok(node)
            }
        }
    }
}
