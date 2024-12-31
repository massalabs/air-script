/*mod constant_propagation;
mod value_numbering;

pub use self::constant_propagation::ConstantPropagation;
pub use self::value_numbering::ValueNumbering;

mod inlining_old;
mod translate_old;
mod unrolling_old;
mod visitor_old;
pub use self::inlining_old::InliningOld;
pub use self::translate_old::AstToMirOld;
pub use self::unrolling_old::UnrollingOld;
pub use self::visitor_old::{Graph, VisitContextOld, VisitOld, VisitOrderOld};*/

mod inlining;
//mod inlining2;
mod translate;
mod unrolling;
mod visitor;
mod visitor2;
// pub use self::inlining::Inlining;
pub use self::inlining::Inlining;
//pub use self::translate::AstToMir;
pub use self::translate::AstToMir;
pub use self::unrolling::Unrolling;
pub use self::visitor::{Visit, VisitContext, VisitOrder};

use std::collections::HashMap;
use std::ops::Deref;

use air_pass::Pass;

use crate::ir::{
    Accessor, Add, Boundary, Call, Enf, Fold, For, If, Link, Matrix, Mul, Op, Parent, Sub, Value,
    Vector,
};

pub struct DumpAst;
impl Pass for DumpAst {
    type Input<'a> = air_parser::ast::Program;
    type Output<'a> = air_parser::ast::Program;
    type Error = air_parser::SemanticAnalysisError;

    fn run<'a>(&mut self, input: Self::Input<'a>) -> Result<Self::Output<'a>, Self::Error> {
        println!("{}", &input);
        Ok(input)
    }
}

pub fn duplicate_node(node: Link<Op>) -> Link<Op> {
    match node.borrow().deref() {
        Op::Enf(enf) => {
            let expr = enf.borrow().deref().expr.clone();
            let new_expr = duplicate_node(expr);
            return Enf::create(new_expr).as_op().into();
        }
        Op::Boundary(boundary) => {
            let expr = boundary.borrow().deref().expr.clone();
            let kind = boundary.borrow().deref().kind;
            let new_expr = duplicate_node(expr);
            return Boundary::create(new_expr, kind).as_op().into();
        }
        Op::Add(add) => {
            let lhs = add.borrow().deref().lhs.clone();
            let rhs = add.borrow().deref().rhs.clone();
            let new_lhs_node = duplicate_node(lhs);
            let new_rhs_node = duplicate_node(rhs);
            return Add::create(new_lhs_node, new_rhs_node).as_op().into();
        }
        Op::Sub(sub) => {
            let lhs = sub.borrow().deref().lhs.clone();
            let rhs = sub.borrow().deref().rhs.clone();
            let new_lhs_node = duplicate_node(lhs);
            let new_rhs_node = duplicate_node(rhs);
            return Sub::create(new_lhs_node, new_rhs_node).as_op().into();
        }
        Op::Mul(mul) => {
            let lhs = mul.borrow().deref().lhs.clone();
            let rhs = mul.borrow().deref().rhs.clone();
            let new_lhs_node = duplicate_node(lhs);
            let new_rhs_node = duplicate_node(rhs);
            return Mul::create(new_lhs_node, new_rhs_node).as_op().into();
        }
        Op::If(if_node) => {
            let condition = if_node.borrow().deref().condition.clone();
            let then_branch = if_node.borrow().deref().then_branch.clone();
            let else_branch = if_node.borrow().deref().else_branch.clone();
            let new_condition = duplicate_node(condition);
            let new_then_branch = duplicate_node(then_branch);
            let new_else_branch = duplicate_node(else_branch);
            return If::create(new_condition, new_then_branch, new_else_branch)
                .as_op()
                .into();
        }
        Op::For(for_node) => {
            let iterators = for_node.borrow().deref().iterators.clone();
            let body = for_node.borrow().deref().expr.clone();
            let selector = for_node.borrow().deref().selector.clone();
            let new_iterators = iterators
                .borrow()
                .iter()
                .cloned()
                .map(|iterator| duplicate_node(iterator))
                .collect::<Vec<_>>()
                .into();
            let new_body = duplicate_node(body);
            let new_selector = duplicate_node(selector);
            return For::create(new_iterators, new_body, new_selector)
                .as_op()
                .into();
        }
        Op::Call(call) => {
            let arguments = call.borrow().deref().arguments.clone();
            let function = call.borrow().deref().function.clone();
            let new_arguments = arguments
                .borrow()
                .iter()
                .cloned()
                .map(|argument| duplicate_node(argument))
                .collect::<Vec<_>>()
                .into();
            return Call::create(function, new_arguments).as_op().into();
        }
        Op::Fold(fold) => {
            let iterator = fold.borrow().deref().iterator.clone();
            let operator = fold.borrow().deref().operator.clone();
            let initial_value = fold.borrow().deref().initial_value.clone();
            let new_iterator = duplicate_node(iterator);
            let new_initial_value = duplicate_node(initial_value);
            return Fold::create(new_iterator, operator, new_initial_value)
                .as_op()
                .into();
        }
        Op::Vector(vector) => {
            let children_link = vector.children().clone();
            let children_ref = children_link.borrow();
            let children = children_ref.deref();
            let new_children = children
                .iter()
                .cloned()
                .map(|child| duplicate_node(child))
                .collect();
            return Vector::create(new_children).as_op().into();
        }
        Op::Matrix(matrix) => {
            let mut new_matrix = Vec::new();
            let children_link = matrix.children().clone();
            let children_ref = children_link.borrow();
            let children = children_ref.deref();
            for row in children.iter() {
                let row_children_link = row
                    .clone()
                    .as_vector()
                    .expect(format!("expected Vector, found {:?}", row).as_str())
                    .children()
                    .clone();
                let row_children_ref = row_children_link.borrow();
                let row_children = row_children_ref.deref();
                let new_row_as_vec = row_children
                    .iter()
                    .cloned()
                    .map(|child| duplicate_node(child))
                    .collect::<Vec<_>>()
                    .into();
                let new_row = Vector::create(new_row_as_vec).into();
                new_matrix.push(new_row);
            }
            return Matrix::create(new_matrix).as_op().into();
        }
        Op::Accessor(accessor) => {
            let indexable = accessor.borrow().deref().indexable.clone();
            let access_type = accessor.borrow().deref().access_type.clone();
            let new_indexable = duplicate_node(indexable);
            return Accessor::create(new_indexable, access_type).as_op().into();
        }
        Op::Parameter(parameter) => {
            return parameter.clone().as_op().into();
        }
        Op::Value(value) => {
            return Value::create(value.borrow().deref().value.clone())
                .as_op()
                .into();
        }
        Op::None => Op::None.into(),
    }
}

// Helper used to duplicate nodes and their children recursively, used during Inlining and Unrolling
// Additionnally, if a Leaf is a Parameter, it is replaced with the corresponding item of the replace_parameter_list Vec.
// This is useful for inlining function calls (and replacing their parameters with the arguments of the call) for Inlining,
// and for unrolling loops (and replacing their parameters with the iterator values) for Unrolling.
// Inlining: replace_parameter_list = arguments should be the arguments from the Call()
// Unrolling: replace_parameter_list = self.for_inlining_context.unwrap().iterators
pub fn duplicate_node_or_replace(
    current_replace_map: &mut HashMap<Link<Op>, Link<Op>>,
    node: Link<Op>,
    replace_parameter_list: Vec<Link<Op>>,
) {
    match node.borrow().deref() {
        Op::Enf(enf) => {
            let expr = enf.borrow().expr.clone();
            let new_expr = current_replace_map.get(&expr).unwrap().clone();
            let new_node = Enf::create(new_expr).as_op().into();
            current_replace_map.insert(node.clone(), new_node);
        }
        Op::Boundary(boundary) => {
            let expr = boundary.borrow().expr.clone();
            let kind = boundary.borrow().kind;
            let new_expr = current_replace_map.get(&expr).unwrap().clone();
            let new_node = Boundary::create(new_expr, kind).as_op().into();
            current_replace_map.insert(node.clone(), new_node);
        }
        Op::Add(add) => {
            let lhs = add.borrow().lhs.clone();
            let rhs = add.borrow().rhs.clone();
            let new_lhs_node = current_replace_map.get(&lhs).unwrap().clone();
            let new_rhs_node = current_replace_map.get(&rhs).unwrap().clone();
            let new_node = Add::create(new_lhs_node, new_rhs_node).as_op().into();
            current_replace_map.insert(node.clone(), new_node);
        }
        Op::Sub(sub) => {
            let lhs = sub.borrow().lhs.clone();
            let rhs = sub.borrow().rhs.clone();
            let new_lhs_node = current_replace_map.get(&lhs).unwrap().clone();
            let new_rhs_node = current_replace_map.get(&rhs).unwrap().clone();
            let new_node = Sub::create(new_lhs_node, new_rhs_node).as_op().into();
            current_replace_map.insert(node.clone(), new_node);
        }
        Op::Mul(mul) => {
            let lhs = mul.borrow().lhs.clone();
            let rhs = mul.borrow().rhs.clone();
            let new_lhs_node = current_replace_map.get(&lhs).unwrap().clone();
            let new_rhs_node = current_replace_map.get(&rhs).unwrap().clone();
            let new_node = Mul::create(new_lhs_node, new_rhs_node).as_op().into();
            current_replace_map.insert(node.clone(), new_node);
        }
        Op::If(if_node) => {
            let cond = if_node.borrow().condition.clone();
            let then_branch = if_node.borrow().then_branch.clone();
            let else_branch = if_node.borrow().else_branch.clone();
            let new_cond = current_replace_map.get(&cond).unwrap().clone();
            let new_then_branch = current_replace_map.get(&then_branch).unwrap().clone();
            let new_else_branch = current_replace_map.get(&else_branch).unwrap().clone();
            let new_node = If::create(new_cond, new_then_branch, new_else_branch)
                .as_op()
                .into();
            current_replace_map.insert(node.clone(), new_node);
        }
        Op::For(for_node) => {
            let iterators = for_node.borrow().iterators.clone();
            let body = for_node.borrow().expr.clone();
            let selector = for_node.borrow().selector.clone();
            let new_iterators = iterators
                .borrow()
                .iter()
                .cloned()
                .map(|iterator| current_replace_map.get(&iterator).unwrap().clone())
                .collect::<Vec<_>>()
                .into();
            let new_body = current_replace_map.get(&body).unwrap().clone();
            let new_selector = current_replace_map
                .get(&selector)
                .unwrap_or(&Link::new(Op::None))
                .clone();
            let new_node = For::create(new_iterators, new_body, new_selector)
                .as_op()
                .into();
            current_replace_map.insert(node.clone(), new_node);
        }
        Op::Call(call) => {
            let arguments = call.borrow().arguments.clone();
            let function = call.borrow().function.clone();
            let new_arguments = arguments
                .borrow()
                .iter()
                .cloned()
                .map(|argument| current_replace_map.get(&argument).unwrap().clone())
                .collect::<Vec<_>>()
                .into();
            let new_node = Call::create(function, new_arguments).as_op().into();
            current_replace_map.insert(node.clone(), new_node);
        }
        Op::Fold(fold) => {
            let iterator = fold.borrow().iterator.clone();
            let operator = fold.borrow().operator.clone();
            let initial_value = fold.borrow().initial_value.clone();
            let new_iterator = current_replace_map.get(&iterator).unwrap().clone();
            let new_initial_value = current_replace_map.get(&initial_value).unwrap().clone();
            let new_node = Fold::create(new_iterator, operator, new_initial_value)
                .as_op()
                .into();
            current_replace_map.insert(node.clone(), new_node);
        }
        Op::Vector(vector) => {
            let children_link = vector.children().clone();
            let children_ref = children_link.borrow();
            let children = children_ref.deref();
            let new_children = children
                .iter()
                .cloned()
                .map(|child| current_replace_map.get(&child).unwrap().clone())
                .collect();
            let new_node = Vector::create(new_children).as_op().into();
            current_replace_map.insert(node.clone(), new_node);
        }
        Op::Matrix(matrix) => {
            let mut new_matrix = Vec::new();
            let children_link = matrix.children().clone();
            let children_ref = children_link.borrow();
            let children = children_ref.deref();
            for row in children.iter() {
                let row_children_link = row
                    .clone()
                    .as_vector()
                    .expect(format!("expected Vector, found {:?}", row).as_str())
                    .children()
                    .clone();
                let row_children_ref = row_children_link.borrow();
                let row_children = row_children_ref.deref();
                let new_row_as_vec = row_children
                    .iter()
                    .cloned()
                    .map(|child| current_replace_map.get(&child).unwrap().clone())
                    .collect::<Vec<_>>()
                    .into();
                let new_row = Vector::create(new_row_as_vec).into();
                new_matrix.push(new_row);
            }
            let new_node = Matrix::create(new_matrix).as_op().into();
            current_replace_map.insert(node.clone(), new_node);
        }
        Op::Accessor(accessor) => {
            let indexable = accessor.borrow().indexable.clone();
            let access_type = accessor.borrow().access_type.clone();
            let new_indexable = current_replace_map.get(&indexable).unwrap().clone();
            let new_node = Accessor::create(new_indexable, access_type).as_op().into();
            current_replace_map.insert(node.clone(), new_node);
        }
        Op::Parameter(parameter) => {
            let new_node = replace_parameter_list[parameter.borrow().position.clone()].clone();
            current_replace_map.insert(node.clone(), new_node);
        }
        Op::Value(value) => {
            let new_node = Value::create(value.borrow().value.clone()).as_op().into();
            current_replace_map.insert(node.clone(), new_node);
        }
        Op::None => {}
    }
}
